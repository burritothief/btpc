use btpc_core::ErrorCategory;
use btpc_core::bencode::Value;
use btpc_core::metainfo::RawMetainfo;

// Spec: META-RAW-001
#[test]
fn rejects_missing_duplicate_and_non_dictionary_info() {
    for (input, field) in [
        (&b"d8:announce3:urle"[..], "info"),
        (b"d4:infode4:infodee", "info"),
        (b"d4:info4:spame", "info"),
        (b"li1ee", "<root>"),
    ] {
        let error = RawMetainfo::from_bytes(input).unwrap_err();
        assert_eq!(error.category(), ErrorCategory::Metainfo, "{input:?}");
        assert_eq!(error.field(), Some(field));
    }
}

// Spec: META-RAW-001
// Spec: META-FIELD-001
#[test]
fn preserves_original_info_bytes_and_parses_common_fields() {
    let bytes = b"d8:announce7:http://13:announce-listll7:http://ee7:comment2:\xffx10:created by4:btpc13:creation datei123e4:infod4:name03:fooe5:nodesll4:hosti80eee7:unknown3:raw8:url-list7:http://e";
    let raw = RawMetainfo::from_bytes(bytes).unwrap();

    assert_eq!(raw.original_bytes(), bytes);
    assert_eq!(raw.info_bytes(), b"d4:name03:fooe");
    assert_eq!(&bytes[raw.info_span().range()], raw.info_bytes());
    assert_eq!(
        raw.announce().and_then(Value::as_bytes),
        Some(&b"http://"[..])
    );
    assert!(raw.announce_list().is_some());
    assert_eq!(
        raw.comment().and_then(Value::as_bytes),
        Some(&[0xff, b'x'][..])
    );
    assert_eq!(
        raw.created_by().and_then(Value::as_bytes),
        Some(&b"btpc"[..])
    );
    assert_eq!(raw.creation_date().and_then(Value::as_integer), Some(123));
    assert!(raw.nodes().is_some());
    assert_eq!(
        raw.url_list().and_then(Value::as_bytes),
        Some(&b"http://"[..])
    );
    assert_eq!(raw.unknown_fields().len(), 1);
    assert_eq!(raw.unknown_fields()[0].0.bytes(), b"unknown");
    assert_eq!(raw.unknown_fields()[0].1.as_bytes(), Some(&b"raw"[..]));
}

// Spec: META-HASH-001
#[test]
fn hashes_exact_noncanonical_info_bytes_with_known_digests() {
    let raw = RawMetainfo::from_bytes(b"d4:infod4:name03:fooee").unwrap();

    assert_eq!(
        raw.info_hash_sha1(),
        [
            0x4f, 0xc2, 0x41, 0xb5, 0x25, 0x0d, 0x47, 0x63, 0x9a, 0xdd, 0x45, 0xed, 0x70, 0xe5,
            0x18, 0xde, 0x67, 0x90, 0xc1, 0x86,
        ]
    );
    assert_eq!(
        raw.info_hash_sha256(),
        [
            0xa6, 0x9e, 0x92, 0x68, 0x1c, 0xef, 0xa6, 0x6d, 0xf2, 0xf8, 0x70, 0xa9, 0xcd, 0x42,
            0x45, 0xc6, 0x17, 0x61, 0x8f, 0x79, 0x5a, 0xe8, 0x74, 0xd7, 0x18, 0x9b, 0x60, 0x9f,
            0xd6, 0x5b, 0x01, 0xf9,
        ]
    );
}

// Spec: META-HASH-001
#[test]
fn top_level_changes_do_not_change_hashes_but_info_changes_do() {
    let first = RawMetainfo::from_bytes(b"d7:comment1:a4:infod4:name3:fooee").unwrap();
    let top_level_changed = RawMetainfo::from_bytes(b"d7:comment1:b4:infod4:name3:fooee").unwrap();
    let info_changed = RawMetainfo::from_bytes(b"d7:comment1:a4:infod4:name3:fopee").unwrap();

    assert_eq!(first.info_hash_sha1(), top_level_changed.info_hash_sha1());
    assert_eq!(
        first.info_hash_sha256(),
        top_level_changed.info_hash_sha256()
    );
    assert_ne!(first.info_hash_sha1(), info_changed.info_hash_sha1());
    assert_ne!(first.info_hash_sha256(), info_changed.info_hash_sha256());
}
