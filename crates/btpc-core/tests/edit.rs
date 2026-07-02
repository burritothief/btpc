use std::fs;

use btpc_core::Metainfo;
use btpc_core::bencode::OwnedValue;
use btpc_core::create::{CreateOptions, Creator, NoProgress};
use btpc_core::edit::MetainfoEditor;

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
