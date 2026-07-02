use std::fs;

use btpc_core::create::{CreateMode, CreateOptions, Creator, HashThreads, NoProgress, PieceLength};
use proptest::prelude::*;
use tempfile::TempDir;

fn create_tree(files: &[Vec<u8>]) -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("tree");
    fs::create_dir(&root).unwrap();
    for (index, bytes) in files.iter().enumerate() {
        fs::write(root.join(format!("file-{index:03}")), bytes).unwrap();
    }
    (temp, root)
}

fn create(root: &std::path::Path, mode: CreateMode, threads: usize) -> Vec<u8> {
    Creator::new(root)
        .options(
            CreateOptions::builder()
                .mode(mode)
                .piece_length(PieceLength::Exact(16 * 1024))
                .hash_threads(HashThreads::Exact(threads))
                .build()
                .unwrap(),
        )
        .create(&NoProgress)
        .unwrap()
        .bytes()
        .to_vec()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn generated_v2_and_hybrid_trees_match_sequential_oracles(
        files in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..40_000), 1..12),
    ) {
        let (_temp, root) = create_tree(&files);
        for mode in [CreateMode::V2, CreateMode::Hybrid] {
            let sequential = create(&root, mode, 1);
            let parallel = create(&root, mode, 2);
            prop_assert_eq!(parallel, sequential);
        }
    }
}
