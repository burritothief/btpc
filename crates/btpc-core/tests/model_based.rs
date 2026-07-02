use std::collections::BTreeMap;

use btpc_core::bencode::{OwnedValue, validate_canonical};
use btpc_core::{Metainfo, TorrentMode};
use proptest::prelude::*;
use sha1::{Digest as _, Sha1};

#[derive(Clone, Debug)]
enum ModelValue {
    Integer(i64),
    Bytes(Vec<u8>),
    List(Vec<ModelValue>),
    Dictionary(BTreeMap<Vec<u8>, ModelValue>),
}

impl ModelValue {
    fn encode(&self, output: &mut Vec<u8>) {
        match self {
            Self::Integer(value) => output.extend_from_slice(format!("i{value}e").as_bytes()),
            Self::Bytes(bytes) => {
                output.extend_from_slice(bytes.len().to_string().as_bytes());
                output.push(b':');
                output.extend_from_slice(bytes);
            }
            Self::List(values) => {
                output.push(b'l');
                for value in values {
                    value.encode(output);
                }
                output.push(b'e');
            }
            Self::Dictionary(entries) => {
                output.push(b'd');
                for (key, value) in entries {
                    Self::Bytes(key.clone()).encode(output);
                    value.encode(output);
                }
                output.push(b'e');
            }
        }
    }

    fn to_owned(&self) -> OwnedValue {
        match self {
            Self::Integer(value) => OwnedValue::integer(*value),
            Self::Bytes(bytes) => OwnedValue::bytes(bytes.clone()),
            Self::List(values) => OwnedValue::list(values.iter().map(Self::to_owned)),
            Self::Dictionary(entries) => OwnedValue::dictionary(
                entries
                    .iter()
                    .map(|(key, value)| (key.clone(), value.to_owned())),
            )
            .unwrap(),
        }
    }
}

fn model_value() -> impl Strategy<Value = ModelValue> {
    let leaf = prop_oneof![
        any::<i64>().prop_map(ModelValue::Integer),
        proptest::collection::vec(any::<u8>(), 0..24).prop_map(ModelValue::Bytes),
    ];
    leaf.prop_recursive(3, 64, 8, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 0..6).prop_map(ModelValue::List),
            proptest::collection::btree_map(
                proptest::collection::vec(any::<u8>(), 0..10),
                inner,
                0..6,
            )
            .prop_map(ModelValue::Dictionary),
        ]
    })
}

fn v1_model(name: &[u8], payload: &[u8], piece_length: usize) -> (Vec<u8>, Vec<u8>) {
    let pieces = payload
        .chunks(piece_length)
        .flat_map(|piece| Sha1::digest(piece).to_vec())
        .collect::<Vec<_>>();
    let info = ModelValue::Dictionary(BTreeMap::from([
        (
            b"length".to_vec(),
            ModelValue::Integer(i64::try_from(payload.len()).unwrap()),
        ),
        (b"name".to_vec(), ModelValue::Bytes(name.to_vec())),
        (
            b"piece length".to_vec(),
            ModelValue::Integer(i64::try_from(piece_length).unwrap()),
        ),
        (b"pieces".to_vec(), ModelValue::Bytes(pieces)),
    ]));
    let mut info_bytes = Vec::new();
    info.encode(&mut info_bytes);
    let root = ModelValue::Dictionary(BTreeMap::from([(b"info".to_vec(), info)]));
    let mut bytes = Vec::new();
    root.encode(&mut bytes);
    (bytes, info_bytes)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn independent_bencode_model_matches_owned_encoder(value in model_value()) {
        let mut expected = Vec::new();
        value.encode(&mut expected);
        let actual = value.to_owned().to_vec().unwrap();
        prop_assert_eq!(&actual, &expected);
        validate_canonical(&actual).unwrap();
    }

    #[test]
    fn independent_v1_model_matches_typed_metainfo(
        name in proptest::collection::vec(1_u8..=127, 1..16)
            .prop_filter("safe name", |name| !name.contains(&b'/') && !name.contains(&b'\\') && name != b"." && name != b".."),
        payload in proptest::collection::vec(any::<u8>(), 0..100_000),
        exponent in 0_u32..5,
    ) {
        let piece_length = 16_384_usize << exponent;
        let (bytes, info_bytes) = v1_model(&name, &payload, piece_length);
        let torrent = Metainfo::from_bytes(&bytes).unwrap();
        prop_assert_eq!(torrent.mode(), TorrentMode::V1);
        prop_assert_eq!(torrent.name(), name.as_slice());
        prop_assert_eq!(torrent.total_length(), payload.len() as u64);
        prop_assert_eq!(torrent.piece_length(), piece_length as u64);
        prop_assert_eq!(torrent.piece_count(), payload.len().div_ceil(piece_length) as u64);
        let info_hash = torrent.info_hash_v1().unwrap();
        let expected_hash = Sha1::digest(&info_bytes);
        prop_assert_eq!(info_hash.as_bytes(), &expected_hash[..]);
        prop_assert_eq!(torrent.to_bytes().unwrap(), bytes);
    }
}
