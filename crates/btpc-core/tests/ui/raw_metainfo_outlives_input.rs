use btpc_core::metainfo::RawMetainfo;

fn invalid() -> RawMetainfo<'static> {
    let bytes = Vec::from(&b"d4:infodee"[..]);
    RawMetainfo::from_bytes(&bytes).unwrap()
}

fn main() {
    let _ = invalid();
}
