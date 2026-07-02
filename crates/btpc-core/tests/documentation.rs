use std::fs;

use btpc_core::Metainfo;
use btpc_core::create::{CreateMode, CreateOptions, Creator, HashThreads, NoProgress, PieceLength};
use btpc_core::magnet::MagnetOptions;
use btpc_core::verify::Verifier;
use tempfile::TempDir;

// Spec: RUSTAPI-DOC-001
#[test]
fn rust_guide_create_parse_magnet_and_verify_example_runs() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir_all(&payload).unwrap();
    fs::write(payload.join("hello.txt"), b"hello torrent\n").unwrap();
    let options = CreateOptions::builder()
        .mode(CreateMode::Hybrid)
        .piece_length(PieceLength::Exact(16_384))
        .hash_threads(HashThreads::Exact(1))
        .creation_date(0)
        .build()
        .unwrap();
    let result = Creator::new(&payload)
        .options(options)
        .create(&NoProgress)
        .unwrap();
    let torrent = Metainfo::from_bytes(result.bytes()).unwrap();
    assert!(torrent.info_hash_v1().is_some());
    assert!(torrent.info_hash_v2().is_some());
    assert!(
        torrent
            .magnet(&MagnetOptions::default())
            .starts_with("magnet:?xt=")
    );
    assert!(
        Verifier::new(&torrent, &payload)
            .verify(&NoProgress)
            .unwrap()
            .is_valid()
    );
}
