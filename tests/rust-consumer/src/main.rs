use btpc_core::create::{CreateOptions, Creator, NoProgress};
use btpc_core::verify::Verifier;
use btpc_core::{Canonicality, Metainfo, TorrentBytes, TorrentPath};

fn main() -> btpc_core::Result<()> {
    let bytes = b"d4:infod6:lengthi0e4:name5:empty12:piece lengthi16e6:pieces0:ee";
    let torrent = Metainfo::from_bytes(bytes)?;
    assert!(matches!(
        torrent.validate().canonicality(),
        Canonicality::Canonical
    ));
    let path = TorrentPath::new([TorrentBytes::new(b"empty".to_vec())])?;
    assert_eq!(path.utf8_components(), Some(vec!["empty"]));

    let _options = CreateOptions::builder().build()?;
    let _creator = Creator::new("payload");
    let _verifier = Verifier::new(&torrent, "payload");
    let _progress = NoProgress;
    Ok(())
}
