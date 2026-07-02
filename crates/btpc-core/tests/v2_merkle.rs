use btpc_core::create::{
    CancellationToken, CreateMode, CreateOptions, Creator, HashProgress, ManifestEntry,
    ManifestOptions, NoProgress, PieceLength, ProgressSink, RootName, V2_BLOCK_LENGTH,
    hash_v2_file_sequential, scan_manifest, v2_zero_hash,
};
use btpc_core::verify::Verifier;
use btpc_core::{Metainfo, bencode};
use proptest::prelude::*;
use sha2::{Digest as _, Sha256};
use std::fmt::Write as _;
use std::sync::Mutex;

fn fixture_bytes(length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from(index % 251).unwrap())
        .collect()
}

fn hash(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .fold(String::new(), |mut output, byte| {
            write!(output, "{byte:02x}").unwrap();
            output
        })
}

fn pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut bytes = [0_u8; 64];
    bytes[..32].copy_from_slice(&left);
    bytes[32..].copy_from_slice(&right);
    hash(&bytes)
}

fn full_tree(bytes: &[u8], piece_length: u64) -> ([u8; 32], Vec<[u8; 32]>) {
    let mut leaves = bytes.chunks(V2_BLOCK_LENGTH).map(hash).collect::<Vec<_>>();
    assert!(!leaves.is_empty());
    let blocks_per_piece = usize::try_from(piece_length).unwrap() / V2_BLOCK_LENGTH;
    let mut piece_layer = Vec::new();
    if leaves.len() > blocks_per_piece {
        for chunk in leaves.chunks(blocks_per_piece) {
            let mut piece = chunk.to_vec();
            piece.resize(blocks_per_piece, [0; 32]);
            while piece.len() > 1 {
                piece = piece
                    .chunks_exact(2)
                    .map(|pair_| pair(pair_[0], pair_[1]))
                    .collect();
            }
            piece_layer.push(piece[0]);
        }
    }
    leaves.resize(leaves.len().next_power_of_two(), [0; 32]);
    while leaves.len() > 1 {
        leaves = leaves
            .chunks_exact(2)
            .map(|pair_| pair(pair_[0], pair_[1]))
            .collect();
    }
    (leaves[0], piece_layer)
}

fn entry(path: &std::path::Path) -> ManifestEntry {
    let options = ManifestOptions::builder()
        .root_name(RootName::Override(b"payload".to_vec()))
        .build()
        .unwrap();
    scan_manifest(path, &options).unwrap().entries()[0].clone()
}

struct CancelOnProgress(CancellationToken);

impl ProgressSink for CancelOnProgress {
    fn on_progress(&self, _progress: HashProgress) {
        self.0.cancel();
    }
}

#[test]
fn fixed_vectors_cover_blocks_zero_hashes_roots_and_piece_layers() {
    // These constants follow the independent BEP 52 reference creator's tree rules.
    let expected_zero_hashes = [
        "0000000000000000000000000000000000000000000000000000000000000000",
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
        "db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71",
        "c78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c",
    ];
    for (level, expected) in expected_zero_hashes.iter().enumerate() {
        assert_eq!(hex(v2_zero_hash(level)), *expected);
    }

    let cases = [
        (
            1,
            "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d",
            vec![],
        ),
        (
            16_384,
            "4348e3b98e8a327b34ced39c1da9e67cdb4cd5e48e4d7960607a3ae403d35f0c",
            vec![],
        ),
        (
            16_385,
            "9d7887c65d577a0237fb3c0998b87b3a62762d03796889a2caea01db914ccbb8",
            vec![],
        ),
        (
            32_768,
            "d9e13d0b676ad681164ef0b7b5910d1328ea83a047cad57e619d76bbe3a08525",
            vec![],
        ),
        (
            32_769,
            "934c793a4cecccfd47faaf6b69c17703875e6c58ffe623bcd31ff3b2170770d1",
            vec![
                "d9e13d0b676ad681164ef0b7b5910d1328ea83a047cad57e619d76bbe3a08525",
                "638b34336e01a45ec7e0bb471943ea880accff4be985e396f05e4e861d60536f",
            ],
        ),
        (
            81_921,
            "3960b8795a1921af5e02e44cd951a9edf944c1170187afbb81775f8b28b62f66",
            vec![
                "d9e13d0b676ad681164ef0b7b5910d1328ea83a047cad57e619d76bbe3a08525",
                "e28097eaaa55956702cf8195d1a551dbabb63e3d679b294cf33d506a6b5ef479",
                "242e3709c173d648fdf5b12a8532b299a3e1985e8f9c7cf55761b12ec4bfd643",
            ],
        ),
    ];
    for (length, expected_root, expected_layer) in cases {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("payload");
        std::fs::write(&path, fixture_bytes(length)).unwrap();
        let result = hash_v2_file_sequential(
            &entry(&path),
            32_768,
            &CancellationToken::new(),
            &NoProgress,
        )
        .unwrap();
        assert_eq!(hex(result.pieces_root().unwrap()), expected_root);
        assert_eq!(
            result.piece_layer().iter().map(hex).collect::<Vec<_>>(),
            expected_layer
        );
    }
}

#[test]
fn bep52_boundaries_agree_across_creation_parsing_and_verification() {
    let piece_length = 32_768_u64;
    for length in [0, 1, 16_383, 16_384, 16_385, 32_767, 32_768, 32_769, 81_921] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("payload");
        let bytes = fixture_bytes(length);
        std::fs::write(&path, &bytes).unwrap();
        let result = Creator::new(&path)
            .options(
                CreateOptions::builder()
                    .mode(CreateMode::V2)
                    .piece_length(PieceLength::Exact(piece_length))
                    .build()
                    .unwrap(),
            )
            .create(&NoProgress)
            .unwrap();
        let parsed = Metainfo::from_bytes(result.bytes()).unwrap();
        let file = &parsed.files()[0];
        let root = bencode::parse(result.bytes()).unwrap();
        let layers = root.get(b"piece layers");

        if bytes.is_empty() {
            assert_eq!(file.pieces_root(), None);
            assert!(layers.is_none());
        } else {
            let (expected_root, expected_layer) = full_tree(&bytes, piece_length);
            assert_eq!(file.pieces_root(), Some(&expected_root));
            if expected_layer.is_empty() {
                assert!(layers.is_none());
            } else {
                let layer = layers
                    .unwrap()
                    .get(&expected_root)
                    .unwrap()
                    .as_bytes()
                    .unwrap();
                assert_eq!(
                    layer,
                    expected_layer
                        .iter()
                        .flat_map(<[u8; 32]>::as_slice)
                        .copied()
                        .collect::<Vec<_>>()
                );
            }
        }

        let report = Verifier::new(&parsed, &path).verify(&NoProgress).unwrap();
        assert!(
            report.is_valid(),
            "length {length}: {:?}",
            report.mismatches()
        );
    }
}

#[test]
fn empty_files_mutations_cancellation_and_large_streaming_work() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("payload");
    std::fs::write(&path, []).unwrap();
    let empty = hash_v2_file_sequential(
        &entry(&path),
        16_384,
        &CancellationToken::new(),
        &NoProgress,
    )
    .unwrap();
    assert_eq!(empty.pieces_root(), None);
    assert!(empty.piece_layer().is_empty());

    std::fs::write(&path, fixture_bytes(8 * 1024 * 1024 + 1)).unwrap();
    let snapshot = entry(&path);
    let baseline =
        hash_v2_file_sequential(&snapshot, 65_536, &CancellationToken::new(), &NoProgress).unwrap();
    let mut changed = fixture_bytes(8 * 1024 * 1024 + 1);
    changed[4 * 1024 * 1024] ^= 1;
    std::fs::write(&path, changed).unwrap();
    let mutated = hash_v2_file_sequential(
        &entry(&path),
        65_536,
        &CancellationToken::new(),
        &NoProgress,
    )
    .unwrap();
    assert_ne!(baseline.pieces_root(), mutated.pieces_root());

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert!(hash_v2_file_sequential(&entry(&path), 65_536, &cancellation, &NoProgress).is_err());

    let small_path = directory.path().join("small");
    std::fs::write(&small_path, b"small").unwrap();
    let callback_cancellation = CancellationToken::new();
    assert!(
        hash_v2_file_sequential(
            &entry(&small_path),
            16_384,
            &callback_cancellation,
            &CancelOnProgress(callback_cancellation.clone()),
        )
        .is_err()
    );
}

#[derive(Default)]
struct Events(Mutex<Vec<HashProgress>>);
impl ProgressSink for Events {
    fn on_progress(&self, progress: HashProgress) {
        self.0.lock().unwrap().push(progress);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn streaming_matches_full_tree(bytes in proptest::collection::vec(any::<u8>(), 1..300_000), exponent in 0_u32..5) {
        let piece_length = 16_384_u64 << exponent;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("payload");
        std::fs::write(&path, &bytes).unwrap();
        let events = Events::default();
        let actual = hash_v2_file_sequential(&entry(&path), piece_length, &CancellationToken::new(), &events).unwrap();
        let expected = full_tree(&bytes, piece_length);
        prop_assert_eq!(actual.pieces_root(), Some(&expected.0));
        prop_assert_eq!(actual.piece_layer(), expected.1.as_slice());
        let events = events.0.lock().unwrap();
        prop_assert_eq!(events.last().map(|event| event.bytes_hashed()), Some(bytes.len() as u64));
    }
}
