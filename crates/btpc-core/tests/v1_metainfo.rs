use btpc_core::ErrorCategory;
use btpc_core::bencode::OwnedValue;
use btpc_core::metainfo::{RawMetainfo, TorrentMode, V1Metainfo};

macro_rules! dictionary {
    ([$(($key:expr, $value:expr $(,)?)),* $(,)?]) => {
        OwnedValue::dictionary([$(($key.as_slice().to_vec(), $value)),*]).unwrap()
    };
}

fn metainfo(info: OwnedValue) -> Vec<u8> {
    dictionary!([(b"info", info)]).to_vec().unwrap()
}

fn pieces(count: usize) -> Vec<u8> {
    vec![0; count * 20]
}

// Spec: META-V1-001
#[test]
fn validates_single_file_golden_and_non_utf8_name() {
    let bytes = metainfo(dictionary!([
        (b"length", OwnedValue::integer(5)),
        (b"name", OwnedValue::bytes(vec![b'f', 0xff])),
        (b"piece length", OwnedValue::integer(4)),
        (b"pieces", OwnedValue::bytes(pieces(2))),
    ]));
    let raw = RawMetainfo::from_bytes(&bytes).unwrap();
    let v1 = V1Metainfo::from_raw(&raw).unwrap();

    assert_eq!(v1.mode(), TorrentMode::V1);
    assert_eq!(v1.name(), &[b'f', 0xff]);
    assert_eq!(v1.name_utf8(), None);
    assert_eq!(v1.piece_length(), 4);
    assert_eq!(v1.pieces().len(), 40);
    assert_eq!(v1.total_length(), 5);
    assert!(v1.is_single_file());
    assert_eq!(v1.files().len(), 1);
    assert_eq!(v1.files()[0].length(), 5);
    assert_eq!(v1.files()[0].path_components(), &[&[b'f', 0xff][..]]);
    assert!(v1.warnings().is_empty());
}

// Spec: META-V1-001
#[test]
fn validates_multifile_cross_file_piece_count_and_utf8_paths() {
    let files = OwnedValue::list([
        dictionary!([
            (b"length", OwnedValue::integer(3)),
            (
                b"path",
                OwnedValue::list([OwnedValue::bytes(b"a".to_vec())]),
            ),
        ]),
        dictionary!([
            (b"length", OwnedValue::integer(4)),
            (
                b"path",
                OwnedValue::list([
                    OwnedValue::bytes(b"dir".to_vec()),
                    OwnedValue::bytes(b"b".to_vec()),
                ]),
            ),
        ]),
    ]);
    let bytes = metainfo(dictionary!([
        (b"files", files),
        (b"name", OwnedValue::bytes(b"root".to_vec())),
        (b"piece length", OwnedValue::integer(4)),
        (b"pieces", OwnedValue::bytes(pieces(2))),
    ]));
    let raw = RawMetainfo::from_bytes(&bytes).unwrap();
    let v1 = V1Metainfo::from_raw(&raw).unwrap();

    assert_eq!(v1.total_length(), 7);
    assert!(!v1.is_single_file());
    assert_eq!(v1.files()[1].path_components(), &[&b"dir"[..], &b"b"[..]]);
    assert_eq!(v1.files()[1].path_utf8().unwrap(), vec!["dir", "b"]);
}

// Spec: META-V1-001
#[test]
fn accepts_zero_length_payload_with_no_piece_hashes() {
    let bytes = metainfo(dictionary!([
        (b"length", OwnedValue::integer(0)),
        (b"name", OwnedValue::bytes(b"empty".to_vec())),
        (b"piece length", OwnedValue::integer(16)),
        (b"pieces", OwnedValue::bytes(Vec::new())),
    ]));
    let raw = RawMetainfo::from_bytes(&bytes).unwrap();
    let v1 = V1Metainfo::from_raw(&raw).unwrap();
    assert_eq!(v1.total_length(), 0);
    assert!(v1.pieces().is_empty());
}

#[test]
fn typed_numeric_fields_reject_arbitrary_precision_values_with_context() {
    let bytes = b"d4:infod6:lengthi18446744073709551616e4:name1:x12:piece lengthi1e6:pieces0:ee";
    let raw = RawMetainfo::from_bytes(bytes).unwrap();
    let error = V1Metainfo::from_raw(&raw).unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Metainfo);
    assert_eq!(error.field(), Some("info.length"));
    assert!(error.to_string().contains("fit in u64"));
}

// Spec: META-V1-001
#[test]
fn rejects_malformed_v1_fields_and_paths() {
    let cases = [
        dictionary!([
            (b"length", OwnedValue::integer(1)),
            (b"files", OwnedValue::list([])),
            (b"name", OwnedValue::bytes(b"x".to_vec())),
            (b"piece length", OwnedValue::integer(1)),
            (b"pieces", OwnedValue::bytes(pieces(1))),
        ]),
        dictionary!([
            (b"length", OwnedValue::integer(-1)),
            (b"name", OwnedValue::bytes(b"x".to_vec())),
            (b"piece length", OwnedValue::integer(1)),
            (b"pieces", OwnedValue::bytes(Vec::new())),
        ]),
        dictionary!([
            (b"length", OwnedValue::integer(1)),
            (b"name", OwnedValue::bytes(b"x".to_vec())),
            (b"piece length", OwnedValue::integer(0)),
            (b"pieces", OwnedValue::bytes(pieces(1))),
        ]),
        dictionary!([
            (b"length", OwnedValue::integer(1)),
            (b"name", OwnedValue::bytes(b"x".to_vec())),
            (b"piece length", OwnedValue::integer(1)),
            (b"pieces", OwnedValue::bytes(vec![0; 19])),
        ]),
        dictionary!([
            (b"length", OwnedValue::integer(2)),
            (b"name", OwnedValue::bytes(b"x".to_vec())),
            (b"piece length", OwnedValue::integer(1)),
            (b"pieces", OwnedValue::bytes(pieces(1))),
        ]),
    ];

    for info in cases {
        let bytes = metainfo(info);
        let raw = RawMetainfo::from_bytes(&bytes).unwrap();
        assert_eq!(
            V1Metainfo::from_raw(&raw).unwrap_err().category(),
            ErrorCategory::Metainfo
        );
    }

    for component in [b"".as_slice(), b".", b"..", b"a/b", b"a\\b", b"a\0b"] {
        let bytes = metainfo(dictionary!([
            (
                b"files",
                OwnedValue::list([dictionary!([
                    (b"length", OwnedValue::integer(1)),
                    (
                        b"path",
                        OwnedValue::list([OwnedValue::bytes(component.to_vec())]),
                    ),
                ])]),
            ),
            (b"name", OwnedValue::bytes(b"root".to_vec())),
            (b"piece length", OwnedValue::integer(1)),
            (b"pieces", OwnedValue::bytes(pieces(1))),
        ]));
        let raw = RawMetainfo::from_bytes(&bytes).unwrap();
        assert_eq!(
            V1Metainfo::from_raw(&raw).unwrap_err().field(),
            Some("info.files.path")
        );
    }
}

// Spec: META-V1-001
#[test]
fn rejects_checked_total_size_overflow() {
    let file = || {
        dictionary!([
            (b"length", OwnedValue::integer(i64::MAX)),
            (
                b"path",
                OwnedValue::list([OwnedValue::bytes(b"x".to_vec())]),
            ),
        ])
    };
    let bytes = metainfo(dictionary!([
        (b"files", OwnedValue::list([file(), file(), file()])),
        (b"name", OwnedValue::bytes(b"root".to_vec())),
        (b"piece length", OwnedValue::integer(i64::MAX)),
        (b"pieces", OwnedValue::bytes(Vec::new())),
    ]));
    let raw = RawMetainfo::from_bytes(&bytes).unwrap();
    let error = V1Metainfo::from_raw(&raw).unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Metainfo);
    assert_eq!(error.field(), Some("info.files.length"));
}

#[test]
fn rejects_duplicate_and_file_directory_prefix_paths() {
    for paths in [
        vec![vec![b"a".to_vec()], vec![b"a".to_vec()]],
        vec![vec![b"a".to_vec()], vec![b"a".to_vec(), b"b".to_vec()]],
        vec![vec![b"a".to_vec(), b"b".to_vec()], vec![b"a".to_vec()]],
    ] {
        let files = paths.into_iter().map(|path| {
            dictionary!([
                (b"length", OwnedValue::integer(0)),
                (
                    b"path",
                    OwnedValue::list(path.into_iter().map(OwnedValue::bytes)),
                ),
            ])
        });
        let bytes = metainfo(dictionary!([
            (b"files", OwnedValue::list(files)),
            (b"name", OwnedValue::bytes(b"root".to_vec())),
            (b"piece length", OwnedValue::integer(1)),
            (b"pieces", OwnedValue::bytes(Vec::new())),
        ]));
        let raw = RawMetainfo::from_bytes(&bytes).unwrap();
        let error = V1Metainfo::from_raw(&raw).unwrap_err();
        assert_eq!(error.field(), Some("info.files.path"));
        assert!(error.to_string().contains("duplicate or prefix"));
    }
}
