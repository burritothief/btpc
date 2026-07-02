use std::fs;

use btpc_core::Metainfo;
use btpc_core::create::{CreateMode, CreateOptions, Creator, NoProgress};
use btpc_core::magnet::MagnetOptions;

#[test]
fn emits_published_hash_representations_for_every_mode() {
    for mode in [CreateMode::V1, CreateMode::V2, CreateMode::Hybrid] {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        fs::write(&payload, b"magnet payload").unwrap();
        let options = CreateOptions::builder().mode(mode).build().unwrap();
        let result = Creator::new(&payload)
            .options(options)
            .create(&NoProgress)
            .unwrap();
        let metainfo = Metainfo::from_bytes(result.bytes()).unwrap();
        let magnet = metainfo.magnet(&MagnetOptions::default());
        if let Some(hash) = result.info_hash_v1() {
            assert!(magnet.contains(&format!("xt=urn:btih:{}", hash.hex())));
        } else {
            assert!(!magnet.contains("urn:btih:"));
        }
        if let Some(hash) = result.info_hash_v2() {
            assert!(magnet.contains(&format!("xt=urn:btmh:1220{}", hash.hex())));
        } else {
            assert!(!magnet.contains("urn:btmh:"));
        }
    }
}

#[test]
fn encodes_name_trackers_and_web_seeds_in_deterministic_order() {
    use btpc_core::bencode::OwnedValue;
    let info = OwnedValue::dictionary([
        (b"length".to_vec(), OwnedValue::integer(1)),
        (b"name".to_vec(), OwnedValue::bytes(b"a b&\xff_c".to_vec())),
        (b"piece length".to_vec(), OwnedValue::integer(16_384)),
        (b"pieces".to_vec(), OwnedValue::bytes(vec![0; 20])),
    ])
    .unwrap();
    let bytes = OwnedValue::dictionary([
        (
            b"announce".to_vec(),
            OwnedValue::bytes(b"https://tracker/one?a=b".to_vec()),
        ),
        (
            b"announce-list".to_vec(),
            OwnedValue::list([
                OwnedValue::list([OwnedValue::bytes(b"https://tracker/one?a=b".to_vec())]),
                OwnedValue::list([OwnedValue::bytes(b"https://tracker/two".to_vec())]),
            ]),
        ),
        (b"info".to_vec(), info),
        (
            b"url-list".to_vec(),
            OwnedValue::list([OwnedValue::bytes(b"https://seed/a b?x=y".to_vec())]),
        ),
    ])
    .unwrap()
    .to_vec()
    .unwrap();
    let metainfo = Metainfo::from_bytes(&bytes).unwrap();
    let magnet = metainfo.magnet(&MagnetOptions::default());
    assert!(magnet.starts_with("magnet:?xt=urn:btih:"));
    assert!(magnet.contains("&dn=a%20b%26%FF_c"));
    assert!(magnet.contains("&tr=https%3A%2F%2Ftracker%2Fone%3Fa%3Db"));
    assert!(magnet.contains("&tr=https%3A%2F%2Ftracker%2Ftwo"));
    assert!(magnet.ends_with("&ws=https%3A%2F%2Fseed%2Fa%20b%3Fx%3Dy"));
    assert_eq!(magnet, metainfo.magnet(&MagnetOptions::default()));
}

#[test]
fn options_can_omit_optional_parameters() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"data").unwrap();
    let result = Creator::new(&payload).create(&NoProgress).unwrap();
    let metainfo = Metainfo::from_bytes(result.bytes()).unwrap();
    let magnet = metainfo.magnet(
        &MagnetOptions::builder()
            .display_name(false)
            .trackers(false)
            .web_seeds(false)
            .build(),
    );
    assert!(!magnet.contains("&dn="));
    assert!(!magnet.contains("&tr="));
    assert!(!magnet.contains("&ws="));
}
