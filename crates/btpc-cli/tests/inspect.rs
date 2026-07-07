use std::fs;

use assert_cmd::Command;
use btpc_core::bencode::OwnedValue;
use btpc_core::create::{Creator, NoProgress};
use predicates::prelude::*;
use tempfile::TempDir;

macro_rules! dictionary {
    ([$(($key:expr, $value:expr $(,)?)),* $(,)?]) => {
        OwnedValue::dictionary([$(($key.as_slice().to_vec(), $value)),*]).unwrap()
    };
}

fn btpc() -> Command {
    Command::cargo_bin("btpc").unwrap()
}

fn fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let torrent = temp.path().join("payload.torrent");
    fs::write(&payload, b"data").unwrap();
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
fn inspect_human_and_json_are_machine_safe() {
    let (_temp, torrent) = fixture();
    let metainfo = btpc_core::Metainfo::from_path(&torrent).unwrap();
    let hash = metainfo.info_hash_v1().unwrap().hex();
    let expected = format!(
        "Torrent info:\n  Name:         payload\n  Mode:         v1\n  Info hash v1: {hash}\n  Size:         4 B\n  Piece length: 16.0 KiB\n  Pieces:       1\n  Magnet:       magnet:?xt=urn:btih:{hash}&dn=payload\n  Created by:   btpc/0.1.0\n"
    );
    btpc()
        .args(["inspect", torrent.to_str().unwrap()])
        .assert()
        .success()
        .stdout(expected)
        .stderr("");
    let output = btpc()
        .args(["inspect", torrent.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema"], "btpc.inspect.v1");
    assert_eq!(json["mode"], "v1");
    assert_eq!(json["name"]["encoding"], "utf-8");
}

#[test]
fn inspect_hybrid_summary_orders_metadata_tiers_and_redacts_secrets() {
    use btpc_core::create::{CreateMode, CreateOptions, PieceLength};

    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let torrent = temp.path().join("payload.torrent");
    fs::write(&payload, b"metadata").unwrap();
    let options = CreateOptions::builder()
        .mode(CreateMode::Hybrid)
        .piece_length(PieceLength::Exact(16 * 1024))
        .trackers([
            vec![b"https://one.example/announce".to_vec()],
            vec![b"https://user:password@two.example/announce?passkey=hidden".to_vec()],
        ])
        .web_seeds([b"https://seed.example/payload".to_vec()])
        .nodes([(vec![0xfe, b'n'], 1), (b"router.example".to_vec(), 65_535)])
        .private(false)
        .source(vec![0xff, b's'])
        .comment(b"comment".to_vec())
        .created_by(b"btpc-test".to_vec())
        .creation_date(0)
        .build()
        .unwrap();
    Creator::new(&payload)
        .options(options)
        .create_to_path(
            &torrent,
            btpc_core::create::OverwritePolicy::Deny,
            &NoProgress,
        )
        .unwrap();
    let output = btpc()
        .env("TZ", "UTC")
        .args(["inspect", torrent.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let human = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "Torrent info:\n",
        "  Mode:          hybrid\n",
        "  Info hash v1:",
        "  Info hash v2:",
        "  Private:       no\n",
        "  Source:        0xff73\n",
        "  Comment:       comment\n",
        "  Created by:    btpc-test\n",
        "  Creation date: 1970-01-01 00:00:00 +00:00\n",
        "  DHT node:      0xfe6e:1\n",
        "  DHT node:      router.example:65535\n",
        "  Trackers:\n    Tier 1:\n      https://one.example/announce\n    Tier 2:\n      <redacted-url>\n",
        "  Web seeds:\n    https://seed.example/payload\n",
    ] {
        assert!(human.contains(expected), "missing {expected:?} in {human}");
    }
    assert!(!human.contains("password"));
    assert!(!human.contains("hidden"));
    assert!(!human.contains("password%40"));
    assert!(human.is_ascii());

    let output = btpc()
        .args(["inspect", torrent.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["nodes"][0]["host"]["value"], "fe6e");
    assert_eq!(json["nodes"][0]["port"], 1);
    assert_eq!(json["source"]["value"], "ff73");
    assert_eq!(json["comment"]["value"], "comment");
    assert_eq!(json["created_by"]["value"], "btpc-test");
    assert_eq!(json["creation_date"], 0);
}

#[test]
fn inspect_and_validate_reject_malformed_recognized_optional_fields() {
    let temp = TempDir::new().unwrap();
    let torrent = temp.path().join("invalid.torrent");
    fs::write(
        &torrent,
        b"d7:commenti1e4:infod6:lengthi0e4:name1:x12:piece lengthi16384e6:pieces0:ee",
    )
    .unwrap();
    for command in ["inspect", "validate"] {
        btpc()
            .args([command, torrent.to_str().unwrap()])
            .assert()
            .code(4)
            .stderr(predicate::str::contains("comment"));
    }
}

#[test]
fn validate_reads_only_metainfo_and_maps_invalid_data() {
    let (_temp, torrent) = fixture();
    btpc()
        .args(["validate", torrent.to_str().unwrap()])
        .assert()
        .success()
        .stdout("valid\n")
        .stderr("");
    btpc()
        .args(["validate", torrent.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\":\"btpc.validate.v1\""));
    fs::write(&torrent, b"not bencode").unwrap();
    btpc()
        .args(["validate", torrent.to_str().unwrap()])
        .assert()
        .code(4)
        .stdout("")
        .stderr(predicate::str::contains("bencode"));
}

#[test]
fn read_commands_share_configurable_resource_limits() {
    let (_temp, torrent) = fixture();
    let length = fs::metadata(&torrent).unwrap().len().to_string();
    for command in ["inspect", "validate", "magnet"] {
        btpc()
            .args([
                command,
                torrent.to_str().unwrap(),
                "--max-input-bytes",
                &length,
            ])
            .assert()
            .success();
        btpc()
            .args([command, torrent.to_str().unwrap(), "--max-input-bytes", "1"])
            .assert()
            .code(4)
            .stderr(predicate::str::contains("total input"));
        btpc()
            .args([
                command,
                torrent.to_str().unwrap(),
                "--max-integer-digits",
                "1",
            ])
            .assert()
            .code(4)
            .stderr(predicate::str::contains("integer digits"));
    }
}

#[test]
fn validate_json_distinguishes_noncanonical_valid_input() {
    let (_temp, torrent) = fixture();
    let canonical = fs::read(&torrent).unwrap();
    let marker = b"12:piece lengthi16384e";
    let offset = canonical
        .windows(marker.len())
        .position(|window| window == marker)
        .unwrap();
    let mut noncanonical = canonical.clone();
    noncanonical.splice(
        offset..offset + marker.len(),
        b"12:piece lengthi016384e".iter().copied(),
    );
    fs::write(&torrent, noncanonical).unwrap();

    let output = btpc()
        .args(["validate", torrent.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["valid"], true);
    assert_eq!(json["canonical"], false);
}

#[test]
fn validate_rejects_ambiguous_v1_file_graphs() {
    let temp = TempDir::new().unwrap();
    let torrent = temp.path().join("ambiguous.torrent");
    let files = OwnedValue::list([
        dictionary!([
            (b"length", OwnedValue::integer(0)),
            (
                b"path",
                OwnedValue::list([OwnedValue::bytes(b"a".to_vec())]),
            ),
        ]),
        dictionary!([
            (b"length", OwnedValue::integer(0)),
            (
                b"path",
                OwnedValue::list([
                    OwnedValue::bytes(b"a".to_vec()),
                    OwnedValue::bytes(b"b".to_vec()),
                ]),
            ),
        ]),
    ]);
    let bytes = dictionary!([(
        b"info",
        dictionary!([
            (b"files", files),
            (b"name", OwnedValue::bytes(b"root".to_vec())),
            (b"piece length", OwnedValue::integer(1)),
            (b"pieces", OwnedValue::bytes(Vec::new())),
        ]),
    )])
    .to_vec()
    .unwrap();
    fs::write(&torrent, bytes).unwrap();
    btpc()
        .args(["validate", torrent.to_str().unwrap()])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("duplicate or prefix"));
}

#[test]
fn inspect_json_covers_v2_hybrid_and_non_utf8_names() {
    let temp = TempDir::new().unwrap();
    for (name, hybrid, expected) in [("v2", false, "v2"), ("hybrid", true, "hybrid")] {
        let mut info = vec![
            (
                b"file tree".to_vec(),
                dictionary!([(
                    b"root",
                    dictionary!([(
                        b"",
                        dictionary!([
                            (b"length", OwnedValue::integer(1)),
                            (b"pieces root", OwnedValue::bytes(vec![1; 32]))
                        ])
                    )])
                )]),
            ),
            (b"meta version".to_vec(), OwnedValue::integer(2)),
            (b"name".to_vec(), OwnedValue::bytes(b"root".to_vec())),
            (b"piece length".to_vec(), OwnedValue::integer(16_384)),
        ];
        if hybrid {
            info.push((b"length".to_vec(), OwnedValue::integer(1)));
            info.push((b"pieces".to_vec(), OwnedValue::bytes(vec![0; 20])));
        }
        let bytes = OwnedValue::dictionary([
            (b"comment".to_vec(), OwnedValue::bytes(vec![b'n', 0xff])),
            (b"info".to_vec(), OwnedValue::dictionary(info).unwrap()),
            (b"piece layers".to_vec(), dictionary!([])),
        ])
        .unwrap()
        .to_vec()
        .unwrap();
        let path = temp.path().join(format!("{name}.torrent"));
        fs::write(&path, bytes).unwrap();
        let output = btpc()
            .args(["inspect", path.to_str().unwrap(), "--json"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(json["mode"], expected);
        assert_eq!(json["name"]["encoding"], "utf-8");
    }
}

#[test]
fn inspect_fields_formats_and_pagination_are_stable() {
    let (_temp, torrent) = fixture();
    btpc()
        .args([
            "inspect",
            torrent.to_str().unwrap(),
            "--field",
            "mode",
            "--format",
            "plain",
        ])
        .assert()
        .success()
        .stdout("v1\n")
        .stderr("");
    btpc()
        .args([
            "inspect",
            torrent.to_str().unwrap(),
            "--field",
            "file-count",
            "--field",
            "mode",
            "--format",
            "tsv",
        ])
        .assert()
        .success()
        .stdout("file-count\t1\nmode\tv1\n");
    let output = btpc()
        .args([
            "inspect",
            torrent.to_str().unwrap(),
            "--files",
            "--offset",
            "0",
            "--limit",
            "1",
            "--format",
            "json-pretty",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema"], "btpc.inspect.selection.v1");
    assert_eq!(json["fields"][0]["name"], "files");
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("\n  \"schema\"")
    );
}

#[test]
fn validate_policy_flags_and_pretty_json_preserve_no_payload_access() {
    let (temp, torrent) = fixture();
    let missing = temp.path().join("missing-payload");
    assert!(!missing.exists());
    btpc()
        .args(["validate", torrent.to_str().unwrap(), "--canonical"])
        .assert()
        .success()
        .stdout("valid\n");
    let output = btpc()
        .args([
            "validate",
            torrent.to_str().unwrap(),
            "--format",
            "json-pretty",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("\n  \"schema\"")
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn every_selector_handles_absent_values_and_non_utf8_paths() {
    let (_temp, torrent) = fixture();
    let fields = [
        "mode",
        "name",
        "total-size",
        "piece-length",
        "piece-count",
        "file-count",
        "hash-v1",
        "hash-v2",
        "private",
        "trackers",
        "web-seeds",
        "nodes",
        "comment",
        "creator",
        "creation-date",
        "source",
        "canonicality",
        "warnings",
        "files",
        "unknown-fields",
    ];
    let mut args = vec!["inspect", torrent.to_str().unwrap(), "--format", "json"];
    for field in fields {
        args.extend(["--field", field]);
    }
    let output = btpc().args(args).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["fields"].as_array().unwrap().len(), fields.len());
    btpc()
        .args([
            "inspect",
            torrent.to_str().unwrap(),
            "--field",
            "hash-v2",
            "--format",
            "plain",
        ])
        .assert()
        .success()
        .stdout("\n");

    let temp = TempDir::new().unwrap();
    let torrent = temp.path().join("nonutf.torrent");
    let bytes = dictionary!([(
        b"info",
        dictionary!([
            (
                b"files",
                OwnedValue::list([dictionary!([
                    (b"length", OwnedValue::integer(0)),
                    (
                        b"path",
                        OwnedValue::list([OwnedValue::bytes(vec![b'a', 0xff])])
                    ),
                ])])
            ),
            (b"name", OwnedValue::bytes(b"root".to_vec())),
            (b"piece length", OwnedValue::integer(16_384)),
            (b"pieces", OwnedValue::bytes(Vec::new())),
        ])
    )])
    .to_vec()
    .unwrap();
    fs::write(&torrent, bytes).unwrap();
    btpc()
        .args([
            "inspect",
            torrent.to_str().unwrap(),
            "--files",
            "--format",
            "plain",
        ])
        .assert()
        .code(4);
    btpc()
        .args([
            "inspect",
            torrent.to_str().unwrap(),
            "--files",
            "--path-encoding",
            "escaped",
            "--format",
            "plain",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r"a\\xff"));
    btpc()
        .args([
            "inspect",
            torrent.to_str().unwrap(),
            "--files",
            "--path-encoding",
            "hex",
            "--format",
            "plain",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("61ff"));
}

#[test]
fn canonical_policy_fails_noncanonical_input_with_stable_data_exit() {
    let (_temp, torrent) = fixture();
    let canonical = fs::read(&torrent).unwrap();
    let marker = b"12:piece lengthi16384e";
    let offset = canonical
        .windows(marker.len())
        .position(|window| window == marker)
        .unwrap();
    let mut noncanonical = canonical;
    noncanonical.splice(
        offset..offset + marker.len(),
        b"12:piece lengthi016384e".iter().copied(),
    );
    fs::write(&torrent, noncanonical).unwrap();
    btpc()
        .args([
            "validate",
            torrent.to_str().unwrap(),
            "--canonical",
            "--json",
        ])
        .assert()
        .code(4)
        .stdout(predicate::str::contains("\"valid\":false"));
}

#[test]
fn verbose_and_pretty_human_inspect_add_details_and_nested_tree() {
    // Spec: TEST-CLI-DISPLAY-001
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("root");
    fs::create_dir_all(payload.join("directory/nested")).unwrap();
    fs::write(payload.join("directory/nested/file.bin"), vec![0_u8; 1536]).unwrap();
    fs::write(payload.join("other.txt"), b"hello").unwrap();
    let torrent = temp.path().join("root.torrent");
    Creator::new(&payload)
        .create_to_path(
            &torrent,
            btpc_core::create::OverwritePolicy::Deny,
            &NoProgress,
        )
        .unwrap();

    let output = btpc()
        .args(["-v", "inspect", torrent.to_str().unwrap(), "--pretty"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let human = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "\nDetails:\n",
        "  Canonical: yes\n",
        "  Payload files: 2\n",
        "  Padding files: 0\n",
        "\nAdditional metadata:\n  Created by: btpc/0.1.0\n",
        "\nFile tree:\nroot/\n",
        "|-- directory/\n",
        "|   `-- nested/\n",
        "|       `-- file.bin (1.5 KiB)\n",
        "`-- other.txt (5 B)\n",
    ] {
        assert!(human.contains(expected), "missing {expected:?} in {human}");
    }
    assert!(!human.contains("pieces root"));
    assert!(!human.contains("piece layers"));
}

#[test]
fn explicit_tree_respects_pagination_and_path_encoding() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("root");
    fs::create_dir(&payload).unwrap();
    for name in ["a", "b", "c"] {
        fs::write(payload.join(name), name).unwrap();
    }
    let torrent = temp.path().join("root.torrent");
    Creator::new(&payload)
        .create_to_path(
            &torrent,
            btpc_core::create::OverwritePolicy::Deny,
            &NoProgress,
        )
        .unwrap();
    btpc()
        .args([
            "inspect",
            torrent.to_str().unwrap(),
            "--tree",
            "--offset",
            "1",
            "--limit",
            "1",
            "--path-encoding",
            "escaped",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("`-- b (1 B)"))
        .stdout(predicate::str::contains("... 2 file(s) omitted"))
        .stdout(predicate::str::contains("a (1 B)").not())
        .stdout(predicate::str::contains("c (1 B)").not());
}
