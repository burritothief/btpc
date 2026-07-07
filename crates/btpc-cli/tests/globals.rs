use std::fs;

use assert_cmd::Command;
use btpc_core::create::{Creator, NoProgress};
use predicates::prelude::*;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

fn torrent_fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    let torrent = temp.path().join("payload.torrent");
    fs::write(&payload, b"global option payload").unwrap();
    let result = Creator::new(&payload).create(&NoProgress).unwrap();
    fs::write(&torrent, result.bytes()).unwrap();
    (temp, payload, torrent)
}

#[test]
fn global_options_work_before_and_after_subcommands() {
    let (_temp, _payload, torrent) = torrent_fixture();
    let before = btpc()
        .args([
            "--no-config",
            "--color",
            "never",
            "inspect",
            torrent.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    let after = btpc()
        .args([
            "inspect",
            torrent.to_str().unwrap(),
            "--json",
            "--color",
            "never",
            "--no-config",
        ])
        .output()
        .unwrap();
    assert!(before.status.success());
    assert_eq!(before.stdout, after.stdout);
    assert_eq!(before.stderr, after.stderr);
}

#[test]
fn incompatible_global_controls_are_usage_errors() {
    let (_temp, _payload, torrent) = torrent_fixture();
    btpc()
        .args(["--quiet", "--verbose", "inspect", torrent.to_str().unwrap()])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("cannot be used with"));
    btpc()
        .args(["--config", "custom.toml", "--no-config", "manpage"])
        .assert()
        .code(2);
    btpc()
        .args(["--quiet", "inspect", torrent.to_str().unwrap(), "--pretty"])
        .assert()
        .code(2);
}

#[test]
fn quiet_suppresses_human_summaries_but_not_machine_values() {
    let (_temp, payload, torrent) = torrent_fixture();
    btpc()
        .args(["--quiet", "inspect", torrent.to_str().unwrap()])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    btpc()
        .args(["inspect", torrent.to_str().unwrap(), "--json", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\":\"btpc.inspect.v1\""))
        .stderr("");
    btpc()
        .args(["--quiet", "magnet", torrent.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("magnet:?"))
        .stderr("");
    btpc()
        .args([
            "--quiet",
            "verify",
            torrent.to_str().unwrap(),
            payload.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn pretty_is_human_only_and_does_not_change_current_summary() {
    let (_temp, _payload, torrent) = torrent_fixture();
    let normal = btpc()
        .args(["inspect", torrent.to_str().unwrap()])
        .output()
        .unwrap();
    let pretty = btpc()
        .args(["inspect", torrent.to_str().unwrap(), "--pretty"])
        .output()
        .unwrap();
    assert!(normal.status.success());
    assert_ne!(normal.stdout, pretty.stdout);
    let pretty_text = String::from_utf8(pretty.stdout.clone()).unwrap();
    assert!(pretty_text.contains("Torrent info:"));
    assert!(pretty_text.contains("Details:"));
    assert!(pretty_text.contains("Canonical: yes"));
    assert_eq!(normal.stderr, pretty.stderr);
    btpc()
        .args(["inspect", torrent.to_str().unwrap(), "--pretty", "--json"])
        .assert()
        .code(2);
}

#[test]
fn piped_output_has_no_ansi_for_all_color_inputs() {
    let (_temp, _payload, torrent) = torrent_fixture();
    for color in ["auto", "never", "always"] {
        btpc()
            .env("NO_COLOR", "1")
            .args(["--color", color, "inspect", torrent.to_str().unwrap()])
            .assert()
            .success()
            .stdout(predicate::str::contains("\u{1b}[").not())
            .stderr(predicate::str::contains("\u{1b}[").not());
    }
}

#[test]
fn existing_json_aliases_keep_schemas_and_exit_codes() {
    let (_temp, payload, torrent) = torrent_fixture();
    for (command, schema) in [
        ("inspect", "btpc.inspect.v1"),
        ("validate", "btpc.validate.v1"),
    ] {
        let output = btpc()
            .args([command, torrent.to_str().unwrap(), "--json"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["schema"], schema);
        assert!(output.stderr.is_empty());
    }
    fs::write(&payload, b"global option payloae").unwrap();
    let output = btpc()
        .args([
            "verify",
            torrent.to_str().unwrap(),
            payload.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(6));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema"], "btpc.verify.v2");
    assert!(output.stderr.is_empty());
}
