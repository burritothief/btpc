use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

#[test]
fn multiple_inputs_use_output_dir_and_report_in_input_order() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("out");
    fs::create_dir(&output).unwrap();
    let a = temp.path().join("a");
    let b = temp.path().join("b");
    fs::write(&a, b"a").unwrap();
    fs::write(&b, b"b").unwrap();
    let assertion = btpc()
        .args([
            "create",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--output-dir",
            output.to_str().unwrap(),
            "--print",
            "path",
            "--quiet",
        ])
        .assert()
        .success();
    let lines = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(lines.lines().next().unwrap().ends_with("a.torrent"));
    assert!(lines.lines().nth(1).unwrap().ends_with("b.torrent"));
    assert!(output.join("a.torrent").exists());
    assert!(output.join("b.torrent").exists());
}

#[test]
fn batch_schema_dry_run_and_cli_overrides_are_deterministic() {
    let temp = TempDir::new().unwrap();
    let a = temp.path().join("a");
    let b = temp.path().join("b");
    let oa = temp.path().join("a.torrent");
    let ob = temp.path().join("b.torrent");
    fs::write(&a, b"a").unwrap();
    fs::write(&b, b"b").unwrap();
    let batch = temp.path().join("jobs.toml");
    fs::write(&batch, format!(
        "version = 1\n[[jobs]]\ninput = {a:?}\noutput = {oa:?}\nmode = \"v2\"\n[[jobs]]\ninput = {b:?}\noutput = {ob:?}\n"
    )).unwrap();
    btpc()
        .args([
            "create",
            "--batch",
            batch.to_str().unwrap(),
            "--mode",
            "v1",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("plan:"));
    assert!(!oa.exists());
    assert!(!ob.exists());
    btpc()
        .args([
            "create",
            "--batch",
            batch.to_str().unwrap(),
            "--mode",
            "v1",
            "--quiet",
        ])
        .assert()
        .success();
    assert_eq!(
        btpc_core::Metainfo::from_path(oa).unwrap().mode(),
        btpc_core::TorrentMode::V1
    );
    assert_eq!(
        btpc_core::Metainfo::from_path(ob).unwrap().mode(),
        btpc_core::TorrentMode::V1
    );
}

#[test]
fn batch_preflight_rejects_collisions_and_invalid_shapes_before_writes() {
    let temp = TempDir::new().unwrap();
    let a = temp.path().join("a");
    let b = temp.path().join("b");
    fs::write(&a, b"a").unwrap();
    fs::write(&b, b"b").unwrap();
    btpc()
        .args([
            "create",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--output",
            temp.path().join("one.torrent").to_str().unwrap(),
        ])
        .assert()
        .code(4);
    btpc()
        .args(["create", a.to_str().unwrap(), "--jobs", "0"])
        .assert()
        .code(4);
    let batch = temp.path().join("bad.toml");
    fs::write(&batch, "version = 1\nunknown = true\n").unwrap();
    btpc()
        .args(["create", "--batch", batch.to_str().unwrap()])
        .assert()
        .code(4);
}
