use std::fs;

use assert_cmd::Command;
use btpc_core::create::{CreateMode, CreateOptions, Creator, NoProgress, PieceLength};
use predicates::prelude::*;
use tempfile::TempDir;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

#[test]
fn help_and_version_are_available() {
    btpc()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("create"));
    btpc()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "btpc {}",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn create_infers_output_and_keeps_stdout_clean_by_default() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"data").unwrap();

    btpc()
        .args(["create", payload.to_str().unwrap(), "--quiet"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    assert!(temp.path().join("payload.torrent").exists());
}

#[test]
fn json_output_is_versioned_and_only_uses_stdout() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("out.torrent");
    fs::write(&payload, b"data").unwrap();

    let assertion = btpc()
        .args([
            "create",
            payload.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .stderr("");
    let value: serde_json::Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
    assert_eq!(value["schema"], "btpc.create.v2");
    assert_eq!(value["mode"], "v1");
    assert_eq!(value["output"]["schema"], "btpc.filesystem-path.v2");
    assert_eq!(
        value["output"]["display"],
        output.to_string_lossy().as_ref()
    );
    assert_eq!(value["output_display"], output.to_string_lossy().as_ref());
    #[cfg(unix)]
    {
        use std::fmt::Write as _;
        use std::os::unix::ffi::OsStrExt as _;
        let expected =
            output
                .as_os_str()
                .as_bytes()
                .iter()
                .fold(String::new(), |mut encoded, byte| {
                    write!(encoded, "{byte:02x}").unwrap();
                    encoded
                });
        assert_eq!(value["output"]["encoding"], "unix-bytes-hex");
        assert_eq!(value["output"]["value"], expected);
    }
    assert_eq!(value["file_count"], 1);
    assert!(value["info_hash_v1"].as_str().unwrap().len() == 40);
}

#[test]
fn overwrite_force_invalid_options_and_exit_codes_are_stable() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("out.torrent");
    fs::write(&payload, b"data").unwrap();
    fs::write(&output, b"existing").unwrap();

    btpc()
        .args([
            "create",
            payload.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .code(3)
        .stdout("")
        .stderr(predicate::str::contains("already exists"));
    btpc()
        .args([
            "create",
            payload.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--force",
            "--durable",
            "--quiet",
        ])
        .assert()
        .success();
    btpc()
        .args(["create", payload.to_str().unwrap(), "--piece-length", "3"])
        .assert()
        .code(2);
    btpc()
        .args(["create", payload.to_str().unwrap(), "--threads", "invalid"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn no_color_and_non_tty_diagnostics_have_no_ansi() {
    btpc()
        .env("NO_COLOR", "1")
        .args(["create", "/definitely/missing"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("\u{1b}[").not());
}

#[test]
fn cli_bytes_match_direct_core_creation() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("cli.torrent");
    fs::write(&payload, b"core parity").unwrap();

    btpc()
        .args([
            "create",
            payload.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--quiet",
        ])
        .assert()
        .success();
    let direct = Creator::new(&payload).create(&NoProgress).unwrap();
    assert_eq!(fs::read(output).unwrap(), direct.bytes());
}

#[test]
fn explicit_cli_threads_match_the_sequential_oracle() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, vec![0x6d; 4 * 1024 * 1024 + 17]).unwrap();
    let sequential = temp.path().join("sequential.torrent");
    let parallel = temp.path().join("parallel.torrent");
    for (threads, output) in [("1", &sequential), ("4", &parallel)] {
        btpc()
            .args([
                "create",
                payload.to_str().unwrap(),
                "--piece-length",
                "262144",
                "--threads",
                threads,
                "--output",
                output.to_str().unwrap(),
                "--quiet",
            ])
            .assert()
            .success();
    }
    assert_eq!(fs::read(sequential).unwrap(), fs::read(parallel).unwrap());
}

#[test]
fn every_mode_matches_core_and_reports_applicable_hashes() {
    for (name, mode) in [
        ("v1", CreateMode::V1),
        ("v2", CreateMode::V2),
        ("hybrid", CreateMode::Hybrid),
    ] {
        let temp = TempDir::new().unwrap();
        let payload = temp.path().join("payload");
        let output = temp.path().join("cli.torrent");
        fs::write(&payload, b"mode parity").unwrap();
        let assertion = btpc()
            .args([
                "create",
                payload.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
                "--mode",
                name,
                "--piece-length",
                "16384",
                "--json",
            ])
            .assert()
            .success();
        let value: serde_json::Value =
            serde_json::from_slice(&assertion.get_output().stdout).unwrap();
        assert_eq!(value["mode"], name);
        assert_eq!(value["info_hash_v1"].is_string(), mode != CreateMode::V2);
        assert_eq!(value["info_hash_v2"].is_string(), mode != CreateMode::V1);
        let options = CreateOptions::builder()
            .mode(mode)
            .piece_length(PieceLength::Exact(16_384))
            .build()
            .unwrap();
        let direct = Creator::new(&payload)
            .options(options)
            .create(&NoProgress)
            .unwrap();
        assert_eq!(fs::read(&output).unwrap(), direct.bytes());
    }
}

#[test]
fn automatic_piece_policy_matches_core_for_every_mode() {
    for (name, mode) in [
        ("v1", CreateMode::V1),
        ("v2", CreateMode::V2),
        ("hybrid", CreateMode::Hybrid),
    ] {
        let temp = TempDir::new().unwrap();
        let payload = temp.path().join("payload");
        let output = temp.path().join("cli.torrent");
        fs::write(&payload, b"automatic parity").unwrap();
        btpc()
            .args([
                "create",
                payload.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
                "--mode",
                name,
                "--quiet",
            ])
            .assert()
            .success();
        let options = CreateOptions::builder().mode(mode).build().unwrap();
        let direct = Creator::new(&payload)
            .options(options)
            .create(&NoProgress)
            .unwrap();
        assert_eq!(fs::read(output).unwrap(), direct.bytes());
    }
}

#[test]
fn v2_modes_reject_v1_only_piece_lengths() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"data").unwrap();
    for mode in ["v2", "hybrid"] {
        btpc()
            .args([
                "create",
                payload.to_str().unwrap(),
                "--mode",
                mode,
                "--piece-length",
                "8192",
            ])
            .assert()
            .code(4)
            .stderr(predicate::str::contains("between 16384"));
    }
}

#[test]
fn ergonomic_piece_lengths_target_policy_and_print_order_work() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("out.torrent");
    fs::write(&payload, vec![7_u8; 100_000]).unwrap();
    let assertion = btpc()
        .args([
            "create",
            payload.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--piece-length",
            "2^14",
            "--print",
            "path",
            "--print",
            "info-hash-v1",
        ])
        .assert()
        .success();
    let lines = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let path_line = lines.lines().next().unwrap();
    #[cfg(not(windows))]
    assert!(path_line.ends_with("out.torrent"));
    #[cfg(windows)]
    {
        let units = path_line
            .strip_prefix("windows-utf16:")
            .unwrap()
            .split(',')
            .map(|unit| u16::from_str_radix(unit, 16).unwrap())
            .collect::<Vec<_>>();
        assert!(String::from_utf16(&units).unwrap().ends_with("out.torrent"));
    }
    assert_eq!(lines.lines().nth(1).unwrap().len(), 40);
    fs::remove_file(&output).unwrap();
    btpc()
        .args([
            "create",
            payload.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--target-pieces",
            "4",
            "--max-piece-length",
            "64KiB",
            "--quiet",
        ])
        .assert()
        .success();
    assert!(
        btpc_core::Metainfo::from_path(output)
            .unwrap()
            .piece_count()
            <= 4
    );
}

#[test]
fn dry_run_aliases_public_dates_and_entropy_are_explicit() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("out.torrent");
    let config = temp.path().join("config.toml");
    fs::write(&payload, b"planning payload").unwrap();
    fs::write(
        &config,
        "version = 1\n[trackers.private]\nurl = \"https://tracker.example/announce\"\n",
    )
    .unwrap();
    btpc()
        .args([
            "--config",
            config.to_str().unwrap(),
            "create",
            payload.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--tracker-alias",
            "private",
            "--public",
            "--creation-date",
            "2026-01-01T00:00:00+00:00",
            "--entropy",
            "00ff",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("plan:"));
    assert!(!output.exists());
    btpc()
        .args([
            "--config",
            config.to_str().unwrap(),
            "create",
            payload.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--tracker-alias",
            "private",
            "--public",
            "--creation-date",
            "none",
            "--entropy",
            "00ff",
            "--quiet",
        ])
        .assert()
        .success();
    let torrent = btpc_core::Metainfo::from_path(output).unwrap();
    assert_eq!(torrent.private(), Some(false));
    assert_eq!(torrent.trackers().len(), 1);
}

#[test]
fn creation_conflicts_and_size_errors_are_usage_failures() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"data").unwrap();
    btpc()
        .args(["create", payload.to_str().unwrap(), "--private", "--public"])
        .assert()
        .code(2);
    btpc()
        .args([
            "create",
            payload.to_str().unwrap(),
            "--piece-length",
            "3MiB",
        ])
        .assert()
        .code(2);
    btpc()
        .args([
            "create",
            payload.to_str().unwrap(),
            "--json",
            "--print",
            "path",
        ])
        .assert()
        .code(2);
}

#[test]
fn creator_identity_defaults_overrides_and_omits() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"creator").unwrap();
    for (name, extra, expected) in [
        ("default", vec![], "btpc/0.1.0\n"),
        ("override", vec!["--created-by", "custom/π"], "custom/π\n"),
        ("omit", vec!["--no-created-by"], "\n"),
    ] {
        let torrent = temp.path().join(format!("{name}.torrent"));
        let mut command = btpc();
        command
            .args(["create", payload.to_str().unwrap(), "--output"])
            .arg(&torrent)
            .args(["--quiet"])
            .args(extra)
            .assert()
            .success();
        btpc()
            .args([
                "inspect",
                torrent.to_str().unwrap(),
                "--field",
                "creator",
                "--format",
                "plain",
            ])
            .assert()
            .success()
            .stdout(expected);
    }
    btpc()
        .args([
            "create",
            payload.to_str().unwrap(),
            "--created-by",
            "x",
            "--no-created-by",
        ])
        .assert()
        .code(2);
}
