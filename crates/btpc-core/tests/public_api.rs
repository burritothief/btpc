use std::fs;
use std::io::Cursor;

use btpc_core::bencode::OwnedValue;
use btpc_core::{
    Canonicality, ErrorCategory, Metainfo, ParseLimits, ParseOptions, TorrentBytes, TorrentMode,
    TorrentPath,
};

macro_rules! dictionary {
    ([$(($key:expr, $value:expr $(,)?)),* $(,)?]) => {
        OwnedValue::dictionary([$(($key.as_slice().to_vec(), $value)),*]).unwrap()
    };
}

fn v1_bytes() -> Vec<u8> {
    dictionary!([
        (b"announce", OwnedValue::bytes(b"https://tracker".to_vec())),
        (b"comment", OwnedValue::bytes(b"kept".to_vec())),
        (
            b"info",
            dictionary!([
                (b"length", OwnedValue::integer(3)),
                (b"name", OwnedValue::bytes(b"file".to_vec())),
                (b"piece length", OwnedValue::integer(4)),
                (b"pieces", OwnedValue::bytes(vec![1; 20])),
                (b"private", OwnedValue::integer(1)),
            ]),
        ),
        (
            b"url-list",
            OwnedValue::list([OwnedValue::bytes(b"https://seed".to_vec())])
        ),
        (b"x-extension", OwnedValue::integer(7)),
    ])
    .to_vec()
    .unwrap()
}

fn v2_bytes(hybrid: bool) -> Vec<u8> {
    let mut info = vec![
        (
            b"file tree".to_vec(),
            dictionary!([(
                b"file",
                dictionary!([(
                    b"",
                    dictionary!([
                        (b"length", OwnedValue::integer(1)),
                        (b"pieces root", OwnedValue::bytes(vec![2; 32])),
                    ]),
                )]),
            )]),
        ),
        (b"meta version".to_vec(), OwnedValue::integer(2)),
        (b"name".to_vec(), OwnedValue::bytes(b"file".to_vec())),
        (b"piece length".to_vec(), OwnedValue::integer(16_384)),
    ];
    if hybrid {
        info.push((b"length".to_vec(), OwnedValue::integer(1)));
        info.push((b"pieces".to_vec(), OwnedValue::bytes(vec![3; 20])));
    }
    OwnedValue::dictionary([
        (b"info".to_vec(), OwnedValue::dictionary(info).unwrap()),
        (b"piece layers".to_vec(), dictionary!([])),
    ])
    .unwrap()
    .to_vec()
    .unwrap()
}

#[test]
fn public_v1_api_preserves_originals_and_exposes_owned_values() {
    let bytes = v1_bytes();
    let torrent = Metainfo::from_bytes(&bytes).unwrap();

    assert_eq!(torrent.mode(), TorrentMode::V1);
    assert_eq!(torrent.name(), b"file");
    assert_eq!(torrent.piece_length(), 4);
    assert_eq!(torrent.files().len(), 1);
    assert_eq!(torrent.files()[0].length(), 3);
    assert_eq!(torrent.files()[0].path_components(), &[b"file".to_vec()]);
    assert_eq!(torrent.trackers(), &[vec![b"https://tracker".to_vec()]]);
    assert_eq!(torrent.web_seeds(), &[b"https://seed".to_vec()]);
    assert_eq!(torrent.private(), Some(true));
    assert!(torrent.info_hash_v1().is_some());
    assert!(torrent.info_hash_v2().is_none());
    assert_eq!(torrent.original_bytes(), bytes);
    assert_eq!(torrent.unknown_fields().len(), 1);
    let unknown = &torrent.unknown_fields()[0];
    assert_eq!(torrent.unknown_field_bytes(unknown), b"11:x-extensioni7e");
    assert_eq!(unknown.span().end() - unknown.span().start(), 17);
    assert!(torrent.validate().is_valid());

    let mut original = Vec::new();
    torrent.write_original(&mut original).unwrap();
    assert_eq!(original, bytes);
    let canonical = torrent.to_bytes().unwrap();
    assert_eq!(canonical, bytes);
}

#[test]
fn from_path_and_writer_api_work_without_borrowing_the_source() {
    let path = std::env::temp_dir().join(format!("btpc-public-api-{}.torrent", std::process::id()));
    fs::write(&path, v1_bytes()).unwrap();
    let torrent = Metainfo::from_path(&path).unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(torrent.name_utf8(), Some("file"));
    let mut writer = Cursor::new(Vec::new());
    torrent.write_canonical(&mut writer).unwrap();
    assert_eq!(writer.into_inner(), v1_bytes());
}

#[test]
fn load_options_enforce_input_and_owned_boundaries_for_bytes_and_paths() {
    let bytes = v1_bytes();
    let input_limit = ParseLimits::new(128, 1_000_000, bytes.len(), bytes.len(), usize::MAX);
    Metainfo::from_bytes_with_options(&bytes, ParseOptions::new(input_limit)).unwrap();

    let over_input = ParseLimits::new(128, 1_000_000, bytes.len(), bytes.len() - 1, usize::MAX);
    let error =
        Metainfo::from_bytes_with_options(&bytes, ParseOptions::new(over_input)).unwrap_err();
    assert_eq!(error.category(), ErrorCategory::ResourceLimit);
    assert_eq!(error.limit(), Some("total input"));

    let minimum_owned = (0..=256 * bytes.len())
        .find(|maximum| {
            let limits = ParseLimits::new(128, 1_000_000, bytes.len(), bytes.len(), *maximum);
            Metainfo::from_bytes_with_options(&bytes, ParseOptions::new(limits)).is_ok()
        })
        .expect("a bounded owned allocation must succeed");
    let exact = ParseLimits::new(128, 1_000_000, bytes.len(), bytes.len(), minimum_owned);
    Metainfo::from_bytes_with_options(&bytes, ParseOptions::new(exact)).unwrap();
    let under = ParseLimits::new(128, 1_000_000, bytes.len(), bytes.len(), minimum_owned - 1);
    let error = Metainfo::from_bytes_with_options(&bytes, ParseOptions::new(under)).unwrap_err();
    assert_eq!(error.category(), ErrorCategory::ResourceLimit);
    assert_eq!(error.limit(), Some("owned allocation"));

    let path = std::env::temp_dir().join(format!(
        "btpc-load-options-{}-{}.torrent",
        std::process::id(),
        minimum_owned
    ));
    fs::write(&path, &bytes).unwrap();
    Metainfo::from_path_with_options(&path, ParseOptions::new(exact)).unwrap();
    let error = Metainfo::from_path_with_options(&path, ParseOptions::new(over_input)).unwrap_err();
    fs::remove_file(path).unwrap();
    assert_eq!(error.category(), ErrorCategory::ResourceLimit);
    assert_eq!(error.limit(), Some("total input"));
}

#[test]
fn allocation_budget_includes_unknown_fields_and_canonical_expansion() {
    let extension = vec![b'x'; 4_096];
    let root = dictionary!([
        (
            b"info",
            dictionary!([
                (b"length", OwnedValue::integer(0)),
                (b"name", OwnedValue::bytes(b"x".to_vec())),
                (b"piece length", OwnedValue::integer(1)),
                (b"pieces", OwnedValue::bytes(Vec::new())),
            ])
        ),
        (b"unknown", OwnedValue::bytes(extension)),
    ]);
    let bytes = root.to_vec().unwrap();
    let limits = ParseLimits::new(128, 1_000_000, bytes.len(), bytes.len(), bytes.len());
    let error = Metainfo::from_bytes_with_options(&bytes, ParseOptions::new(limits)).unwrap_err();
    assert_eq!(error.category(), ErrorCategory::ResourceLimit);
    assert_eq!(error.limit(), Some("owned allocation"));
}

#[test]
fn unknown_arbitrary_precision_integer_survives_owned_round_trip() {
    let bytes = b"d4:infod6:lengthi0e4:name1:x12:piece lengthi1e6:pieces0:e7:unknowni123456789012345678901234567890ee";
    let torrent = Metainfo::from_bytes(bytes).unwrap();
    assert_eq!(torrent.to_bytes().unwrap(), bytes);
}

#[test]
fn noncanonical_arbitrary_precision_integer_normalizes_without_truncation() {
    let bytes = b"d4:infod6:lengthi0e4:name1:x12:piece lengthi1e6:pieces0:e7:unknowni000123456789012345678901234567890ee";
    let torrent = Metainfo::from_bytes(bytes).unwrap();
    assert!(!torrent.validate().canonicality().is_canonical());
    assert_eq!(
        torrent.to_bytes().unwrap(),
        b"d4:infod6:lengthi0e4:name1:x12:piece lengthi1e6:pieces0:e7:unknowni123456789012345678901234567890ee"
    );
}

#[test]
fn hash_value_types_format_and_retain_raw_bytes() {
    let torrent = Metainfo::from_bytes(&v1_bytes()).unwrap();
    let hash = torrent.info_hash_v1().unwrap();
    assert_eq!(hash.as_bytes().len(), 20);
    assert_eq!(hash.to_string().len(), 40);
    assert_eq!(hash.hex(), hash.to_string());
}

#[test]
fn validation_rejects_invalid_protocol_data_at_construction() {
    let invalid = dictionary!([(
        b"info",
        dictionary!([
            (b"length", OwnedValue::integer(2)),
            (b"name", OwnedValue::bytes(b"x".to_vec())),
            (b"piece length", OwnedValue::integer(1)),
            (b"pieces", OwnedValue::bytes(vec![0; 20])),
        ]),
    )])
    .to_vec()
    .unwrap();
    assert!(Metainfo::from_bytes(&invalid).is_err());
}

#[test]
fn syntax_canonicality_protocol_and_warnings_remain_distinct() {
    let canonical = v1_bytes();
    let marker = b"12:piece lengthi4e";
    let offset = canonical
        .windows(marker.len())
        .position(|window| window == marker)
        .unwrap();
    let mut noncanonical = canonical.clone();
    noncanonical.splice(
        offset..offset + marker.len(),
        b"12:piece lengthi04e".iter().copied(),
    );

    let raw = btpc_core::metainfo::RawMetainfo::from_bytes(&noncanonical).unwrap();
    assert!(matches!(
        raw.canonicality(),
        Canonicality::NonCanonical { .. }
    ));
    let validated = Metainfo::from_bytes(&noncanonical).unwrap();
    assert!(validated.validate().is_valid());
    assert!(!validated.validate().canonicality().is_canonical());
    assert!(validated.validate().canonicality().offset().is_some());
    btpc_core::bencode::validate_canonical(&validated.to_bytes().unwrap()).unwrap();

    let malformed = Metainfo::from_bytes(b"not bencode").unwrap_err();
    assert_eq!(malformed.category(), ErrorCategory::BencodeSyntax);
    let invalid = dictionary!([(
        b"info",
        dictionary!([
            (b"length", OwnedValue::integer(2)),
            (b"name", OwnedValue::bytes(b"x".to_vec())),
            (b"piece length", OwnedValue::integer(1)),
            (b"pieces", OwnedValue::bytes(vec![0; 20])),
        ]),
    )])
    .to_vec()
    .unwrap();
    assert_eq!(
        Metainfo::from_bytes(&invalid).unwrap_err().category(),
        ErrorCategory::Metainfo
    );

    let warned = Metainfo::from_bytes(&v2_bytes(false)).unwrap();
    assert!(warned.validate().canonicality().is_canonical());
    assert_eq!(warned.validate().warning_details().len(), 1);
    assert_eq!(
        warned.validate().warning_details()[0].message(),
        "piece layers is present but empty; canonical output omits it"
    );
    assert_eq!(
        warned.validate().warning_details()[0].field(),
        Some("piece layers")
    );
    assert!(warned.validate().warning_details()[0].offset().is_some());
}

#[test]
fn public_v2_and_hybrid_modes_expose_applicable_hashes() {
    let v2 = Metainfo::from_bytes(&v2_bytes(false)).unwrap();
    assert_eq!(v2.mode(), TorrentMode::V2);
    assert!(v2.info_hash_v1().is_none());
    assert!(v2.info_hash_v2().is_some());
    assert_eq!(v2.files()[0].pieces_root(), Some(&[2; 32]));

    let hybrid = Metainfo::from_bytes(&v2_bytes(true)).unwrap();
    assert_eq!(hybrid.mode(), TorrentMode::Hybrid);
    assert!(hybrid.info_hash_v1().is_some());
    assert!(hybrid.info_hash_v2().is_some());
}

#[test]
fn torrent_byte_and_path_values_preserve_raw_identity() {
    let utf8 = TorrentBytes::new("é".as_bytes().to_vec());
    let decomposed = TorrentBytes::new("e\u{301}".as_bytes().to_vec());
    let invalid = TorrentBytes::new(vec![0xff]);
    assert_eq!(utf8.utf8(), Some("é"));
    assert_eq!(invalid.utf8(), None);
    assert_ne!(utf8, decomposed);
    assert!(decomposed < utf8);

    let path = TorrentPath::new([TorrentBytes::new(b"dir".to_vec()), invalid.clone()]).unwrap();
    assert_eq!(path.components()[1], invalid);
    assert_eq!(path.utf8_components(), None);
    #[cfg(unix)]
    assert_eq!(
        path.to_path_buf().unwrap().as_os_str().as_encoded_bytes(),
        b"dir/\xff"
    );

    for unsafe_component in [
        b"".as_slice(),
        b".".as_slice(),
        b"..".as_slice(),
        b"a/b",
        b"a\\b",
        b"a\0b",
    ] {
        assert!(TorrentPath::new([TorrentBytes::new(unsafe_component.to_vec())]).is_err());
    }
}
