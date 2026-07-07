use btpc_core::Metainfo;

fn main() -> btpc_core::Result<()> {
    let path = std::env::args_os().nth(1).ok_or_else(|| {
        btpc_core::Error::unsupported("usage: cargo run --example inspect -- FILE.torrent")
    })?;
    let torrent = Metainfo::from_path(path)?;
    println!("mode: {:?}", torrent.mode());
    println!("files: {}", torrent.files().len());
    println!("bytes: {}", torrent.total_length());
    Ok(())
}
