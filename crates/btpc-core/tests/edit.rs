use std::fs;

use btpc_core::Metainfo;
use btpc_core::bencode::OwnedValue;
use btpc_core::create::{CreateMode, CreateOptions, Creator, NoProgress};
use btpc_core::edit::MetainfoEditor;
use btpc_core::metainfo::RawMetainfo;

fn noncanonical_torrent(payload: &std::path::Path, mode: CreateMode) -> Vec<u8> {
    let options = CreateOptions::builder().mode(mode).build().unwrap();
    let canonical = Creator::new(payload)
        .options(options)
        .create(&NoProgress)
        .unwrap()
        .bytes()
        .to_vec();
    let raw = RawMetainfo::from_bytes(&canonical).unwrap();
    let info = btpc_core::bencode::parse(raw.info_bytes()).unwrap();
    let btpc_core::bencode::ValueKind::Dictionary(entries) = info.kind() else {
        unreachable!();
    };
    let mut reordered = vec![b'd'];
    for (key, value) in entries.iter().rev() {
        reordered.extend_from_slice(&raw.info_bytes()[key.span().start()..value.span().end()]);
    }
    reordered.push(b'e');
    let info_range = raw.info_span().range();
    let mut output = canonical;
    output.splice(info_range, reordered);
    output
}

fn info_bytes(metainfo: &Metainfo) -> &[u8] {
    RawMetainfo::from_bytes(metainfo.original_bytes())
        .unwrap()
        .info_bytes()
}

#[test]
fn every_top_level_edit_preserves_exact_noncanonical_info_bytes() {
    for mode in [CreateMode::V1, CreateMode::V2, CreateMode::Hybrid] {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        fs::write(&payload, b"data").unwrap();
        let bytes = noncanonical_torrent(&payload, mode);
        let parsed = Metainfo::from_bytes(&bytes).unwrap();
        let original_info = info_bytes(&parsed).to_vec();
        let original_v1 = parsed.info_hash_v1();
        let original_v2 = parsed.info_hash_v2();

        let set = MetainfoEditor::from_metainfo(&parsed)
            .unwrap()
            .trackers([vec![b"https://tracker".to_vec()]])
            .web_seeds([b"https://seed".to_vec()])
            .nodes([(b"router.example".to_vec(), 6881)])
            .comment(Some(b"comment".to_vec()))
            .created_by(Some(b"creator".to_vec()))
            .creation_date(Some(123))
            .raw_top_level(b"x-custom".to_vec(), OwnedValue::integer(7))
            .unwrap()
            .to_metainfo()
            .unwrap();
        assert_eq!(info_bytes(&set), original_info, "set {mode:?}");
        assert_eq!(set.info_hash_v1(), original_v1, "set {mode:?}");
        assert_eq!(set.info_hash_v2(), original_v2, "set {mode:?}");

        let removed = MetainfoEditor::from_metainfo(&set)
            .unwrap()
            .trackers([])
            .web_seeds([])
            .nodes([])
            .comment(None)
            .created_by(None)
            .creation_date(None)
            .to_metainfo()
            .unwrap();
        assert_eq!(info_bytes(&removed), original_info, "remove {mode:?}");
        assert_eq!(removed.info_hash_v1(), original_v1, "remove {mode:?}");
        assert_eq!(removed.info_hash_v2(), original_v2, "remove {mode:?}");
        assert!(
            removed
                .unknown_fields()
                .iter()
                .any(|field| field.key() == b"x-custom")
        );
        let canonical = Metainfo::from_bytes(&removed.to_bytes().unwrap()).unwrap();
        assert_ne!(canonical.original_bytes(), removed.original_bytes());
        if original_v1.is_some() {
            assert_ne!(canonical.info_hash_v1(), original_v1, "canonical {mode:?}");
        }
        if original_v2.is_some() {
            assert_ne!(canonical.info_hash_v2(), original_v2, "canonical {mode:?}");
        }
    }
}

#[test]
fn info_edits_canonicalize_and_recompute_every_applicable_hash() {
    for mode in [CreateMode::V1, CreateMode::V2, CreateMode::Hybrid] {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        fs::write(&payload, b"data").unwrap();
        let parsed = Metainfo::from_bytes(&noncanonical_torrent(&payload, mode)).unwrap();
        let original_v1 = parsed.info_hash_v1();
        let original_v2 = parsed.info_hash_v2();

        let edited = MetainfoEditor::from_metainfo(&parsed)
            .unwrap()
            .source(Some(b"source".to_vec()))
            .to_metainfo()
            .unwrap();

        btpc_core::bencode::validate_canonical(info_bytes(&edited)).unwrap();
        if original_v1.is_some() {
            assert_ne!(edited.info_hash_v1(), original_v1, "{mode:?}");
        }
        if original_v2.is_some() {
            assert_ne!(edited.info_hash_v2(), original_v2, "{mode:?}");
        }
    }
}

#[test]
fn hybrid_real_file_attributes_update_v1_and_v2_representations() {
    for multifile in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        let path = if multifile {
            fs::create_dir(&payload).unwrap();
            fs::write(payload.join("a"), b"a").unwrap();
            fs::write(payload.join("b"), b"b").unwrap();
            vec![b"a".to_vec()]
        } else {
            fs::write(&payload, b"data").unwrap();
            vec![b"payload".to_vec()]
        };
        let options = CreateOptions::builder()
            .mode(CreateMode::Hybrid)
            .build()
            .unwrap();
        let parsed = Metainfo::from_bytes(
            Creator::new(&payload)
                .options(options)
                .create(&NoProgress)
                .unwrap()
                .bytes(),
        )
        .unwrap();
        let edited = MetainfoEditor::from_metainfo(&parsed)
            .unwrap()
            .file_attributes(&path, b"x".to_vec())
            .unwrap()
            .to_metainfo()
            .unwrap();
        assert_eq!(
            edited
                .original_bytes()
                .windows(b"4:attr1:x".len())
                .filter(|window| *window == b"4:attr1:x")
                .count(),
            2,
            "multifile={multifile}"
        );
    }
}

#[test]
fn hybrid_padding_attributes_only_update_the_v1_padding_entry() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir(&payload).unwrap();
    fs::write(payload.join("a"), b"a").unwrap();
    fs::write(payload.join("b"), b"b").unwrap();
    let options = CreateOptions::builder()
        .mode(CreateMode::Hybrid)
        .build()
        .unwrap();
    let parsed = Metainfo::from_bytes(
        Creator::new(&payload)
            .options(options)
            .create(&NoProgress)
            .unwrap()
            .bytes(),
    )
    .unwrap();
    let padding = parsed
        .files()
        .iter()
        .find(|file| file.is_padding())
        .expect("hybrid multifile creation inserts padding");
    let edited = MetainfoEditor::from_metainfo(&parsed)
        .unwrap()
        .file_attributes(padding.path_components(), b"px".to_vec())
        .unwrap()
        .to_metainfo()
        .unwrap();
    assert_eq!(
        edited
            .original_bytes()
            .windows(b"4:attr2:px".len())
            .filter(|window| *window == b"4:attr2:px")
            .count(),
        1
    );
}

#[test]
fn top_level_edits_preserve_info_hash_and_unknown_fields() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"data").unwrap();
    let options = CreateOptions::builder()
        .trackers([vec![b"https://old".to_vec()]])
        .build()
        .unwrap();
    let result = Creator::new(&payload)
        .options(options)
        .create(&NoProgress)
        .unwrap();
    let mut root = match btpc_core::bencode::parse(result.bytes()).unwrap().kind() {
        btpc_core::bencode::ValueKind::Dictionary(_) => result.bytes().to_vec(),
        _ => unreachable!(),
    };
    let insertion = b"3:foo3:bar";
    root.splice(1..1, insertion.iter().copied());
    let parsed = Metainfo::from_bytes(&root).unwrap();
    let original_hash = parsed.info_hash_v1();

    let edited = MetainfoEditor::from_metainfo(&parsed)
        .unwrap()
        .trackers([vec![b"https://one".to_vec(), b"https://two".to_vec()]])
        .web_seeds([b"https://seed".to_vec()])
        .nodes([(b"router.example".to_vec(), 6881)])
        .comment(Some(b"comment".to_vec()))
        .created_by(Some(b"editor-test".to_vec()))
        .creation_date(Some(123))
        .to_metainfo()
        .unwrap();
    assert_eq!(edited.info_hash_v1(), original_hash);
    assert_eq!(edited.trackers()[0].len(), 2);
    assert_eq!(edited.web_seeds(), &[b"https://seed".to_vec()]);
    assert!(
        edited
            .unknown_fields()
            .iter()
            .any(|field| field.key() == b"foo")
    );
}

#[test]
fn info_edits_change_hash_and_validate_canonically() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"data").unwrap();
    let result = Creator::new(&payload).create(&NoProgress).unwrap();
    let parsed = Metainfo::from_bytes(result.bytes()).unwrap();
    let original_hash = parsed.info_hash_v1();
    let edited = MetainfoEditor::from_metainfo(&parsed)
        .unwrap()
        .private(Some(true))
        .source(Some(b"source".to_vec()))
        .to_metainfo()
        .unwrap();
    assert_ne!(edited.info_hash_v1(), original_hash);
    assert_eq!(edited.private(), Some(true));
    btpc_core::bencode::validate_canonical(&edited.to_bytes().unwrap()).unwrap();
}

#[test]
fn raw_fields_reject_reserved_keys_and_support_extensions() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"data").unwrap();
    let parsed =
        Metainfo::from_bytes(Creator::new(&payload).create(&NoProgress).unwrap().bytes()).unwrap();
    assert!(
        MetainfoEditor::from_metainfo(&parsed)
            .unwrap()
            .raw_top_level(b"announce".to_vec(), OwnedValue::bytes(b"bad".to_vec()))
            .is_err()
    );
    let edited = MetainfoEditor::from_metainfo(&parsed)
        .unwrap()
        .raw_top_level(b"x-custom".to_vec(), OwnedValue::integer(7))
        .unwrap()
        .to_metainfo()
        .unwrap();
    assert!(
        edited
            .unknown_fields()
            .iter()
            .any(|field| field.key() == b"x-custom")
    );
}

#[test]
fn file_attributes_round_trip_for_v1_and_v2_shapes() {
    for mode in [
        btpc_core::create::CreateMode::V1,
        btpc_core::create::CreateMode::V2,
    ] {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        fs::write(&payload, b"data").unwrap();
        let options = CreateOptions::builder().mode(mode).build().unwrap();
        let parsed = Metainfo::from_bytes(
            Creator::new(&payload)
                .options(options)
                .create(&NoProgress)
                .unwrap()
                .bytes(),
        )
        .unwrap();
        let edited = MetainfoEditor::from_metainfo(&parsed)
            .unwrap()
            .file_attributes(&[b"payload".to_vec()], b"x".to_vec())
            .unwrap()
            .to_metainfo()
            .unwrap();
        assert_eq!(edited.files()[0].attributes(), b"x");
    }
}

#[test]
fn multifile_v1_attributes_are_addressed_by_raw_path() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir(&payload).unwrap();
    fs::write(payload.join("a"), b"a").unwrap();
    fs::write(payload.join("b"), b"b").unwrap();
    let parsed =
        Metainfo::from_bytes(Creator::new(&payload).create(&NoProgress).unwrap().bytes()).unwrap();
    let edited = MetainfoEditor::from_metainfo(&parsed)
        .unwrap()
        .file_attributes(&[b"b".to_vec()], b"x".to_vec())
        .unwrap()
        .to_metainfo()
        .unwrap();
    assert_eq!(edited.files()[1].attributes(), b"x");
}

#[test]
fn can_start_from_owned_info_dictionary() {
    let info = OwnedValue::dictionary([
        (b"length".to_vec(), OwnedValue::integer(0)),
        (b"name".to_vec(), OwnedValue::bytes(b"empty".to_vec())),
        (b"piece length".to_vec(), OwnedValue::integer(16_384)),
        (b"pieces".to_vec(), OwnedValue::bytes(Vec::new())),
    ])
    .unwrap();
    let metainfo = MetainfoEditor::from_info(info)
        .unwrap()
        .to_metainfo()
        .unwrap();
    assert_eq!(metainfo.name(), b"empty");
}

#[derive(Clone, Debug)]
enum EditOperation {
    Comment(Option<Vec<u8>>),
    CreatedBy(Option<Vec<u8>>),
    CreationDate(Option<i64>),
    Private(Option<bool>),
    Source(Option<Vec<u8>>),
}

impl EditOperation {
    fn apply(&self, editor: MetainfoEditor) -> MetainfoEditor {
        match self {
            Self::Comment(value) => editor.comment(value.clone()),
            Self::CreatedBy(value) => editor.created_by(value.clone()),
            Self::CreationDate(value) => editor.creation_date(*value),
            Self::Private(value) => editor.private(*value),
            Self::Source(value) => editor.source(value.clone()),
        }
    }
}

fn edit_operation() -> impl proptest::strategy::Strategy<Value = EditOperation> {
    use proptest::prelude::*;

    prop_oneof![
        proptest::option::of(proptest::collection::vec(any::<u8>(), 0..16))
            .prop_map(EditOperation::Comment),
        proptest::option::of(proptest::collection::vec(any::<u8>(), 0..16))
            .prop_map(EditOperation::CreatedBy),
        proptest::option::of(any::<i64>()).prop_map(EditOperation::CreationDate),
        proptest::option::of(any::<bool>()).prop_map(EditOperation::Private),
        proptest::option::of(proptest::collection::vec(any::<u8>(), 0..16))
            .prop_map(EditOperation::Source),
    ]
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]

    #[test]
    fn generated_edit_sequences_preserve_state_and_hash_rules(
        operations in proptest::collection::vec(edit_operation(), 1..24)
    ) {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        fs::write(&payload, b"state machine payload").unwrap();
        let mut metainfo = Metainfo::from_bytes(
            Creator::new(&payload).create(&NoProgress).unwrap().bytes()
        ).unwrap();
        let original_hash = metainfo.info_hash_v1();
        let mut private = None;
        let mut source = None;

        for operation in operations {
            match &operation {
                EditOperation::Private(value) => private = *value,
                EditOperation::Source(value) => source = value.clone(),
                _ => {}
            }
            metainfo = operation
                .apply(MetainfoEditor::from_metainfo(&metainfo).unwrap())
                .to_metainfo()
                .unwrap();
            let bytes = metainfo.to_bytes().unwrap();
            btpc_core::bencode::validate_canonical(&bytes).unwrap();
            let reparsed = Metainfo::from_bytes(&bytes).unwrap();
            proptest::prop_assert_eq!(reparsed.info_hash_v1(), metainfo.info_hash_v1());
            proptest::prop_assert_eq!(
                metainfo.info_hash_v1() == original_hash,
                private.is_none() && source.is_none()
            );
            proptest::prop_assert_eq!(metainfo.original_bytes(), bytes.as_slice());
            metainfo = reparsed;
        }
    }
}
