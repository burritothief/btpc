use std::fs;

use assert_cmd::Command;
use btpc_core::create::{CreateMode, CreateOptions, Creator, NoProgress};
use predicates::prelude::*;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

fn fixture(mode: CreateMode) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    let torrent = temp.path().join("payload.torrent");
    fs::write(&payload, b"verification payload").unwrap();
    let options = CreateOptions::builder().mode(mode).build().unwrap();
    let result = Creator::new(&payload)
        .options(options)
        .create(&NoProgress)
        .unwrap();
    fs::write(&torrent, result.bytes()).unwrap();
    (temp, payload, torrent)
}

#[test]
fn verify_human_and_json_cover_valid_and_mismatch_results() {
    for mode in [CreateMode::V1, CreateMode::V2, CreateMode::Hybrid] {
        let (_temp, payload, torrent) = fixture(mode);
        btpc()
            .args([
                "verify",
                torrent.to_str().unwrap(),
                payload.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout("valid\n")
            .stderr("");

        fs::write(&payload, b"verification payloae").unwrap();
        let assertion = btpc()
            .args([
                "verify",
                torrent.to_str().unwrap(),
                payload.to_str().unwrap(),
                "--json",
            ])
            .assert()
            .code(6)
            .stderr("");
        let value: serde_json::Value =
            serde_json::from_slice(&assertion.get_output().stdout).unwrap();
        assert_eq!(value["schema"], "btpc.verify.v2");
        assert_eq!(value["valid"], false);
        let mismatch = &value["mismatches"][0];
        assert_eq!(mismatch["path"]["schema"], "btpc.filesystem-path.v2");
        assert!(mismatch["path_display"].is_string());
    }
}

#[test]
fn verify_controls_fail_fast_and_extra_files() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    let torrent = temp.path().join("payload.torrent");
    fs::create_dir(&payload).unwrap();
    fs::write(payload.join("a"), b"a").unwrap();
    fs::write(payload.join("b"), b"b").unwrap();
    let result = Creator::new(&payload).create(&NoProgress).unwrap();
    fs::write(&torrent, result.bytes()).unwrap();
    fs::remove_file(payload.join("a")).unwrap();
    fs::remove_file(payload.join("b")).unwrap();
    fs::write(payload.join("extra"), b"extra").unwrap();

    let assertion = btpc()
        .args([
            "verify",
            torrent.to_str().unwrap(),
            payload.to_str().unwrap(),
            "--json",
            "--fail-fast",
            "--extra-files",
        ])
        .assert()
        .code(6);
    let value: serde_json::Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
    assert_eq!(value["mismatches"].as_array().unwrap().len(), 1);
}

#[cfg(unix)]
#[test]
fn verify_reports_symlink_escape_without_following_it() {
    use std::os::unix::fs::symlink;

    let (temp, payload, torrent) = fixture(CreateMode::V2);
    let outside = temp.path().join("outside");
    fs::rename(&payload, &outside).unwrap();
    symlink(&outside, &payload).unwrap();
    btpc()
        .args([
            "verify",
            torrent.to_str().unwrap(),
            payload.to_str().unwrap(),
        ])
        .assert()
        .code(6)
        .stdout(predicate::str::contains("unsafe_path"));
}
