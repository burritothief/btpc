use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

#[test]
fn diagnostics_are_structured_contextual_and_styling_obeys_policy() {
    btpc()
        .args(["--color", "never", "create", "/definitely/missing"])
        .assert()
        .code(3)
        .stdout("")
        .stderr(predicate::str::contains("error [io]").and(predicate::str::contains("path:")))
        .stderr(predicate::str::contains("hint:"))
        .stderr(predicate::str::contains("\u{1b}[").not());
    btpc()
        .args(["--color", "always", "create", "/definitely/missing"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("\u{1b}["));
}

#[test]
fn bencode_diagnostics_include_byte_offset_and_remediation() {
    let temp = TempDir::new().unwrap();
    let torrent = temp.path().join("bad.torrent");
    fs::write(&torrent, b"not bencode").unwrap();
    btpc()
        .args(["validate", torrent.to_str().unwrap()])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("byte offset:"))
        .stderr(predicate::str::contains("hint:"));
}

#[test]
fn close_names_receive_suggestions_but_unrelated_names_do_not() {
    let temp = TempDir::new().unwrap();
    let config = temp.path().join("config.toml");
    fs::write(&config, "version = 1\n[presets.release]\nmode = \"v2\"\n").unwrap();
    btpc()
        .args([
            "--config",
            config.to_str().unwrap(),
            "config",
            "preset",
            "show",
            "relese",
        ])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("did you mean \"release\""));
    btpc()
        .args([
            "--config",
            config.to_str().unwrap(),
            "config",
            "preset",
            "show",
            "unrelated",
        ])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("did you mean").not());
}

#[test]
fn clap_suggests_commands_fields_shells_and_enum_values() {
    for (arguments, suggestion) in [
        (vec!["crate"], "create"),
        (vec!["completions", "bahs"], "bash"),
        (vec!["create", "/tmp/missing", "--mode", "hybrd"], "hybrid"),
        (
            vec!["inspect", "/tmp/missing", "--max-input-byte"],
            "--max-input-bytes",
        ),
    ] {
        btpc()
            .args(arguments)
            .assert()
            .code(2)
            .stderr(predicate::str::contains(suggestion));
    }
}

#[test]
fn create_pretty_and_verbose_expand_only_human_stderr() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"rendering payload").unwrap();
    btpc()
        .args(["-v", "create", payload.to_str().unwrap(), "--quiet"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    fs::remove_file(temp.path().join("payload.torrent")).unwrap();
    btpc()
        .args(["-v", "create", payload.to_str().unwrap(), "--pretty"])
        .assert()
        .success()
        .stdout("")
        .stderr(predicate::str::contains("✓ created"))
        .stderr(predicate::str::contains("timing: scan="));
}
