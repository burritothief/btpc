use std::fs;

use assert_cmd::Command;
use btpc_core::Metainfo;
use predicates::prelude::*;
use tempfile::TempDir;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

fn write_config(temp: &TempDir, contents: &str) -> std::path::PathBuf {
    let path = temp.path().join("config.toml");
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn explicit_config_presets_and_cli_merge_in_order() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("out.torrent");
    fs::write(&payload, b"configured payload").unwrap();
    let config = write_config(
        &temp,
        r#"
version = 1

[create]
mode = "v2"
web_seeds = ["https://seed.example/base"]
includes = ["*.bin"]

[trackers.public]
url = "https://tracker.example/announce"

[trackers.secret]
url = "https://user:password@tracker.example/announce/secret?passkey=hidden"

[tracker_groups.all]
trackers = ["public", "secret"]

[presets.base]
tracker_groups = ["all"]
web_seeds = ["https://seed.example/preset"]

[presets.private]
extends = ["base"]
private = true
source = "configured"
"#,
    );

    btpc()
        .args([
            "--config",
            config.to_str().unwrap(),
            "create",
            payload.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--preset",
            "private",
            "--mode",
            "hybrid",
            "--source",
            "cli",
            "--include",
            "payload",
            "--quiet",
        ])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let torrent = Metainfo::from_path(&output).unwrap();
    assert_eq!(torrent.mode(), btpc_core::TorrentMode::Hybrid);
    assert_eq!(torrent.private(), Some(true));
    assert!(
        torrent
            .to_bytes()
            .unwrap()
            .windows(b"6:source3:cli".len())
            .any(|window| window == b"6:source3:cli")
    );
    assert_eq!(torrent.trackers().len(), 1);
    assert_eq!(torrent.trackers()[0].len(), 2);
    assert_eq!(torrent.web_seeds().len(), 2);
}

#[test]
fn clear_operations_reset_inherited_lists_before_cli_additions() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("out.torrent");
    fs::write(&payload, b"configured payload").unwrap();
    let config = write_config(
        &temp,
        r#"
version = 1
[create]
trackers = ["https://tracker.example/config"]
web_seeds = ["https://seed.example/config"]
includes = ["*"]
excludes = ["*.tmp"]
"#,
    );
    btpc()
        .args([
            "--config",
            config.to_str().unwrap(),
            "create",
            payload.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--clear-trackers",
            "--tracker",
            "https://tracker.example/cli",
            "--clear-web-seeds",
            "--web-seed",
            "https://seed.example/cli",
            "--clear-includes",
            "--include",
            "payload",
            "--clear-excludes",
            "--quiet",
        ])
        .assert()
        .success();
    let torrent = Metainfo::from_path(&output).unwrap();
    assert_eq!(
        torrent.trackers(),
        &[vec![b"https://tracker.example/cli".to_vec()]]
    );
    assert_eq!(torrent.web_seeds(), &[b"https://seed.example/cli".to_vec()]);
}

#[test]
fn no_config_ignores_environment_and_preserves_default_bytes() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let baseline = temp.path().join("baseline.torrent");
    let ignored = temp.path().join("ignored.torrent");
    fs::write(&payload, b"deterministic payload").unwrap();
    let config = write_config(&temp, "version = 1\n[create]\nmode = \"v2\"\n");

    btpc()
        .args([
            "create",
            payload.to_str().unwrap(),
            "--output",
            baseline.to_str().unwrap(),
            "--quiet",
        ])
        .assert()
        .success();
    btpc()
        .env("BTPC_CONFIG", &config)
        .args([
            "--no-config",
            "create",
            payload.to_str().unwrap(),
            "--output",
            ignored.to_str().unwrap(),
            "--quiet",
        ])
        .assert()
        .success();
    assert_eq!(fs::read(baseline).unwrap(), fs::read(ignored).unwrap());
}

#[test]
fn explicit_config_precedes_environment_and_absent_implicit_config_is_allowed() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("out.torrent");
    let env_config = temp.path().join("env.toml");
    let explicit_config = temp.path().join("explicit.toml");
    fs::write(&payload, b"config precedence payload").unwrap();
    fs::write(&env_config, "version = 1\n[create]\nmode = \"v2\"\n").unwrap();
    fs::write(
        &explicit_config,
        "version = 1\n[create]\nmode = \"hybrid\"\n",
    )
    .unwrap();
    btpc()
        .env("BTPC_CONFIG", &env_config)
        .args([
            "--config",
            explicit_config.to_str().unwrap(),
            "create",
            payload.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--quiet",
        ])
        .assert()
        .success();
    assert_eq!(
        Metainfo::from_path(&output).unwrap().mode(),
        btpc_core::TorrentMode::Hybrid
    );

    let isolated_home = temp.path().join("isolated-home");
    fs::create_dir(&isolated_home).unwrap();
    btpc()
        .env_remove("BTPC_CONFIG")
        .env("HOME", &isolated_home)
        .env("XDG_CONFIG_HOME", isolated_home.join("xdg"))
        .args(["manpage"])
        .assert()
        .success();
}

#[test]
fn environment_config_is_used_but_current_directory_config_is_not() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let env_output = temp.path().join("env.torrent");
    let local_output = temp.path().join("local.torrent");
    let env_config = temp.path().join("env.toml");
    fs::write(&payload, b"environment config payload").unwrap();
    fs::write(&env_config, "version = 1\n[create]\nmode = \"v2\"\n").unwrap();
    btpc()
        .env("BTPC_CONFIG", &env_config)
        .args([
            "create",
            payload.to_str().unwrap(),
            "--output",
            env_output.to_str().unwrap(),
            "--quiet",
        ])
        .assert()
        .success();
    assert_eq!(
        Metainfo::from_path(&env_output).unwrap().mode(),
        btpc_core::TorrentMode::V2
    );

    let working = temp.path().join("working");
    let isolated_home = temp.path().join("home");
    fs::create_dir(&working).unwrap();
    fs::create_dir(&isolated_home).unwrap();
    fs::write(
        working.join("config.toml"),
        "version = 1\n[create]\nmode = \"v2\"\n",
    )
    .unwrap();
    btpc()
        .current_dir(&working)
        .env_remove("BTPC_CONFIG")
        .env("HOME", &isolated_home)
        .env("XDG_CONFIG_HOME", isolated_home.join("xdg"))
        .args([
            "create",
            payload.to_str().unwrap(),
            "--output",
            local_output.to_str().unwrap(),
            "--quiet",
        ])
        .assert()
        .success();
    assert_eq!(
        Metainfo::from_path(&local_output).unwrap().mode(),
        btpc_core::TorrentMode::V1
    );
}

#[test]
fn malformed_unknown_version_and_cycles_are_rejected_without_secret_leaks() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"payload").unwrap();
    for contents in [
        "version = 1\n[create\nmode = \"v2\"\n",
        "version = 1\nunknown = true\n",
        "version = 2\n",
        "version = 1\n[presets.a]\nextends = [\"b\"]\n[presets.b]\nextends = [\"a\"]\n",
        "version = 1\n[trackers.secret]\nurl = \"https://user:password@example.com/hidden?passkey=secret\"\n[presets.a]\ntracker_aliases = [\"missing\"]\n",
    ] {
        let config = write_config(&temp, contents);
        btpc()
            .args([
                "--config",
                config.to_str().unwrap(),
                "create",
                payload.to_str().unwrap(),
                "--preset",
                "a",
                "--quiet",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("password").not())
            .stderr(predicate::str::contains("passkey=secret").not());
    }
}

#[test]
fn multi_preset_order_and_resolved_conflicts_are_deterministic() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("out.torrent");
    fs::write(&payload, b"preset order payload").unwrap();
    let config = write_config(
        &temp,
        r#"
version = 1
[presets.first]
mode = "v2"
[presets.second]
mode = "hybrid"
"#,
    );
    btpc()
        .args([
            "--config",
            config.to_str().unwrap(),
            "create",
            payload.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--preset",
            "first",
            "--preset",
            "second",
            "--quiet",
        ])
        .assert()
        .success();
    assert_eq!(
        Metainfo::from_path(&output).unwrap().mode(),
        btpc_core::TorrentMode::Hybrid
    );

    let invalid = write_config(
        &temp,
        "version = 1\n[create]\nmode = \"v2\"\npiece_length = 8192\n",
    );
    btpc()
        .args([
            "--config",
            invalid.to_str().unwrap(),
            "create",
            payload.to_str().unwrap(),
            "--quiet",
        ])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("between 16384"));
}

#[test]
fn cycle_and_missing_reference_errors_include_resolution_chains() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"payload").unwrap();
    let cycle = write_config(
        &temp,
        "version = 1\n[presets.a]\nextends = [\"b\"]\n[presets.b]\nextends = [\"a\"]\n",
    );
    btpc()
        .args([
            "--config",
            cycle.to_str().unwrap(),
            "create",
            payload.to_str().unwrap(),
            "--preset",
            "a",
        ])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("a -> b -> a"));

    let missing = write_config(&temp, "version = 1\n[presets.a]\nextends = [\"b\"]\n");
    btpc()
        .args([
            "--config",
            missing.to_str().unwrap(),
            "create",
            payload.to_str().unwrap(),
            "--preset",
            "a",
        ])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("a -> b"));
}
