use std::fs;
use std::path::{Path, PathBuf};

use btpc_core::{Metainfo, TorrentMode};

#[derive(Debug)]
struct Fixture {
    file: String,
    mode: String,
    disposition: String,
    info_hash_v1: String,
    info_hash_v2: String,
    warning_expected: bool,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/interoperability")
}

fn fixtures() -> Vec<Fixture> {
    let manifest = fs::read_to_string(fixture_root().join("manifest.tsv")).unwrap();
    manifest
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let columns = line.split('\t').collect::<Vec<_>>();
            assert_eq!(columns.len(), 9, "invalid fixture row: {line}");
            Fixture {
                file: columns[0].to_owned(),
                mode: columns[3].to_owned(),
                disposition: columns[4].to_owned(),
                info_hash_v1: columns[5].to_owned(),
                info_hash_v2: columns[6].to_owned(),
                warning_expected: columns[4] == "accept-warning"
                    || columns[8].contains("accepted_with_warning"),
            }
        })
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut output, byte| {
        write!(output, "{byte:02x}").unwrap();
        output
    })
}

#[test]
fn documented_interoperability_corpus_obeys_declared_contracts() {
    let fixtures = fixtures();
    assert!(fixtures.len() >= 10, "fixture corpus is unexpectedly small");

    for fixture in fixtures {
        let bytes = fs::read(fixture_root().join(&fixture.file)).unwrap();
        let parsed = Metainfo::from_bytes(&bytes);
        if fixture.disposition == "reject" {
            assert!(parsed.is_err(), "{} unexpectedly parsed", fixture.file);
            continue;
        }

        let torrent = parsed.unwrap_or_else(|error| panic!("{}: {error}", fixture.file));
        let expected_mode = match fixture.mode.as_str() {
            "v1" => TorrentMode::V1,
            "v2" => TorrentMode::V2,
            "hybrid" => TorrentMode::Hybrid,
            mode => panic!("unknown mode {mode}"),
        };
        assert_eq!(torrent.mode(), expected_mode, "{}", fixture.file);
        assert_eq!(torrent.original_bytes(), bytes, "{}", fixture.file);
        assert_eq!(
            torrent.info_hash_v1().map(|hash| hex(hash.as_bytes())),
            (!fixture.info_hash_v1.is_empty()).then_some(fixture.info_hash_v1),
            "{}",
            fixture.file
        );
        assert_eq!(
            torrent.info_hash_v2().map(|hash| hex(hash.as_bytes())),
            (!fixture.info_hash_v2.is_empty()).then_some(fixture.info_hash_v2),
            "{}",
            fixture.file
        );
        assert_eq!(
            torrent.validate().warnings().is_empty(),
            !fixture.warning_expected,
            "{}",
            fixture.file
        );
        let canonical = torrent.to_bytes().unwrap();
        Metainfo::from_bytes(&canonical)
            .unwrap_or_else(|error| panic!("{} canonical output: {error}", fixture.file));
    }
}
