use std::fs;

use btpc_core::bencode::ValueKind;
use btpc_core::create::{CreateMode, CreateOptions, Creator, HashThreads, NoProgress, PieceLength};
use btpc_core::{Metainfo, TorrentMode};
use sha1::{Digest as _, Sha1};

// Spec: CREATE-HYBRID-001

fn hybrid_options() -> CreateOptions {
    CreateOptions::builder()
        .mode(CreateMode::Hybrid)
        .piece_length(PieceLength::Exact(16_384))
        .build()
        .unwrap()
}

#[test]
fn bounded_threads_match_hybrid_oracle_with_many_padding_files() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("tree");
    fs::create_dir(&payload).unwrap();
    for index in 0..96 {
        fs::write(
            payload.join(format!("file-{index:03}")),
            vec![u8::try_from(index).unwrap(); 4_097],
        )
        .unwrap();
    }
    let build = |threads| {
        Creator::new(&payload)
            .options(
                CreateOptions::builder()
                    .mode(CreateMode::Hybrid)
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
    assert_eq!(parallel.info_hash_v1(), sequential.info_hash_v1());
    assert_eq!(parallel.info_hash_v2(), sequential.info_hash_v2());
}

#[test]
fn single_file_hybrid_has_both_independently_verified_hashes() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"abcdefghij").unwrap();
    let result = Creator::new(&payload)
        .options(hybrid_options())
        .create(&NoProgress)
        .unwrap();
    assert_eq!(result.mode(), CreateMode::Hybrid);
    assert!(result.info_hash_v1().is_some());
    assert!(result.info_hash_v2().is_some());
    let parsed = Metainfo::from_bytes(result.bytes()).unwrap();
    assert_eq!(parsed.mode(), TorrentMode::Hybrid);
    assert_eq!(parsed.info_hash_v1(), result.info_hash_v1());
    assert_eq!(parsed.info_hash_v2(), result.info_hash_v2());

    let root = btpc_core::bencode::parse(result.bytes()).unwrap();
    let info = root.get(b"info").unwrap();
    let pieces = info.get(b"pieces").unwrap().as_bytes().unwrap();
    assert_eq!(pieces, Sha1::digest(b"abcdefghij").as_slice());
}

#[test]
fn multifile_hybrid_inserts_unique_padding_and_validates() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir_all(payload.join("nested")).unwrap();
    fs::write(payload.join("a"), b"abc").unwrap();
    fs::write(payload.join("empty"), []).unwrap();
    fs::write(payload.join("nested/b"), b"def").unwrap();
    fs::write(payload.join("nested/c"), b"ghi").unwrap();

    let result = Creator::new(&payload)
        .options(hybrid_options())
        .create(&NoProgress)
        .unwrap();
    let parsed = Metainfo::from_bytes(result.bytes()).unwrap();
    assert_eq!(parsed.mode(), TorrentMode::Hybrid);
    assert_eq!(parsed.files().len(), 6);
    assert_eq!(
        parsed
            .files()
            .iter()
            .filter(|file| file.is_padding())
            .count(),
        2
    );

    let root = btpc_core::bencode::parse(result.bytes()).unwrap();
    let raw = btpc_core::metainfo::RawMetainfo::from_bytes(result.bytes()).unwrap();
    let v1 = btpc_core::metainfo::V1Metainfo::from_raw(&raw).unwrap();
    let info = root.get(b"info").unwrap();
    let ValueKind::List(files) = info.get(b"files").unwrap().kind() else {
        panic!("hybrid files must be a list");
    };
    let pads = files
        .iter()
        .filter(|file| {
            file.get(b"attr")
                .and_then(btpc_core::bencode::Value::as_bytes)
                == Some(b"p")
        })
        .collect::<Vec<_>>();
    assert_eq!(pads.len(), 2);
    let paths = pads
        .iter()
        .map(|file| {
            let ValueKind::List(path) = file.get(b"path").unwrap().kind() else {
                panic!("path must be a list");
            };
            path.iter()
                .map(|part| part.as_bytes().unwrap())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_ne!(paths[0], paths[1]);
    assert!(paths.iter().all(|path| path[0] == b".pad"));
    assert!(
        v1.files()
            .iter()
            .any(btpc_core::metainfo::V1File::is_padding)
    );
    for file in v1.files().iter().filter(|file| file.is_padding()) {
        assert_eq!(file.path_components().len(), 2);
        assert_eq!(file.path_components()[0], b".pad");
    }

    let pieces = info.get(b"pieces").unwrap().as_bytes().unwrap();
    let mut logical = Vec::new();
    for file in files {
        let length = usize::try_from(file.get(b"length").unwrap().as_integer().unwrap()).unwrap();
        if file
            .get(b"attr")
            .and_then(btpc_core::bencode::Value::as_bytes)
            == Some(b"p")
        {
            logical.resize(logical.len() + length, 0);
            continue;
        }
        let ValueKind::List(path) = file.get(b"path").unwrap().kind() else {
            panic!("path must be a list");
        };
        let mut source = payload.clone();
        for part in path {
            source.push(std::str::from_utf8(part.as_bytes().unwrap()).unwrap());
        }
        logical.extend(fs::read(source).unwrap());
    }
    let expected = logical
        .chunks(16_384)
        .flat_map(|piece| Sha1::digest(piece).to_vec())
        .collect::<Vec<_>>();
    assert_eq!(pieces, expected);
}

#[test]
fn exact_boundaries_need_no_padding_and_reserved_pad_paths_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir(&payload).unwrap();
    fs::write(payload.join("a"), vec![0_u8; 16_384]).unwrap();
    fs::write(payload.join("b"), b"b").unwrap();
    let result = Creator::new(&payload)
        .options(hybrid_options())
        .create(&NoProgress)
        .unwrap();
    assert!(!result.bytes().windows(6).any(|window| window == b"4:attr"));

    let collision = temp.path().join("collision");
    fs::create_dir_all(collision.join(".pad")).unwrap();
    fs::write(collision.join("a"), b"a").unwrap();
    fs::write(collision.join(".pad/real"), b"real").unwrap();
    assert!(
        Creator::new(collision)
            .options(hybrid_options())
            .create(&NoProgress)
            .is_err()
    );
}
