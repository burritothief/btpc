use std::fs;

use btpc_core::create::{CreateMode, CreateOptions, Creator, HashThreads, NoProgress, PieceLength};

// Spec: CREATE-V2-001
use btpc_core::{Metainfo, TorrentMode};

#[test]
fn creates_single_file_v2_golden_with_only_required_fields() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload.bin");
    fs::write(&payload, b"abcdefghij").unwrap();
    let options = CreateOptions::builder()
        .mode(CreateMode::V2)
        .piece_length(PieceLength::Exact(16_384))
        .private(true)
        .source(b"source".to_vec())
        .build()
        .unwrap();

    let first = Creator::new(&payload)
        .options(options.clone())
        .create(&NoProgress)
        .unwrap();
    let second = Creator::new(&payload)
        .options(options)
        .create(&NoProgress)
        .unwrap();
    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(first.mode(), CreateMode::V2);
    assert_eq!(first.info_hash_v1(), None);
    assert!(first.info_hash_v2().is_some());
    assert_eq!(first.piece_count(), 1);
    assert_eq!(first.piece_length(), 16_384);

    let parsed = Metainfo::from_bytes(first.bytes()).unwrap();
    assert_eq!(parsed.mode(), TorrentMode::V2);
    assert_eq!(parsed.info_hash_v1(), None);
    assert_eq!(parsed.info_hash_v2(), first.info_hash_v2());
    assert_eq!(
        first.info_hash_v2().unwrap().hex(),
        "b9a3493f653ece0b3926b134d5043d012c4ac0f0511b6d5eebc2ad0bb4addf7d"
    );
    assert_eq!(
        parsed.files()[0].pieces_root().unwrap(),
        &[
            0x72, 0x39, 0x93, 0x61, 0xda, 0x6a, 0x77, 0x54, 0xfe, 0xc9, 0x86, 0xdc, 0xa5, 0xb7,
            0xcb, 0xaf, 0x1c, 0x81, 0x0a, 0x28, 0xde, 0xd4, 0xab, 0xaf, 0x56, 0xb2, 0x10, 0x6d,
            0x06, 0xcb, 0x78, 0xb0,
        ]
    );
    assert!(
        !first
            .bytes()
            .windows(b"6:pieces20:".len())
            .any(|window| window == b"6:pieces20:")
    );
}

#[test]
fn bounded_per_file_threads_match_the_v2_oracle() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("tree");
    for index in 0..128 {
        let path = payload
            .join(format!("dir-{:02}", index % 8))
            .join(format!("file-{index:03}"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, vec![u8::try_from(index).unwrap(); 32 * 1024]).unwrap();
    }
    let build = |threads| {
        Creator::new(&payload)
            .options(
                CreateOptions::builder()
                    .mode(CreateMode::V2)
                    .piece_length(PieceLength::Exact(16 * 1024))
                    .hash_threads(HashThreads::Exact(threads))
                    .build()
                    .unwrap(),
            )
            .create(&NoProgress)
            .unwrap()
    };
    let sequential = build(1);
    let parallel = build(2);
    assert_eq!(parallel.bytes(), sequential.bytes());
    assert_eq!(parallel.info_hash_v2(), sequential.info_hash_v2());
}

#[test]
fn creates_nested_v2_tree_empty_files_and_only_required_piece_layers() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir_all(payload.join("nested")).unwrap();
    fs::write(payload.join("empty"), []).unwrap();
    fs::write(payload.join("nested/small"), b"small").unwrap();
    fs::write(payload.join("large"), vec![7_u8; 32_769]).unwrap();
    let options = CreateOptions::builder()
        .mode(CreateMode::V2)
        .piece_length(PieceLength::Exact(16_384))
        .build()
        .unwrap();

    let result = Creator::new(&payload)
        .options(options)
        .create(&NoProgress)
        .unwrap();
    let parsed = Metainfo::from_bytes(result.bytes()).unwrap();
    assert_eq!(parsed.mode(), TorrentMode::V2);
    assert_eq!(parsed.files().len(), 3);
    assert_eq!(parsed.files()[0].path_components(), &[b"empty".to_vec()]);
    assert_eq!(parsed.files()[0].pieces_root(), None);
    assert_eq!(parsed.files()[1].path_components(), &[b"large".to_vec()]);
    assert_eq!(parsed.files()[1].length().div_ceil(16_384), 3);
    assert_eq!(
        parsed.files()[2].path_components(),
        &[b"nested".to_vec(), b"small".to_vec()]
    );
    assert!(parsed.files()[2].length() < 16_384);
    assert_eq!(result.piece_count(), 4);
}

#[test]
fn boundary_sizes_and_duplicate_roots_are_canonical() {
    for length in [16_383, 16_384, 16_385] {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        fs::write(&payload, vec![3_u8; length]).unwrap();
        let options = CreateOptions::builder()
            .mode(CreateMode::V2)
            .piece_length(PieceLength::Exact(16_384))
            .build()
            .unwrap();
        let result = Creator::new(&payload)
            .options(options)
            .create(&NoProgress)
            .unwrap();
        let parsed = Metainfo::from_bytes(result.bytes()).unwrap();
        assert_eq!(parsed.files()[0].length(), length as u64);
        assert_eq!(result.piece_count(), length.div_ceil(16_384));
    }

    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir(&payload).unwrap();
    let contents = vec![9_u8; 16_385];
    fs::write(payload.join("a"), &contents).unwrap();
    fs::write(payload.join("b"), &contents).unwrap();
    let options = CreateOptions::builder()
        .mode(CreateMode::V2)
        .piece_length(PieceLength::Exact(16_384))
        .build()
        .unwrap();
    let result = Creator::new(&payload)
        .options(options)
        .create(&NoProgress)
        .unwrap();
    let raw = btpc_core::bencode::parse(result.bytes()).unwrap();
    let layers = raw.get(b"piece layers").unwrap();
    let btpc_core::bencode::ValueKind::Dictionary(layers) = layers.kind() else {
        panic!("piece layers must be a dictionary");
    };
    assert_eq!(layers.len(), 1);
}
