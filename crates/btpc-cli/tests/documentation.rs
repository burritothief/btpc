use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

// Spec: CLI-DOC-001
#[test]
fn readme_cli_tour_runs_for_every_mode() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir_all(&payload).unwrap();
    fs::write(payload.join("hello.txt"), b"hello torrent\n").unwrap();

    for mode in ["v1", "v2", "hybrid"] {
        let torrent = temp.path().join(format!("payload-{mode}.torrent"));
        let mut create = btpc();
        create.args(["create", payload.to_str().unwrap(), "--mode", mode]);
        if mode != "v1" {
            create.args(["--piece-length", "16384"]);
        }
        create
            .args(["--threads", "1", "--creation-date", "0", "-o"])
            .arg(&torrent)
            .assert()
            .success();
        btpc()
            .args(["inspect", torrent.to_str().unwrap()])
            .assert()
            .success();
        btpc()
            .args(["validate", torrent.to_str().unwrap()])
            .assert()
            .success();
        btpc()
            .args(["magnet", torrent.to_str().unwrap()])
            .assert()
            .success();
        btpc()
            .args([
                "verify",
                torrent.to_str().unwrap(),
                payload.to_str().unwrap(),
            ])
            .assert()
            .success();
    }
}
