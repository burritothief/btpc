use std::fs;
use std::path::Path;

use assert_cmd::Command;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

fn normalized_help(output: &[u8]) -> String {
    String::from_utf8_lossy(output)
        .replace("\r\n", "\n")
        .replace("btpc.exe", "btpc")
}

// Spec: CLI-DOC-001
#[test]
fn completions_and_manpage_are_generated_from_the_cli_definition() {
    btpc()
        .args(["completion", "generate", "bash"])
        .assert()
        .success()
        .stdout(predicates::str::contains("_btpc"));
    btpc()
        .arg("manpage")
        .assert()
        .success()
        .stdout(predicates::str::contains(".TH btpc 1"));
}

#[test]
fn checked_in_help_reference_matches_the_binary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/reference");
    for (name, arguments) in [
        ("btpc.txt", vec!["--help"]),
        ("btpc-create.txt", vec!["create", "--help"]),
        ("btpc-inspect.txt", vec!["inspect", "--help"]),
        ("btpc-validate.txt", vec!["validate", "--help"]),
        ("btpc-verify.txt", vec!["verify", "--help"]),
        ("btpc-edit.txt", vec!["edit", "--help"]),
        ("btpc-magnet.txt", vec!["magnet", "--help"]),
        ("btpc-completion.txt", vec!["completion", "--help"]),
        (
            "btpc-completion-generate.txt",
            vec!["completion", "generate", "--help"],
        ),
        (
            "btpc-completion-install.txt",
            vec!["completion", "install", "--help"],
        ),
        (
            "btpc-completion-uninstall.txt",
            vec!["completion", "uninstall", "--help"],
        ),
        ("btpc-manpage.txt", vec!["manpage", "--help"]),
        ("btpc-config.txt", vec!["config", "--help"]),
        ("btpc-config-path.txt", vec!["config", "path", "--help"]),
        ("btpc-config-init.txt", vec!["config", "init", "--help"]),
        ("btpc-config-show.txt", vec!["config", "show", "--help"]),
        ("btpc-config-check.txt", vec!["config", "check", "--help"]),
        (
            "btpc-config-explain.txt",
            vec!["config", "explain", "--help"],
        ),
        (
            "btpc-config-explain-create.txt",
            vec!["config", "explain", "create", "--help"],
        ),
        (
            "btpc-config-tracker.txt",
            vec!["config", "tracker", "--help"],
        ),
        (
            "btpc-config-tracker-list.txt",
            vec!["config", "tracker", "list", "--help"],
        ),
        (
            "btpc-config-tracker-add.txt",
            vec!["config", "tracker", "add", "--help"],
        ),
        (
            "btpc-config-tracker-remove.txt",
            vec!["config", "tracker", "remove", "--help"],
        ),
        ("btpc-config-preset.txt", vec!["config", "preset", "--help"]),
        (
            "btpc-config-preset-list.txt",
            vec!["config", "preset", "list", "--help"],
        ),
        (
            "btpc-config-preset-show.txt",
            vec!["config", "preset", "show", "--help"],
        ),
        (
            "btpc-config-preset-save.txt",
            vec!["config", "preset", "save", "--help"],
        ),
        (
            "btpc-config-preset-remove.txt",
            vec!["config", "preset", "remove", "--help"],
        ),
    ] {
        let output = btpc().args(arguments).output().unwrap();
        assert!(output.status.success());
        assert_eq!(
            normalized_help(&output.stdout),
            normalized_help(&fs::read(root.join(name)).unwrap()),
            "stale help reference {name}"
        );
    }
    let manpage = btpc().arg("manpage").output().unwrap();
    assert!(manpage.status.success());
    assert_eq!(
        normalized_help(&manpage.stdout),
        normalized_help(&fs::read(root.join("btpc.1")).unwrap())
    );

    let completions = root.parent().unwrap().join("completions");
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let output = btpc()
            .args(["completion", "generate", shell])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            normalized_help(&output.stdout),
            normalized_help(&fs::read(completions.join(format!("btpc.{shell}"))).unwrap()),
            "stale {shell} completion"
        );
    }
}
