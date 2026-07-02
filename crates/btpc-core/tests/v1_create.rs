use std::fs;
use std::sync::{Arc, Barrier};
use std::time::Duration;

use btpc_core::Metainfo;
use btpc_core::create::{
    CreateOptions, Creator, DurabilityPolicy, NoProgress, OverwritePolicy, PIECE_LENGTH_POLICY_ID,
    PieceLength,
};
use tempfile::TempDir;

#[test]
fn creates_canonical_reproducible_single_file_with_metadata() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload.bin");
    fs::write(&payload, b"abcdefghij").unwrap();
    let options = CreateOptions::builder()
        .piece_length(PieceLength::Exact(8 * 1024))
        .trackers([
            vec![b"https://one".to_vec()],
            vec![b"https://two".to_vec(), b"https://three".to_vec()],
        ])
        .web_seeds([b"https://seed".to_vec()])
        .nodes([(b"router.example".to_vec(), 6881)])
        .private(true)
        .source(b"source".to_vec())
        .comment(b"comment".to_vec())
        .created_by(b"btpc-test".to_vec())
        .build()
        .unwrap();

    let first = Creator::new(&payload)
        .options(options.clone())
        .create(&NoProgress)
        .unwrap();
    let second = Creator::new(&payload)
        .options(options)
        .create(&NoProgress)
        .unwrap();
    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(first.info_hash_v1(), second.info_hash_v1());
    assert_eq!(first.file_count(), 1);
    assert_eq!(first.payload_bytes(), 10);
    assert_eq!(first.piece_count(), 1);
    assert_eq!(first.piece_length(), 8 * 1024);
    assert_eq!(first.piece_length_policy(), None);
    assert!(first.metrics().scan() >= Duration::ZERO);
    assert!(first.metrics().hash() >= Duration::ZERO);
    assert!(first.metrics().serialize() >= Duration::ZERO);

    let parsed = Metainfo::from_bytes(first.bytes()).unwrap();
    assert_eq!(parsed.name(), b"payload.bin");
    assert_eq!(parsed.trackers().len(), 2);
    assert_eq!(parsed.web_seeds(), &[b"https://seed".to_vec()]);
    assert_eq!(parsed.private(), Some(true));
    assert_eq!(parsed.info_hash_v1(), first.info_hash_v1());
    assert!(
        first
            .bytes()
            .windows(b"creation date".len())
            .all(|window| window != b"creation date")
    );
}

#[test]
fn creates_multifile_with_automatic_policy_and_sorted_paths() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir_all(payload.join("dir")).unwrap();
    fs::write(payload.join("z"), b"z").unwrap();
    fs::write(payload.join("dir/a"), b"abc").unwrap();

    let result = Creator::new(&payload).create(&NoProgress).unwrap();
    assert_eq!(result.piece_length_policy(), Some(PIECE_LENGTH_POLICY_ID));
    assert_eq!(result.piece_length(), 16 * 1024);
    let parsed = Metainfo::from_bytes(result.bytes()).unwrap();
    assert_eq!(
        parsed.files()[0].path_components(),
        &[b"dir".to_vec(), b"a".to_vec()]
    );
    assert_eq!(parsed.files()[1].path_components(), &[b"z".to_vec()]);
}

#[test]
fn atomic_write_respects_overwrite_policy_and_leaves_no_temp_files() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("payload.torrent");
    fs::write(&payload, b"data").unwrap();

    let creator = Creator::new(&payload);
    let result = creator
        .create_to_path(&output, OverwritePolicy::Deny, &NoProgress)
        .unwrap();
    assert_eq!(fs::read(&output).unwrap(), result.bytes());
    assert!(
        creator
            .create_to_path(&output, OverwritePolicy::Deny, &NoProgress)
            .is_err()
    );
    let replaced = creator
        .create_to_path(&output, OverwritePolicy::Replace, &NoProgress)
        .unwrap();
    assert_eq!(fs::read(&output).unwrap(), replaced.bytes());
    assert_eq!(
        fs::read_dir(temp.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".btpc-tmp"))
            .count(),
        0
    );
}

#[test]
fn concurrent_no_clobber_publication_has_exactly_one_winner() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("payload.torrent");
    fs::write(&payload, b"race").unwrap();
    let barrier = Arc::new(Barrier::new(8));
    let handles = (0..8)
        .map(|_| {
            let payload = payload.clone();
            let output = output.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                Creator::new(payload).create_to_path(output, OverwritePolicy::Deny, &NoProgress)
            })
        })
        .collect::<Vec<_>>();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|result| result.is_err()).count(), 7);
    let bytes = fs::read(&output).unwrap();
    assert!(btpc_core::Metainfo::from_bytes(&bytes).is_ok());
    assert_eq!(temporary_count(temp.path()), 0);
}

#[test]
fn durable_publication_and_symlink_destinations_follow_policy() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let output = temp.path().join("payload.torrent");
    fs::write(&payload, b"durable").unwrap();
    Creator::new(&payload)
        .create_to_path_with_durability(
            &output,
            OverwritePolicy::Deny,
            DurabilityPolicy::FileAndDirectory,
            &NoProgress,
        )
        .unwrap();
    assert_eq!(temporary_count(temp.path()), 0);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        use std::os::unix::fs::symlink;

        fs::set_permissions(&output, fs::Permissions::from_mode(0o640)).unwrap();
        Creator::new(&payload)
            .create_to_path(&output, OverwritePolicy::Replace, &NoProgress)
            .unwrap();
        assert_eq!(
            fs::metadata(&output).unwrap().permissions().mode() & 0o777,
            0o640
        );

        let target = temp.path().join("target");
        let link = temp.path().join("link.torrent");
        fs::write(&target, b"unchanged").unwrap();
        symlink(&target, &link).unwrap();
        assert!(
            Creator::new(&payload)
                .create_to_path(&link, OverwritePolicy::Deny, &NoProgress)
                .is_err()
        );
        assert_eq!(fs::read(&target).unwrap(), b"unchanged");
        Creator::new(&payload)
            .create_to_path(&link, OverwritePolicy::Replace, &NoProgress)
            .unwrap();
        assert!(
            !fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read(&target).unwrap(), b"unchanged");
        assert_eq!(temporary_count(temp.path()), 0);
    }
}

fn temporary_count(directory: &std::path::Path) -> usize {
    fs::read_dir(directory)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().contains(".btpc-tmp"))
        .count()
}

#[test]
// Spec: CREATE-CREATOR-001
fn creator_identity_defaults_overrides_and_omits_without_changing_info_hash() {
    use btpc_core::metainfo::RawMetainfo;

    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"creator").unwrap();
    let create = |options| {
        Creator::new(&payload)
            .options(options)
            .create(&NoProgress)
            .unwrap()
    };
    let default = create(CreateOptions::builder().build().unwrap());
    let explicit = create(
        CreateOptions::builder()
            .created_by("custom/π".as_bytes().to_vec())
            .build()
            .unwrap(),
    );
    let omitted = create(CreateOptions::builder().omit_created_by().build().unwrap());
    let creator = |bytes: &[u8]| {
        RawMetainfo::from_bytes(bytes)
            .unwrap()
            .created_by()
            .and_then(btpc_core::bencode::Value::as_bytes)
            .map(ToOwned::to_owned)
    };
    assert_eq!(creator(default.bytes()), Some(b"btpc/0.1.0".to_vec()));
    assert_eq!(
        creator(explicit.bytes()),
        Some("custom/π".as_bytes().to_vec())
    );
    assert_eq!(creator(omitted.bytes()), None);
    assert_eq!(default.info_hash_v1(), explicit.info_hash_v1());
    assert_eq!(default.info_hash_v1(), omitted.info_hash_v1());
}
