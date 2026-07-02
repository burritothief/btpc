use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn btpc(home: &TempDir) -> Command {
    let mut command = Command::cargo_bin("btpc").unwrap();
    command
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_DATA_HOME", home.path().join("data"))
        .env("BTPC_CONFIG", home.path().join("config/btpc/config.toml"));
    command
}

// Spec: CLI-GLOBAL-001
// Spec: CLI-COMPAT-001
// Spec: TEST-CLI-001
#[test]
#[allow(clippy::too_many_lines)]
fn clean_home_workflow_is_reproducible_redacted_and_compatible() {
    let home = TempDir::new().unwrap();
    let work = TempDir::new().unwrap();
    let payload = work.path().join("payload");
    fs::write(&payload, b"acceptance payload").unwrap();
    let configured = work.path().join("configured.torrent");
    let explicit = work.path().join("explicit.torrent");
    let config = home.path().join("config/btpc/config.toml");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(
        &config,
        r#"version = 1

[trackers.secret]
url = "https://user:password@tracker.example/announce?passkey=hidden"

[tracker_groups.release]
trackers = ["secret"]

[presets.base]
mode = "v1"
creation_date = 0
threads = 1

[presets.release]
extends = ["base"]
tracker_groups = ["release"]
private = true
source = "acceptance"
"#,
    )
    .unwrap();

    btpc(&home)
        .args([
            "create",
            payload.to_str().unwrap(),
            "--preset",
            "release",
            "--output",
        ])
        .arg(&configured)
        .assert()
        .success()
        .stderr(predicate::str::contains("password").not())
        .stderr(predicate::str::contains("hidden").not());
    btpc(&home)
        .args([
            "--no-config",
            "create",
            payload.to_str().unwrap(),
            "--mode",
            "v1",
            "--creation-date",
            "0",
            "--threads",
            "1",
            "--tracker",
            "https://user:password@tracker.example/announce?passkey=hidden",
            "--private",
            "--source",
            "acceptance",
            "--output",
        ])
        .arg(&explicit)
        .assert()
        .success();
    assert_eq!(fs::read(&configured).unwrap(), fs::read(&explicit).unwrap());

    btpc(&home)
        .args([
            "inspect",
            configured.to_str().unwrap(),
            "--field",
            "hash-v1",
            "--format",
            "plain",
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_match("^[0-9a-f]{40}\\n$").unwrap());
    btpc(&home)
        .args(["inspect", configured.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("password").not())
        .stdout(predicate::str::contains("hidden").not());

    let output_dir = work.path().join("multi");
    fs::create_dir(&output_dir).unwrap();
    let second = work.path().join("second");
    fs::write(&second, b"second").unwrap();
    btpc(&home)
        .args([
            "--no-config",
            "create",
            payload.to_str().unwrap(),
            second.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--creation-date",
            "0",
            "--threads",
            "1",
            "--print",
            "path",
            "--quiet",
        ])
        .assert()
        .success();

    let edited = work.path().join("edited.torrent");
    btpc(&home)
        .args(["edit", configured.to_str().unwrap(), "--output"])
        .arg(&edited)
        .args(["--comment", "accepted", "--quiet"])
        .assert()
        .success();
    btpc(&home)
        .args(["validate", edited.to_str().unwrap(), "--canonical"])
        .assert()
        .success();
    btpc(&home)
        .args([
            "verify",
            edited.to_str().unwrap(),
            payload.to_str().unwrap(),
            "--quiet",
        ])
        .assert()
        .success();

    let modern = btpc(&home)
        .args(["--no-config", "completion", "generate", "bash"])
        .output()
        .unwrap();
    let legacy = btpc(&home)
        .args(["--no-config", "--quiet", "completions", "bash"])
        .output()
        .unwrap();
    assert_eq!(modern.stdout, legacy.stdout);
    assert!(legacy.stderr.is_empty());
}
