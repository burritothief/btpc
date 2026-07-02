#![no_main]

use btpc_core::Metainfo;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(torrent) = Metainfo::from_bytes(data) else {
        return;
    };
    assert!(torrent.validate().is_valid());
    assert_eq!(torrent.original_bytes(), data);
    let _ = torrent.files();
    let _ = torrent.trackers();
    let _ = torrent.web_seeds();
    let _ = torrent.unknown_fields();
});
