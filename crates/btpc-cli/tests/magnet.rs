use std::fs;

use assert_cmd::Command;
use btpc_core::Metainfo;
use btpc_core::create::{CreateMode, CreateOptions, Creator, NoProgress};
use btpc_core::magnet::MagnetOptions;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

#[test]
fn prints_only_core_magnet_for_every_mode() {
    for mode in [CreateMode::V1, CreateMode::V2, CreateMode::Hybrid] {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        let torrent = temp.path().join("payload.torrent");
        fs::write(&payload, b"magnet payload").unwrap();
        let options = CreateOptions::builder().mode(mode).build().unwrap();
        let result = Creator::new(&payload)
            .options(options)
            .create(&NoProgress)
            .unwrap();
        fs::write(&torrent, result.bytes()).unwrap();
        let expected = Metainfo::from_bytes(result.bytes())
            .unwrap()
            .magnet(&MagnetOptions::default());
        btpc()
            .args(["magnet", torrent.to_str().unwrap()])
            .assert()
            .success()
            .stdout(format!("{expected}\n"))
            .stderr("");
    }
}

#[test]
fn optional_parameters_can_be_omitted() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    let torrent = temp.path().join("payload.torrent");
    fs::write(&payload, b"data").unwrap();
    let result = Creator::new(&payload).create(&NoProgress).unwrap();
    fs::write(&torrent, result.bytes()).unwrap();
    let output = btpc()
        .args([
            "magnet",
            torrent.to_str().unwrap(),
            "--no-display-name",
            "--no-trackers",
            "--no-web-seeds",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    assert!(!output.contains("&dn="));
    assert!(!output.contains("&tr="));
    assert!(!output.contains("&ws="));
}
