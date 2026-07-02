#![no_main]

use btpc_core::Metainfo;
use btpc_core::magnet::MagnetOptions;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(torrent) = Metainfo::from_bytes(data) else {
        return;
    };
    let default = torrent.magnet(&MagnetOptions::default());
    assert!(default.starts_with("magnet:?xt="));
    let minimal = torrent.magnet(
        &MagnetOptions::builder()
            .display_name(false)
            .trackers(false)
            .web_seeds(false)
            .build(),
    );
    assert!(minimal.starts_with("magnet:?xt="));
    assert!(!minimal.contains("&dn="));
    assert!(!minimal.contains("&tr="));
    assert!(!minimal.contains("&ws="));
});
