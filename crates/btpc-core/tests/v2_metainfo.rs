use btpc_core::ErrorCategory;
use btpc_core::bencode::OwnedValue;
use btpc_core::metainfo::{RawMetainfo, TorrentMode, V2Metainfo};
use sha2::{Digest as _, Sha256};

// Spec: META-V2-001

macro_rules! dictionary {
    ([$(($key:expr, $value:expr $(,)?)),* $(,)?]) => {
        OwnedValue::dictionary([$(($key.as_slice().to_vec(), $value)),*]).unwrap()
    };
}

fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

fn root_from_piece_hashes(hashes: &[[u8; 32]]) -> [u8; 32] {
    let mut layer = hashes.to_vec();
    let target = layer.len().next_power_of_two();
    layer.resize(target, [0; 32]);
    while layer.len() > 1 {
        layer = layer
            .chunks_exact(2)
            .map(|pair| hash_pair(pair[0], pair[1]))
            .collect();
    }
    layer[0]
}

fn leaf(length: i64, root: Option<[u8; 32]>, attr: Option<&[u8]>) -> OwnedValue {
    let mut entries = vec![(b"length".to_vec(), OwnedValue::integer(length))];
    if let Some(root) = root {
        entries.push((b"pieces root".to_vec(), OwnedValue::bytes(root.to_vec())));
    }
    if let Some(attr) = attr {
        entries.push((b"attr".to_vec(), OwnedValue::bytes(attr.to_vec())));
    }
    OwnedValue::dictionary(entries).unwrap()
}

fn tree_file(path: &[&[u8]], properties: OwnedValue) -> OwnedValue {
    let mut node = dictionary!([(b"", properties)]);
    for component in path.iter().rev() {
        node = OwnedValue::dictionary([(component.to_vec(), node)]).unwrap();
    }
    node
}

fn merge_trees(left: OwnedValue, right: OwnedValue) -> OwnedValue {
    let (OwnedValue::Dictionary(mut left), OwnedValue::Dictionary(right)) = (left, right) else {
        panic!("tree nodes are dictionaries");
    };
    for (key, value) in right {
        if let Some(existing) = left.remove(&key) {
            left.insert(key, merge_trees(existing, value));
        } else {
            left.insert(key, value);
        }
    }
    OwnedValue::Dictionary(left)
}

fn torrent(info: OwnedValue, layers: OwnedValue) -> Vec<u8> {
    dictionary!([(b"info", info), (b"piece layers", layers)])
        .to_vec()
        .unwrap()
}

fn v2_info(tree: OwnedValue, piece_length: i64) -> OwnedValue {
    dictionary!([
        (b"file tree", tree),
        (b"meta version", OwnedValue::integer(2)),
        (b"name", OwnedValue::bytes(b"root".to_vec())),
        (b"piece length", OwnedValue::integer(piece_length)),
    ])
}

#[test]
fn validates_single_multifile_and_empty_v2_files() {
    let root_a = [1; 32];
    let tree = merge_trees(
        tree_file(&[b"a"], leaf(10, Some(root_a), Some(b"x"))),
        tree_file(&[b"dir", b"empty"], leaf(0, None, None)),
    );
    let bytes = torrent(v2_info(tree, 16_384), dictionary!([]));
    let raw = RawMetainfo::from_bytes(&bytes).unwrap();
    let v2 = V2Metainfo::from_raw(&raw).unwrap();

    assert_eq!(v2.mode(), TorrentMode::V2);
    assert_eq!(v2.files().len(), 2);
    assert_eq!(v2.files()[0].path_components(), &[&b"a"[..]]);
    assert_eq!(v2.files()[0].pieces_root(), Some(&root_a));
    assert_eq!(v2.files()[0].attributes(), b"x");
    assert_eq!(v2.files()[1].length(), 0);
    assert_eq!(v2.files()[1].pieces_root(), None);
}

#[test]
fn validates_piece_layer_membership_length_and_root() {
    let piece_hashes = [[2; 32], [3; 32], [4; 32]];
    let root = root_from_piece_hashes(&piece_hashes);
    let layer_bytes: Vec<u8> = piece_hashes.into_iter().flatten().collect();
    let tree = tree_file(&[b"large"], leaf(40_000, Some(root), None));
    let bytes = torrent(
        v2_info(tree, 16_384),
        OwnedValue::dictionary([(root.to_vec(), OwnedValue::bytes(layer_bytes))]).unwrap(),
    );
    let raw = RawMetainfo::from_bytes(&bytes).unwrap();
    let v2 = V2Metainfo::from_raw(&raw).unwrap();
    assert_eq!(v2.files()[0].piece_layer().unwrap().len(), 96);

    for layers in [
        dictionary!([]),
        OwnedValue::dictionary([(root.to_vec(), OwnedValue::bytes(vec![0; 64]))]).unwrap(),
        OwnedValue::dictionary([(root.to_vec(), OwnedValue::bytes(vec![9; 96]))]).unwrap(),
        OwnedValue::dictionary([([8; 32].to_vec(), OwnedValue::bytes(vec![0; 32]))]).unwrap(),
    ] {
        let bytes = torrent(
            v2_info(
                tree_file(&[b"large"], leaf(40_000, Some(root), None)),
                16_384,
            ),
            layers,
        );
        let raw = RawMetainfo::from_bytes(&bytes).unwrap();
        assert_eq!(
            V2Metainfo::from_raw(&raw).unwrap_err().category(),
            ErrorCategory::Metainfo
        );
    }
}

#[test]
fn rejects_invalid_piece_lengths_roots_and_tree_shapes() {
    let valid_tree = tree_file(&[b"a"], leaf(1, Some([1; 32]), None));
    for info in [
        dictionary!([
            (b"file tree", valid_tree.clone()),
            (b"meta version", OwnedValue::integer(3)),
            (b"name", OwnedValue::bytes(b"root".to_vec())),
            (b"piece length", OwnedValue::integer(16_384)),
        ]),
        v2_info(valid_tree.clone(), 8_192),
        v2_info(valid_tree.clone(), 20_000),
        v2_info(
            tree_file(
                &[b"a"],
                dictionary!([
                    (b"length", OwnedValue::integer(1)),
                    (b"pieces root", OwnedValue::bytes(vec![1; 31])),
                ]),
            ),
            16_384,
        ),
        v2_info(tree_file(&[b"a"], leaf(0, Some([1; 32]), None)), 16_384),
        v2_info(tree_file(&[b"a"], leaf(1, None, None)), 16_384),
        v2_info(dictionary!([(b"", leaf(1, Some([1; 32]), None))]), 16_384),
        v2_info(
            OwnedValue::dictionary([(
                b"a".to_vec(),
                dictionary!([
                    (b"", leaf(1, Some([1; 32]), None)),
                    (b"child", dictionary!([]))
                ]),
            )])
            .unwrap(),
            16_384,
        ),
    ] {
        let bytes = torrent(info, dictionary!([]));
        let raw = RawMetainfo::from_bytes(&bytes).unwrap();
        assert_eq!(
            V2Metainfo::from_raw(&raw).unwrap_err().category(),
            ErrorCategory::Metainfo
        );
    }
}

#[test]
fn classifies_matching_hybrid_and_rejects_mismatch_or_bad_padding() {
    let root_a = [1; 32];
    let root_b = [2; 32];
    let tree = merge_trees(
        tree_file(&[b"a"], leaf(3, Some(root_a), None)),
        tree_file(&[b"b"], leaf(4, Some(root_b), None)),
    );
    let files = OwnedValue::list([
        dictionary!([
            (b"length", OwnedValue::integer(3)),
            (
                b"path",
                OwnedValue::list([OwnedValue::bytes(b"a".to_vec())])
            ),
        ]),
        dictionary!([
            (b"attr", OwnedValue::bytes(b"p".to_vec())),
            (b"length", OwnedValue::integer(16_381)),
            (
                b"path",
                OwnedValue::list([
                    OwnedValue::bytes(b".pad".to_vec()),
                    OwnedValue::bytes(b"3-16381".to_vec())
                ])
            ),
        ]),
        dictionary!([
            (b"length", OwnedValue::integer(4)),
            (
                b"path",
                OwnedValue::list([OwnedValue::bytes(b"b".to_vec())])
            ),
        ]),
    ]);
    let hybrid_info = dictionary!([
        (b"file tree", tree.clone()),
        (b"files", files),
        (b"meta version", OwnedValue::integer(2)),
        (b"name", OwnedValue::bytes(b"root".to_vec())),
        (b"piece length", OwnedValue::integer(16_384)),
        (b"pieces", OwnedValue::bytes(vec![0; 40])),
    ]);
    let bytes = torrent(hybrid_info, dictionary!([]));
    let raw = RawMetainfo::from_bytes(&bytes).unwrap();
    assert_eq!(
        V2Metainfo::from_raw(&raw).unwrap().mode(),
        TorrentMode::Hybrid
    );

    for (second_path, pad_length) in [(b"c".as_slice(), 16_381), (b"b".as_slice(), 16_380)] {
        let files = OwnedValue::list([
            dictionary!([
                (b"length", OwnedValue::integer(3)),
                (
                    b"path",
                    OwnedValue::list([OwnedValue::bytes(b"a".to_vec())])
                )
            ]),
            dictionary!([
                (b"attr", OwnedValue::bytes(b"p".to_vec())),
                (b"length", OwnedValue::integer(pad_length)),
                (
                    b"path",
                    OwnedValue::list([OwnedValue::bytes(b".pad".to_vec())])
                )
            ]),
            dictionary!([
                (b"length", OwnedValue::integer(4)),
                (
                    b"path",
                    OwnedValue::list([OwnedValue::bytes(second_path.to_vec())])
                )
            ]),
        ]);
        let info = dictionary!([
            (b"file tree", tree.clone()),
            (b"files", files),
            (b"meta version", OwnedValue::integer(2)),
            (b"name", OwnedValue::bytes(b"root".to_vec())),
            (b"piece length", OwnedValue::integer(16_384)),
            (b"pieces", OwnedValue::bytes(vec![0; 40])),
        ]);
        let bytes = torrent(info, dictionary!([]));
        let raw = RawMetainfo::from_bytes(&bytes).unwrap();
        assert_eq!(
            V2Metainfo::from_raw(&raw).unwrap_err().category(),
            ErrorCategory::Metainfo
        );
    }
}

#[test]
fn rejects_fake_misplaced_consecutive_and_unneeded_padding() {
    let root_a = [1; 32];
    let root_b = [2; 32];
    let tree = merge_trees(
        tree_file(&[b"a"], leaf(3, Some(root_a), None)),
        tree_file(&[b"b"], leaf(4, Some(root_b), None)),
    );
    let real = |path: &[u8], length| {
        dictionary!([
            (b"length", OwnedValue::integer(length)),
            (
                b"path",
                OwnedValue::list([OwnedValue::bytes(path.to_vec())]),
            ),
        ])
    };
    let pad = |path: &[u8], length| {
        dictionary!([
            (b"attr", OwnedValue::bytes(b"p".to_vec())),
            (b"length", OwnedValue::integer(length)),
            (
                b"path",
                OwnedValue::list([
                    OwnedValue::bytes(b".pad".to_vec()),
                    OwnedValue::bytes(path.to_vec()),
                ]),
            ),
        ])
    };
    let cases = [
        vec![pad(b"0-16384", 16_384), real(b"a", 3), real(b"b", 4)],
        vec![real(b"a", 3), pad(b"3-16381", 16_381)],
        vec![
            real(b"a", 3),
            pad(b"3-16381", 16_381),
            pad(b"16384-1", 1),
            real(b"b", 4),
        ],
        vec![real(b"a", 3), pad(b"4-16381", 16_381), real(b"b", 4)],
        vec![real(b"a", 3), pad(b"3-0", 0), real(b"b", 4)],
        vec![
            dictionary!([
                (b"attr", OwnedValue::bytes(b"p".to_vec())),
                (b"length", OwnedValue::integer(3)),
                (
                    b"path",
                    OwnedValue::list([OwnedValue::bytes(b"a".to_vec())]),
                ),
            ]),
            real(b"b", 4),
        ],
    ];
    for files in cases {
        let info = dictionary!([
            (b"file tree", tree.clone()),
            (b"files", OwnedValue::list(files)),
            (b"meta version", OwnedValue::integer(2)),
            (b"name", OwnedValue::bytes(b"root".to_vec())),
            (b"piece length", OwnedValue::integer(16_384)),
            (b"pieces", OwnedValue::bytes(vec![0; 40])),
        ]);
        let bytes = torrent(info, dictionary!([]));
        let raw = RawMetainfo::from_bytes(&bytes).unwrap();
        assert_eq!(
            V2Metainfo::from_raw(&raw).unwrap_err().category(),
            ErrorCategory::Metainfo
        );
    }

    let aligned_tree = merge_trees(
        tree_file(&[b"a"], leaf(16_384, Some(root_a), None)),
        tree_file(&[b"b"], leaf(4, Some(root_b), None)),
    );
    let info = dictionary!([
        (b"file tree", aligned_tree),
        (
            b"files",
            OwnedValue::list([real(b"a", 16_384), pad(b"16384-1", 1), real(b"b", 4),]),
        ),
        (b"meta version", OwnedValue::integer(2)),
        (b"name", OwnedValue::bytes(b"root".to_vec())),
        (b"piece length", OwnedValue::integer(16_384)),
        (b"pieces", OwnedValue::bytes(vec![0; 40])),
    ]);
    let bytes = torrent(info, dictionary!([]));
    let raw = RawMetainfo::from_bytes(&bytes).unwrap();
    assert_eq!(
        V2Metainfo::from_raw(&raw).unwrap_err().category(),
        ErrorCategory::Metainfo
    );
}
