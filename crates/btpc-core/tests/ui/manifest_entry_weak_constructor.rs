use btpc_core::create::ManifestEntry;

fn main() {
    let _ = ManifestEntry::from_snapshot("payload", vec![b"payload".to_vec()], 0, None);
    let _ = ManifestEntry::for_test("payload", vec![b"payload".to_vec()], 0);
}
