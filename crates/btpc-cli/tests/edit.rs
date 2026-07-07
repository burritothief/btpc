use std::fs;

use assert_cmd::Command;
use btpc_core::create::{CreateMode, CreateOptions, Creator, NoProgress};
use btpc_core::metainfo::RawMetainfo;
use predicates::prelude::*;
use tempfile::TempDir;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

fn fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let torrent = temp.path().join("payload.torrent");
    fs::write(&payload, b"edit payload").unwrap();
    Creator::new(&payload)
        .create_to_path(
            &torrent,
            btpc_core::create::OverwritePolicy::Deny,
            &NoProgress,
        )
        .unwrap();
    fs::remove_file(payload).unwrap();
    (temp, torrent)
}

#[test]
fn copy_by_default_and_dry_run_never_read_payload() {
    let (temp, input) = fixture();
    let expected = temp.path().join("payload.edited.torrent");
    btpc()
        .args([
            "edit",
            input.to_str().unwrap(),
            "--comment",
            "updated",
            "--dry-run",
            "--diff",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("would write"))
        .stdout(predicate::str::contains("unchanged"));
    assert!(!expected.exists());
    btpc()
        .args(["edit", input.to_str().unwrap(), "--comment", "updated"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote"));
    assert!(input.exists());
    assert!(expected.exists());
    btpc_core::Metainfo::from_path(expected).unwrap();
}

#[test]
fn collisions_force_and_atomic_in_place_are_safe() {
    let (temp, input) = fixture();
    let output = temp.path().join("custom.torrent");
    fs::write(&output, b"existing").unwrap();
    btpc()
        .args([
            "edit",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--comment",
            "x",
        ])
        .assert()
        .code(3);
    assert_eq!(fs::read(&output).unwrap(), b"existing");
    btpc()
        .args([
            "edit",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--comment",
            "x",
            "--force",
            "--durable",
        ])
        .assert()
        .success();
    btpc_core::Metainfo::from_path(&output).unwrap();
    btpc()
        .args([
            "edit",
            input.to_str().unwrap(),
            "--in-place",
            "--source",
            "private-source",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("changed"));
    btpc_core::Metainfo::from_path(input).unwrap();
}

#[test]
fn typed_set_and_clear_operations_validate_and_report_hash_domains() {
    let (temp, input) = fixture();
    let output = temp.path().join("edited.torrent");
    btpc()
        .args([
            "-v",
            "edit",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--tracker",
            "https://tracker.example/announce",
            "--web-seed",
            "https://seed.example/file",
            "--node",
            "router.example:6881",
            "--comment",
            "comment",
            "--created-by",
            "btpc",
            "--creation-date",
            "0",
            "--private",
            "--source",
            "source",
            "--diff",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("v1:"))
        .stdout(predicate::str::contains("changed"));
    btpc_core::Metainfo::from_path(&output).unwrap();
    btpc()
        .args([
            "edit",
            output.to_str().unwrap(),
            "--in-place",
            "--clear-trackers",
            "--clear-web-seeds",
            "--clear-nodes",
            "--clear-comment",
            "--clear-created-by",
            "--clear-creation-date",
            "--clear-private",
            "--clear-source",
        ])
        .assert()
        .success();
    btpc_core::Metainfo::from_path(output).unwrap();
}

#[test]
fn invalid_file_attribute_edit_rolls_back() {
    let (_temp, input) = fixture();
    let original = fs::read(&input).unwrap();
    btpc()
        .args([
            "edit",
            input.to_str().unwrap(),
            "--in-place",
            "--file-attributes",
            "missing=p",
        ])
        .assert()
        .code(4);
    assert_eq!(fs::read(input).unwrap(), original);
}

#[test]
fn json_summary_and_config_tracker_aliases_are_supported() {
    let (temp, input) = fixture();
    let config = temp.path().join("config.toml");
    let output = temp.path().join("alias.torrent");
    fs::write(
        &config,
        "version = 1\n[trackers.private]\nurl = \"https://tracker.example/announce\"\n[tracker_groups.release]\ntrackers = [\"private\"]\n",
    )
    .unwrap();
    let assertion = btpc()
        .args([
            "--config",
            config.to_str().unwrap(),
            "edit",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--tracker-group",
            "release",
            "--json",
        ])
        .assert()
        .success()
        .stderr("");
    let value: serde_json::Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
    assert_eq!(value["schema"], "btpc.edit.v2");
    assert_eq!(value["output"]["schema"], "btpc.filesystem-path.v2");
    assert_eq!(value["output_display"], output.to_string_lossy().as_ref());
    assert_eq!(value["info_hash_v1_changed"], false);
    assert_eq!(
        btpc_core::Metainfo::from_path(output)
            .unwrap()
            .trackers()
            .len(),
        1
    );
}

#[test]
fn top_level_cli_edit_preserves_noncanonical_info_bytes() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("input.torrent");
    let output = temp.path().join("output.torrent");
    let bytes = b"d4:infod6:pieces0:12:piece lengthi16384e4:name7:payload6:lengthi0eee";
    fs::write(&input, bytes).unwrap();
    let original = btpc_core::Metainfo::from_bytes(bytes).unwrap();
    let original_info = RawMetainfo::from_bytes(bytes)
        .unwrap()
        .info_bytes()
        .to_vec();

    btpc()
        .args([
            "edit",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--comment",
            "updated",
        ])
        .assert()
        .success();

    let edited = btpc_core::Metainfo::from_path(&output).unwrap();
    assert_eq!(edited.info_hash_v1(), original.info_hash_v1());
    assert_eq!(
        RawMetainfo::from_bytes(edited.original_bytes())
            .unwrap()
            .info_bytes(),
        original_info
    );
}

#[test]
fn hybrid_cli_file_attributes_update_both_representations() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let input = temp.path().join("input.torrent");
    let output = temp.path().join("output.torrent");
    fs::write(&payload, b"data").unwrap();
    let options = CreateOptions::builder()
        .mode(CreateMode::Hybrid)
        .build()
        .unwrap();
    fs::write(
        &input,
        Creator::new(&payload)
            .options(options)
            .create(&NoProgress)
            .unwrap()
            .bytes(),
    )
    .unwrap();

    btpc()
        .args([
            "edit",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--file-attributes",
            "payload=x",
        ])
        .assert()
        .success();

    let bytes = fs::read(output).unwrap();
    assert_eq!(
        bytes
            .windows(b"4:attr1:x".len())
            .filter(|window| *window == b"4:attr1:x")
            .count(),
        2
    );
}
