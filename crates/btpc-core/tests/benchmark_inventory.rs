use std::fs;

#[test]
fn criterion_inventory_covers_required_baseline_kernels() {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/benches/core.rs"))
        .expect("benchmark source must exist");
    for benchmark in [
        "bencode_parse",
        "bencode_encode",
        "manifest_sort",
        "manifest_scan",
        "v1_piece_hashing",
        "v2_merkle_hashing",
        "v2_file_tree_creation",
    ] {
        assert!(
            source.contains(benchmark),
            "missing required benchmark {benchmark}"
        );
    }
    assert!(source.contains("black_box"));
    assert!(source.contains("iter_batched"));

    let repository = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let profiling = fs::read_to_string(format!("{repository}/benches/profiling.md"))
        .expect("profiling documentation must exist");
    assert!(profiling.contains("xcrun xctrace"));
    assert!(profiling.contains("perf record"));
    let template = fs::read_to_string(format!("{repository}/benches/baseline-template.md"))
        .expect("baseline template must exist");
    assert!(template.contains("Peak RSS"));
    assert!(template.contains("Validation"));
}
