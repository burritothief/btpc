use std::fs::{self, File};
use std::hint::black_box;
use std::io::Write;
use std::path::Path;

use btpc_core::bencode::{self, OwnedValue};
use btpc_core::create::{
    CancellationToken, CreateMode, CreateOptions, Creator, HashThreads, ManifestOptions,
    NoProgress, ParallelHashOptions, PieceLength, hash_v1_parallel, hash_v1_sequential,
    hash_v2_file_sequential, scan_manifest, sort_manifest_entries,
};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tempfile::TempDir;

const PIECE_LENGTH: u64 = 256 * 1024;
const PAYLOAD_SIZE: usize = 8 * 1024 * 1024;
const FILE_COUNT: usize = 2_048;

fn deterministic_bytes(size: usize) -> Vec<u8> {
    (0..size)
        .map(|index| {
            u8::try_from((index.wrapping_mul(31).wrapping_add(index / 251)) % 256)
                .expect("fixture byte is reduced modulo 256")
        })
        .collect()
}

fn write_payload(path: &Path, size: usize) {
    let mut file = File::create(path).expect("create benchmark payload");
    for chunk in deterministic_bytes(1024 * 1024).chunks(64 * 1024) {
        if file.metadata().expect("payload metadata").len() >= size as u64 {
            break;
        }
        file.write_all(chunk).expect("write benchmark payload");
    }
    file.set_len(size as u64).expect("size benchmark payload");
}

fn bencode_fixture() -> (Vec<u8>, OwnedValue) {
    let files = (0..1_024)
        .map(|index| {
            OwnedValue::dictionary([
                (
                    b"length".to_vec(),
                    OwnedValue::integer(i64::from(index) * 4_096),
                ),
                (
                    b"path".to_vec(),
                    OwnedValue::list([OwnedValue::bytes(format!("file-{index:04}.bin"))]),
                ),
            ])
            .expect("unique fixture keys")
        })
        .collect::<Vec<_>>();
    let value = OwnedValue::dictionary([
        (
            b"announce".to_vec(),
            OwnedValue::bytes("https://tracker.invalid"),
        ),
        (
            b"info".to_vec(),
            OwnedValue::dictionary([
                (b"files".to_vec(), OwnedValue::list(files)),
                (b"name".to_vec(), OwnedValue::bytes("fixture")),
                (
                    b"piece length".to_vec(),
                    OwnedValue::integer(
                        i64::try_from(PIECE_LENGTH).expect("piece length fits i64"),
                    ),
                ),
                (
                    b"pieces".to_vec(),
                    OwnedValue::bytes(vec![7_u8; 20 * 2_048]),
                ),
            ])
            .expect("unique fixture keys"),
        ),
    ])
    .expect("unique fixture keys");
    let encoded = value.to_vec().expect("encode fixture");
    (encoded, value)
}

fn bencode_benchmarks(criterion: &mut Criterion) {
    let (encoded, owned) = bencode_fixture();
    let mut group = criterion.benchmark_group("bencode");
    group.throughput(Throughput::Bytes(encoded.len() as u64));
    group.bench_function("bencode_parse", |bencher| {
        bencher.iter(|| bencode::parse(black_box(&encoded)).expect("parse benchmark fixture"));
    });
    group.bench_function("bencode_encode", |bencher| {
        bencher.iter(|| {
            black_box(&owned)
                .to_vec()
                .expect("encode benchmark fixture")
        });
    });
    group.finish();
}

fn manifest_benchmarks(criterion: &mut Criterion) {
    let sort_directory = TempDir::new().expect("manifest sort tempdir");
    for index in 0..FILE_COUNT {
        let child = sort_directory
            .path()
            .join(format!("dir-{:03}", index % 31))
            .join(format!("file-{index:05}"));
        fs::create_dir_all(child.parent().expect("fixture child has parent"))
            .expect("create sort fixture directory");
        fs::write(child, [u8::try_from(index % 251).expect("fixture byte")])
            .expect("write sort fixture file");
    }
    let mut entries = scan_manifest(sort_directory.path(), &ManifestOptions::default())
        .expect("scan sort fixture")
        .entries()
        .to_vec();
    entries.reverse();
    let mut group = criterion.benchmark_group("manifest");
    group.throughput(Throughput::Elements(entries.len() as u64));
    group.bench_function("manifest_sort", |bencher| {
        bencher.iter_batched(
            || entries.clone(),
            |input| black_box(sort_manifest_entries(input).expect("sort benchmark manifest")),
            BatchSize::SmallInput,
        );
    });
    let directory = TempDir::new().expect("manifest scan tempdir");
    for index in 0..2_048 {
        let child = directory
            .path()
            .join(format!("dir-{:02}", index % 32))
            .join(format!("file-{index:04}.bin"));
        fs::create_dir_all(child.parent().expect("fixture parent")).expect("create fixture dir");
        fs::write(child, [u8::try_from(index % 251).expect("fixture byte")])
            .expect("write fixture file");
    }
    group.bench_function("manifest_scan", |bencher| {
        bencher.iter(|| {
            black_box(
                scan_manifest(directory.path(), &ManifestOptions::default())
                    .expect("scan benchmark fixture"),
            )
        });
    });
    group.finish();
}

fn hashing_benchmarks(criterion: &mut Criterion) {
    let directory = TempDir::new().expect("benchmark tempdir");
    let payload = directory.path().join("payload.bin");
    write_payload(&payload, PAYLOAD_SIZE);
    let manifest = scan_manifest(&payload, &ManifestOptions::default()).expect("scan fixture");
    let entry = manifest.entries()[0].clone();
    let cancellation = CancellationToken::new();
    let mut group = criterion.benchmark_group("hashing");
    group.throughput(Throughput::Bytes(PAYLOAD_SIZE as u64));
    group.bench_with_input(
        BenchmarkId::new("v1_piece_hashing/sequential", PIECE_LENGTH),
        &manifest,
        |bencher, manifest| {
            bencher.iter(|| {
                black_box(
                    hash_v1_sequential(
                        manifest.entries(),
                        PIECE_LENGTH,
                        &cancellation,
                        &NoProgress,
                    )
                    .expect("hash v1 fixture"),
                )
            });
        },
    );
    group.bench_with_input(
        BenchmarkId::new("v1_piece_hashing/parallel-4", PIECE_LENGTH),
        &manifest,
        |bencher, manifest| {
            bencher.iter(|| {
                black_box(
                    hash_v1_parallel(
                        manifest.entries(),
                        PIECE_LENGTH,
                        ParallelHashOptions::new(4, 4).expect("valid benchmark options"),
                        &cancellation,
                        &NoProgress,
                    )
                    .expect("hash parallel v1 fixture"),
                )
            });
        },
    );
    group.bench_with_input(
        BenchmarkId::new("v2_merkle_hashing", PIECE_LENGTH),
        &entry,
        |bencher, entry| {
            bencher.iter(|| {
                black_box(
                    hash_v2_file_sequential(entry, PIECE_LENGTH, &cancellation, &NoProgress)
                        .expect("hash v2 fixture"),
                )
            });
        },
    );
    group.finish();
}

fn v2_file_tree_benchmark(criterion: &mut Criterion) {
    let directory = TempDir::new().expect("benchmark tempdir");
    for index in 0..256 {
        let child = directory
            .path()
            .join(format!("dir-{:02}", index % 16))
            .join(format!("file-{index:04}.bin"));
        fs::create_dir_all(child.parent().expect("fixture parent")).expect("create fixture dir");
        write_payload(&child, 4_096);
    }
    for (mode, threads, name) in [
        (CreateMode::V2, 1, "v2_file_tree_creation/sequential"),
        (CreateMode::V2, 2, "v2_file_tree_creation/parallel-2"),
        (
            CreateMode::Hybrid,
            1,
            "hybrid_file_tree_creation/sequential",
        ),
        (
            CreateMode::Hybrid,
            2,
            "hybrid_file_tree_creation/parallel-2",
        ),
    ] {
        let options = CreateOptions::builder()
            .mode(mode)
            .piece_length(PieceLength::Exact(PIECE_LENGTH))
            .hash_threads(HashThreads::Exact(threads))
            .build()
            .expect("benchmark create options");
        criterion.bench_function(name, |bencher| {
            bencher.iter(|| {
                black_box(
                    Creator::new(directory.path())
                        .options(options.clone())
                        .create(&NoProgress)
                        .expect("create benchmark fixture"),
                )
            });
        });
    }
}

criterion_group!(
    core_benches,
    bencode_benchmarks,
    manifest_benchmarks,
    hashing_benchmarks,
    v2_file_tree_benchmark
);
criterion_main!(core_benches);
