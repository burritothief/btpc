use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use btpc_core::ErrorCategory;
use btpc_core::create::{
    CancellationToken, HashProgress, ManifestEntry, ManifestOptions, NoProgress,
    ParallelHashOptions, ProgressSink, RootName, hash_v1_parallel, hash_v1_sequential,
    scan_manifest, sort_manifest_entries,
};
use tempfile::TempDir;

fn entry(path: &std::path::Path, torrent_name: &[u8]) -> ManifestEntry {
    let options = ManifestOptions::builder()
        .root_name(RootName::Override(torrent_name.to_vec()))
        .build()
        .unwrap();
    scan_manifest(path, &options).unwrap().entries()[0].clone()
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut output, byte| {
        write!(output, "{byte:02x}").unwrap();
        output
    })
}

#[test]
fn hashes_empty_exact_partial_and_multi_piece_payloads() {
    let temp = TempDir::new().unwrap();
    for (name, data, expected) in [
        ("empty", b"".as_slice(), vec![]),
        (
            "exact",
            b"abcdefgh",
            vec!["425af12a0743502b322e93a015bcf868e324d56a"],
        ),
        (
            "partial",
            b"abcdefghij",
            vec![
                "425af12a0743502b322e93a015bcf868e324d56a",
                "4cfa380a7a05ae26270f5ea888009520ab54b677",
            ],
        ),
        (
            "multi",
            b"abcdefgh",
            vec![
                "81fe8bfe87576c3ecb22426f8e57847382917acf",
                "2aed8aa9f826c21ef07d5ee15b48eea06e9c8a62",
            ],
        ),
    ] {
        let path = temp.path().join(name);
        fs::write(&path, data).unwrap();
        let piece_length = if name == "multi" { 4 } else { 8 };
        let result = hash_v1_sequential(
            &[entry(&path, name.as_bytes())],
            piece_length,
            &CancellationToken::new(),
            &NoProgress,
        )
        .unwrap();
        assert_eq!(result.total_bytes(), data.len() as u64);
        assert_eq!(
            result
                .pieces()
                .iter()
                .map(|piece| hex(piece))
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn pieces_cross_file_boundaries_in_manifest_order() {
    let temp = TempDir::new().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    fs::write(&first, b"abc").unwrap();
    fs::write(&second, b"defghij").unwrap();
    let entries = sort_manifest_entries(vec![entry(&second, b"b"), entry(&first, b"a")]).unwrap();

    let result = hash_v1_sequential(&entries, 8, &CancellationToken::new(), &NoProgress).unwrap();
    assert_eq!(
        hex(&result.pieces()[0]),
        "425af12a0743502b322e93a015bcf868e324d56a"
    );
    assert_eq!(
        hex(&result.pieces()[1]),
        "4cfa380a7a05ae26270f5ea888009520ab54b677"
    );
}

#[test]
fn detects_snapshot_size_changes_and_cancellation() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("file");
    fs::write(&path, b"abcd").unwrap();
    let stale = entry(&path, b"file");
    fs::write(&path, b"abcdefgh").unwrap();
    let error =
        hash_v1_sequential(&[stale], 4, &CancellationToken::new(), &NoProgress).unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Io);
    assert_eq!(error.path(), Some(path.as_path()));

    let cancelled = CancellationToken::new();
    cancelled.cancel();
    assert_eq!(
        hash_v1_sequential(&[entry(&path, b"file")], 4, &cancelled, &NoProgress)
            .unwrap_err()
            .category(),
        ErrorCategory::Cancelled
    );
}

#[derive(Default)]
struct RecordingProgress {
    events: Mutex<Vec<HashProgress>>,
    cancel_after_first: Option<Arc<AtomicBool>>,
}

impl ProgressSink for RecordingProgress {
    fn on_progress(&self, progress: HashProgress) {
        self.events.lock().unwrap().push(progress);
        if let Some(cancel) = &self.cancel_after_first {
            cancel.store(true, Ordering::Release);
        }
    }
}

#[test]
fn progress_is_monotonic_and_large_files_stream_deterministically() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("large");
    let data = vec![0x5a; 8 * 1024 * 1024 + 17];
    fs::write(&path, &data).unwrap();
    let progress = RecordingProgress::default();
    let entries = [entry(&path, b"large")];

    let first =
        hash_v1_sequential(&entries, 16 * 1024, &CancellationToken::new(), &progress).unwrap();
    let second =
        hash_v1_sequential(&entries, 16 * 1024, &CancellationToken::new(), &NoProgress).unwrap();
    assert_eq!(first.pieces(), second.pieces());
    assert_eq!(first.piece_count(), 513);
    let events = progress.events.lock().unwrap();
    assert!(!events.is_empty());
    assert!(
        events
            .windows(2)
            .all(|pair| pair[0].bytes_hashed() <= pair[1].bytes_hashed())
    );
    assert_eq!(events.last().unwrap().bytes_hashed(), data.len() as u64);
}

#[test]
fn bounded_parallel_matches_oracle_with_one_slot_backpressure() {
    let temp = TempDir::new().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    fs::write(
        &first,
        (0..131_099)
            .map(|value| u8::try_from(value % 256).unwrap())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    fs::write(&second, vec![0xa5; 77_777]).unwrap();
    let entries = sort_manifest_entries(vec![entry(&second, b"b"), entry(&first, b"a")]).unwrap();
    let oracle =
        hash_v1_sequential(&entries, 16 * 1024, &CancellationToken::new(), &NoProgress).unwrap();

    let progress = RecordingProgress::default();
    let parallel = hash_v1_parallel(
        &entries,
        16 * 1024,
        ParallelHashOptions::new(4, 1).unwrap(),
        &CancellationToken::new(),
        &progress,
    )
    .unwrap();

    assert_eq!(parallel, oracle);
    let events = progress.events.lock().unwrap();
    assert_eq!(events.last().unwrap().bytes_hashed(), oracle.total_bytes());
    assert!(
        events
            .windows(2)
            .all(|pair| pair[0].pieces_hashed() < pair[1].pieces_hashed())
    );
}

#[test]
fn parallel_options_reject_zero_and_report_bounds() {
    assert!(ParallelHashOptions::new(0, 1).is_err());
    assert!(ParallelHashOptions::new(1, 0).is_err());
    let options = ParallelHashOptions::new(3, 2).unwrap();
    assert_eq!(options.workers(), 3);
    assert_eq!(options.queue_capacity(), 2);
    assert!(ParallelHashOptions::automatic().workers() <= 2);
}

#[test]
fn parallel_cancellation_shutdown_does_not_hang() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("large-parallel");
    fs::write(&path, vec![0x33; 16 * 1024 * 1024]).unwrap();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let error = hash_v1_parallel(
        &[entry(&path, b"large-parallel")],
        64 * 1024,
        ParallelHashOptions::new(4, 2).unwrap(),
        &cancellation,
        &NoProgress,
    )
    .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Cancelled);
}

proptest::proptest! {
    #[test]
    fn parallel_hashing_matches_sequential_for_generated_file_streams(
        chunks in proptest::collection::vec(proptest::collection::vec(proptest::num::u8::ANY, 0..50_000), 1..8),
        piece_exponent in 10_u32..17,
        workers in 2_usize..6,
        queue_capacity in 1_usize..5,
    ) {
        let temp = TempDir::new().unwrap();
        let mut entries = Vec::new();
        for (index, chunk) in chunks.iter().enumerate() {
            let path = temp.path().join(format!("file-{index:02}"));
            fs::write(&path, chunk).unwrap();
            entries.push(entry(&path, format!("file-{index:02}").as_bytes()));
        }
        let piece_length = 1_u64 << piece_exponent;
        let oracle = hash_v1_sequential(
            &entries,
            piece_length,
            &CancellationToken::new(),
            &NoProgress,
        ).unwrap();
        let parallel = hash_v1_parallel(
            &entries,
            piece_length,
            ParallelHashOptions::new(workers, queue_capacity).unwrap(),
            &CancellationToken::new(),
            &NoProgress,
        ).unwrap();
        proptest::prop_assert_eq!(parallel, oracle);
    }
}

struct CancellingProgress {
    cancellation: CancellationToken,
    seen: AtomicBool,
}

impl ProgressSink for CancellingProgress {
    fn on_progress(&self, _progress: HashProgress) {
        if !self.seen.swap(true, Ordering::AcqRel) {
            self.cancellation.cancel();
        }
    }
}

#[test]
fn parallel_midflight_cancellation_stops_the_pipeline() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("cancel-midflight");
    fs::write(&path, vec![0x71; 64 * 1024 * 1024]).unwrap();
    let cancellation = CancellationToken::new();
    let progress = CancellingProgress {
        cancellation: cancellation.clone(),
        seen: AtomicBool::new(false),
    };
    let error = hash_v1_parallel(
        &[entry(&path, b"cancel-midflight")],
        64 * 1024,
        ParallelHashOptions::new(2, 1).unwrap(),
        &cancellation,
        &progress,
    )
    .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Cancelled);
}
