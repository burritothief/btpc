use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

fn configured_command(temp: &TempDir) -> (Command, std::path::PathBuf) {
    let path = temp.path().join("config.toml");
    let mut command = btpc();
    command.args(["--config", path.to_str().unwrap()]);
    (command, path)
}

#[test]
fn path_init_refusal_force_check_and_permissions() {
    let temp = TempDir::new().unwrap();
    let (mut path_command, path) = configured_command(&temp);
    path_command
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(expected_plain_path(&path));

    let (mut init, _) = configured_command(&temp);
    init.args(["config", "init"])
        .assert()
        .success()
        .stdout("")
        .stderr(predicate::str::contains("initialized"));
    let original = fs::read_to_string(&path).unwrap();
    assert!(original.contains("version = 1"));

    let (mut refuse, _) = configured_command(&temp);
    refuse
        .args(["config", "init"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("already exists"));
    assert_eq!(fs::read_to_string(&path).unwrap(), original);

    fs::write(&path, "invalid").unwrap();
    let (mut force, _) = configured_command(&temp);
    force.args(["config", "init", "--force"]).assert().success();
    let (mut check, _) = configured_command(&temp);
    check
        .args(["config", "check"])
        .assert()
        .success()
        .stdout("valid\n");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[cfg(not(windows))]
fn expected_plain_path(path: &std::path::Path) -> Vec<u8> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt as _;
        let mut output = path.as_os_str().as_bytes().to_vec();
        output.push(b'\n');
        output
    }
    #[cfg(not(unix))]
    {
        format!("{}\n", path.display()).into_bytes()
    }
}

#[cfg(windows)]
fn expected_plain_path(path: &std::path::Path) -> Vec<u8> {
    use std::fmt::Write as _;
    use std::os::windows::ffi::OsStrExt as _;

    let mut output = "windows-utf16:".to_owned();
    for (index, unit) in path.as_os_str().encode_wide().enumerate() {
        if index > 0 {
            output.push(',');
        }
        write!(output, "{unit:04x}").unwrap();
    }
    output.push('\n');
    output.into_bytes()
}

#[test]
fn tracker_mutations_preserve_other_sections_and_redact_by_default() {
    let temp = TempDir::new().unwrap();
    let (_, path) = configured_command(&temp);
    fs::write(
        &path,
        "version = 1\n[create]\nmode = \"v2\"\n[presets.keep]\nprivate = true\n",
    )
    .unwrap();
    let secret = "https://user:password@tracker.example/announce/hidden?passkey=secret";
    let (mut add, _) = configured_command(&temp);
    add.args(["config", "tracker", "add", "private", secret])
        .assert()
        .success();
    let stored = fs::read_to_string(&path).unwrap();
    assert!(stored.contains("mode = \"v2\""));
    assert!(stored.contains("[presets.keep]"));
    assert!(stored.contains(secret));

    let (mut list, _) = configured_command(&temp);
    list.args(["config", "tracker", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("private\t<redacted-url>"))
        .stdout(predicate::str::contains("password").not());
    let (mut reveal, _) = configured_command(&temp);
    reveal
        .args(["config", "tracker", "list", "--show-secrets"])
        .assert()
        .success()
        .stdout(predicate::str::contains(secret));

    let (mut remove, _) = configured_command(&temp);
    remove
        .args(["config", "tracker", "remove", "private"])
        .assert()
        .success();
    let (mut missing, _) = configured_command(&temp);
    missing
        .args(["config", "tracker", "remove", "private"])
        .assert()
        .code(4);
}

#[test]
fn preset_save_show_list_remove_and_validation_rollback() {
    let temp = TempDir::new().unwrap();
    let (_, path) = configured_command(&temp);
    fs::write(&path, "version = 1\n[create]\nmode = \"v1\"\n").unwrap();
    let (mut save, _) = configured_command(&temp);
    save.args([
        "config",
        "preset",
        "save",
        "release",
        "--mode",
        "v2",
        "--private",
        "--extends",
        "base",
    ])
    .assert()
    .code(4);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "version = 1\n[create]\nmode = \"v1\"\n"
    );

    let (mut base, _) = configured_command(&temp);
    base.args(["config", "preset", "save", "base", "--mode", "v1"])
        .assert()
        .success();
    let (mut release, _) = configured_command(&temp);
    release
        .args([
            "config",
            "preset",
            "save",
            "release",
            "--mode",
            "v2",
            "--private",
            "--extends",
            "base",
        ])
        .assert()
        .success();
    let (mut list, _) = configured_command(&temp);
    list.args(["config", "preset", "list"])
        .assert()
        .success()
        .stdout("base\nrelease\n");
    let (mut show, _) = configured_command(&temp);
    show.args(["config", "preset", "show", "release"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode = \"v2\""));
    let (mut remove, _) = configured_command(&temp);
    remove
        .args(["config", "preset", "remove", "release"])
        .assert()
        .success();
}

#[test]
fn show_and_explain_are_redacted_and_explain_never_reads_payload() {
    let temp = TempDir::new().unwrap();
    let (_, path) = configured_command(&temp);
    fs::write(
        &path,
        r#"version = 1
[trackers.private]
url = "https://user:password@tracker.example/hidden?passkey=secret"
[presets.release]
tracker_aliases = ["private"]
trackers = ["https://user:password@direct.example/announce?passkey=direct-secret"]
web_seeds = ["https://user:password@seed.example/file?token=seed-secret"]
mode = "v2"
"#,
    )
    .unwrap();
    let missing_payload = temp.path().join("does-not-exist");
    let (mut show, _) = configured_command(&temp);
    show.args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<redacted-url>"))
        .stdout(predicate::str::contains("password").not())
        .stdout(predicate::str::contains("direct-secret").not())
        .stdout(predicate::str::contains("seed-secret").not());
    let (mut preset_show, _) = configured_command(&temp);
    preset_show
        .args(["config", "preset", "show", "release", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<redacted-url>"))
        .stdout(predicate::str::contains("direct-secret").not());
    let (mut preset_reveal, _) = configured_command(&temp);
    preset_reveal
        .args([
            "config",
            "preset",
            "show",
            "release",
            "--show-secrets",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("direct-secret"))
        .stdout(predicate::str::contains("seed-secret"));
    let (mut explain, _) = configured_command(&temp);
    explain
        .args([
            "config",
            "explain",
            "create",
            missing_payload.to_str().unwrap(),
            "--preset",
            "release",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode\tv2\tpreset:release"))
        .stdout(predicate::str::contains("password").not());
}

#[test]
fn json_lists_and_deterministic_mutations_are_stable() {
    let temp = TempDir::new().unwrap();
    let (_, path) = configured_command(&temp);
    fs::write(&path, "version = 1\n").unwrap();
    for (name, url) in [
        ("zeta", "https://z.example/announce?passkey=z-secret"),
        ("alpha", "https://a.example/announce?passkey=a-secret"),
    ] {
        let (mut add, _) = configured_command(&temp);
        add.args(["config", "tracker", "add", name, url])
            .assert()
            .success();
    }
    let first = fs::read(&path).unwrap();
    let (mut replace, _) = configured_command(&temp);
    replace
        .args([
            "config",
            "tracker",
            "add",
            "alpha",
            "https://a.example/announce?passkey=a-secret",
        ])
        .assert()
        .success();
    assert_eq!(fs::read(&path).unwrap(), first);
    assert!(
        String::from_utf8(first)
            .unwrap()
            .find("[trackers.alpha]")
            .unwrap()
            < fs::read_to_string(&path)
                .unwrap()
                .find("[trackers.zeta]")
                .unwrap()
    );

    let (mut list, _) = configured_command(&temp);
    list.args(["config", "tracker", "list", "--json"])
        .assert()
        .success()
        .stdout("{\"alpha\":\"<redacted-url>\",\"zeta\":\"<redacted-url>\"}\n");
    let (mut show, _) = configured_command(&temp);
    show.args(["config", "show", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("a-secret").not())
        .stdout(predicate::str::contains("z-secret").not());
}

#[cfg(unix)]
#[test]
fn check_rejects_group_or_world_accessible_configuration() {
    use std::os::unix::fs::PermissionsExt as _;

    let temp = TempDir::new().unwrap();
    let (_, path) = configured_command(&temp);
    fs::write(&path, "version = 1\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    let (mut check, _) = configured_command(&temp);
    check
        .args(["config", "check"])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("owner-only"));
}
