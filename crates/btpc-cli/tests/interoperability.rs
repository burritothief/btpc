use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/interoperability")
}

#[test]
fn validate_and_inspect_cover_the_documented_fixture_corpus() {
    let manifest = fs::read_to_string(fixture_root().join("manifest.tsv")).unwrap();
    for line in manifest
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let columns = line.split('\t').collect::<Vec<_>>();
        let path = fixture_root().join(columns[0]);
        let accepted = columns[4] != "reject";

        Command::cargo_bin("btpc")
            .unwrap()
            .args(["validate", path.to_str().unwrap(), "--json"])
            .assert()
            .code(if accepted { 0 } else { 4 });
        if accepted {
            Command::cargo_bin("btpc")
                .unwrap()
                .args(["inspect", path.to_str().unwrap(), "--json"])
                .assert()
                .success();
        }
    }
}
