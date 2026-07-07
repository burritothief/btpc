use btpc_core::Metainfo;
use btpc_core::bencode::OwnedValue;
use btpc_core::create::{CreateOptions, Creator, NoProgress};

const INFO: &[u8] = b"4:infod6:lengthi0e4:name1:x12:piece lengthi16384e6:pieces0:e";

fn torrent(fields: &[u8], info_prefix: &[u8]) -> Vec<u8> {
    let mut bytes = b"d".to_vec();
    bytes.extend_from_slice(fields);
    bytes.extend_from_slice(INFO);
    if !info_prefix.is_empty() {
        let info_start = bytes
            .windows(b"4:infod".len())
            .position(|window| window == b"4:infod")
            .unwrap();
        bytes.splice(
            info_start + b"4:infod".len()..info_start + b"4:infod".len(),
            info_prefix.iter().copied(),
        );
    }
    bytes.push(b'e');
    bytes
}

fn owned_torrent(
    fields: impl IntoIterator<Item = (Vec<u8>, OwnedValue)>,
    info_fields: impl IntoIterator<Item = (Vec<u8>, OwnedValue)>,
) -> Vec<u8> {
    let mut info = std::collections::BTreeMap::from([
        (b"length".to_vec(), OwnedValue::integer(0)),
        (b"name".to_vec(), OwnedValue::bytes(b"x".to_vec())),
        (b"piece length".to_vec(), OwnedValue::integer(16_384)),
        (b"pieces".to_vec(), OwnedValue::bytes(Vec::new())),
    ]);
    info.extend(info_fields);
    let mut root = fields
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    root.insert(b"info".to_vec(), OwnedValue::Dictionary(info));
    OwnedValue::Dictionary(root).to_vec().unwrap()
}

#[test]
fn optional_metadata_preserves_lossless_values_and_boundary_ports() {
    let bytes = owned_torrent(
        [
            (
                b"announce".to_vec(),
                OwnedValue::bytes(b"ignored-conflict".to_vec()),
            ),
            (
                b"announce-list".to_vec(),
                OwnedValue::list([OwnedValue::list([
                    OwnedValue::bytes(vec![0xff, b't']),
                    OwnedValue::bytes(vec![0xff, b't']),
                ])]),
            ),
            (b"comment".to_vec(), OwnedValue::bytes(vec![0xfe, b'c'])),
            (b"created by".to_vec(), OwnedValue::bytes(vec![0xfd, b'c'])),
            (b"creation date".to_vec(), OwnedValue::integer(0)),
            (
                b"nodes".to_vec(),
                OwnedValue::list([
                    OwnedValue::list([OwnedValue::bytes(vec![0xf6, b'n']), OwnedValue::integer(1)]),
                    OwnedValue::list([
                        OwnedValue::bytes(b"n2".to_vec()),
                        OwnedValue::integer(65_535),
                    ]),
                ]),
            ),
            (
                b"url-list".to_vec(),
                OwnedValue::list([
                    OwnedValue::bytes(vec![0xfe, b'w']),
                    OwnedValue::bytes(vec![0xfe, b'w']),
                ]),
            ),
        ],
        [(b"source".to_vec(), OwnedValue::bytes(vec![0xfc, b'x']))],
    );
    let parsed = Metainfo::from_bytes(&bytes).unwrap();
    assert_eq!(
        parsed.trackers(),
        &[vec![vec![0xff, b't'], vec![0xff, b't']]]
    );
    assert_eq!(parsed.web_seeds(), &[vec![0xfe, b'w'], vec![0xfe, b'w']]);
    assert_eq!(parsed.nodes()[0].host(), &[0xf6, b'n']);
    assert_eq!(parsed.nodes()[0].port(), 1);
    assert_eq!(parsed.nodes()[1].port(), 65_535);
    assert_eq!(parsed.source(), Some(&[0xfc, b'x'][..]));
    assert_eq!(parsed.comment(), Some(&[0xfe, b'c'][..]));
    assert_eq!(parsed.created_by(), Some(&[0xfd, b'c'][..]));
    assert_eq!(parsed.creation_date(), Some(0));
}

#[test]
fn empty_announce_list_falls_back_with_a_warning() {
    let bytes = owned_torrent(
        [
            (b"announce".to_vec(), OwnedValue::bytes(b"primary".to_vec())),
            (b"announce-list".to_vec(), OwnedValue::list([])),
        ],
        [],
    );
    let parsed = Metainfo::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.trackers(), &[vec![b"primary".to_vec()]]);
    assert_eq!(
        parsed.validate().warnings(),
        &["empty announce-list ignored in favor of announce"]
    );
}

#[test]
fn empty_web_seed_list_means_no_web_seeds_without_a_warning() {
    let bytes = owned_torrent([(b"url-list".to_vec(), OwnedValue::list([]))], []);
    let parsed = Metainfo::from_bytes(&bytes).unwrap();
    assert!(parsed.web_seeds().is_empty());
    assert!(parsed.validate().warnings().is_empty());
}

#[test]
fn malformed_or_empty_optional_metadata_is_rejected() {
    let rejected: &[(&[u8], &[u8])] = &[
        (b"8:announce0:", b""),
        (b"8:announcei1e", b""),
        (b"13:announce-listlee", b""),
        (b"8:url-list0:", b""),
        (b"5:nodeslli1ei1eee", b""),
        (b"5:nodesll0:i1eee", b""),
        (b"5:nodesll1:hi0eee", b""),
        (b"5:nodesll1:hi65536eee", b""),
        (b"7:commenti1e", b""),
        (b"10:created byli1ee", b""),
        (b"13:creation date3:bad", b""),
        (b"13:creation datei-1e", b""),
        (b"13:creation datei999999999999999999999999e", b""),
        (b"", b"6:sourcei1e"),
    ];
    for (fields, info) in rejected {
        assert!(
            Metainfo::from_bytes(&torrent(fields, info)).is_err(),
            "{fields:?} {info:?}"
        );
    }
}

#[test]
fn creation_uses_the_same_strict_domains() {
    assert!(
        CreateOptions::builder()
            .trackers([Vec::new()])
            .build()
            .is_err()
    );
    assert!(
        CreateOptions::builder()
            .nodes([(b"host".to_vec(), 0)])
            .build()
            .is_err()
    );
    assert!(CreateOptions::builder().creation_date(-1).build().is_err());

    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    std::fs::write(&payload, b"data").unwrap();
    let options = CreateOptions::builder()
        .trackers([vec![b"tracker".to_vec(), b"tracker".to_vec()]])
        .web_seeds([b"seed".to_vec(), b"seed".to_vec()])
        .nodes([(b"host".to_vec(), 1), (b"host".to_vec(), 65_535)])
        .comment(vec![0xff])
        .source(vec![0xfe])
        .creation_date(0)
        .build()
        .unwrap();
    let bytes = Creator::new(payload)
        .options(options)
        .create(&NoProgress)
        .unwrap();
    let parsed = Metainfo::from_bytes(bytes.bytes()).unwrap();
    assert!(parsed.validate().warnings().is_empty());
    assert_eq!(parsed.nodes().len(), 2);
}
