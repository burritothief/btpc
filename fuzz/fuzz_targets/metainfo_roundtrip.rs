#![no_main]

use btpc_core::Metainfo;
use btpc_core::bencode::validate_canonical;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(torrent) = Metainfo::from_bytes(data) else {
        return;
    };
    let encoded = torrent.to_bytes().expect("Vec writes cannot fail");
    validate_canonical(&encoded).expect("typed serialization is canonical");
    let reparsed = Metainfo::from_bytes(&encoded).expect("typed serialization reparses");
    assert_eq!(torrent.mode(), reparsed.mode());
    assert_eq!(torrent.name(), reparsed.name());
    assert_eq!(torrent.piece_length(), reparsed.piece_length());
    assert_eq!(torrent.total_length(), reparsed.total_length());
    let mut files = torrent.files().to_vec();
    let mut reparsed_files = reparsed.files().to_vec();
    files.sort_by(|left, right| left.path_components().cmp(right.path_components()));
    reparsed_files.sort_by(|left, right| left.path_components().cmp(right.path_components()));
    assert_eq!(files, reparsed_files);
    assert_eq!(torrent.trackers(), reparsed.trackers());
    assert_eq!(torrent.web_seeds(), reparsed.web_seeds());
    assert_eq!(torrent.private(), reparsed.private());
    assert_eq!(
        encoded,
        reparsed.to_bytes().expect("canonical bytes re-encode")
    );
});
