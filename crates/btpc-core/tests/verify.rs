use std::fs;

use btpc_core::Metainfo;
use btpc_core::create::{CreateMode, CreateOptions, Creator, NoProgress, PieceLength};
use btpc_core::verify::{ExtraFilePolicy, MismatchKind, MismatchMode, Verifier, VerifyOptions};

// Spec: VERIFY-PATH-001
// Spec: VERIFY-HASH-001
// Spec: VERIFY-REPORT-001

fn torrent(payload: &std::path::Path, mode: CreateMode) -> Metainfo {
    let options = CreateOptions::builder()
        .mode(mode)
        .piece_length(PieceLength::Exact(16_384))
        .build()
        .unwrap();
    let result = Creator::new(payload)
        .options(options)
        .create(&NoProgress)
        .unwrap();
    Metainfo::from_bytes(result.bytes()).unwrap()
}

#[test]
fn verifies_valid_payloads_in_every_hash_domain() {
    for mode in [CreateMode::V1, CreateMode::V2, CreateMode::Hybrid] {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        fs::create_dir(&payload).unwrap();
        fs::write(payload.join("a"), vec![1_u8; 20_000]).unwrap();
        fs::write(payload.join("b"), vec![2_u8; 20_000]).unwrap();
        let metainfo = torrent(&payload, mode);
        let report = Verifier::new(&metainfo, &payload)
            .verify(&NoProgress)
            .unwrap();
        assert!(report.is_valid(), "{mode:?}: {:?}", report.mismatches());
        assert!(report.mismatches().is_empty());
    }
}

#[test]
fn reports_missing_wrong_size_extra_and_hash_mismatches_deterministically() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir(&payload).unwrap();
    fs::write(payload.join("a"), vec![1_u8; 20_000]).unwrap();
    fs::write(payload.join("b"), vec![2_u8; 20_000]).unwrap();
    fs::write(payload.join("c"), vec![3_u8; 20_000]).unwrap();
    let metainfo = torrent(&payload, CreateMode::Hybrid);

    fs::remove_file(payload.join("a")).unwrap();
    fs::write(payload.join("b"), b"short").unwrap();
    let mut changed = vec![3_u8; 20_000];
    changed[10_000] ^= 1;
    fs::write(payload.join("c"), changed).unwrap();
    fs::write(payload.join("extra"), b"extra").unwrap();

    let options = VerifyOptions::builder()
        .mismatch_mode(MismatchMode::CollectAll)
        .extra_files(ExtraFilePolicy::Report)
        .build();
    let report = Verifier::new(&metainfo, &payload)
        .options(options)
        .verify(&NoProgress)
        .unwrap();
    let kinds = report
        .mismatches()
        .iter()
        .map(btpc_core::verify::Mismatch::kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&MismatchKind::Missing));
    assert!(kinds.contains(&MismatchKind::WrongSize));
    assert!(kinds.contains(&MismatchKind::Extra));
    assert!(kinds.contains(&MismatchKind::V2Hash));
    assert!(
        report
            .mismatches()
            .windows(2)
            .all(|pair| pair[0].sort_key() <= pair[1].sort_key())
    );
}

#[test]
fn reports_first_middle_and_last_v1_piece_mutations() {
    for (offset, expected_piece) in [(0, 0), (20_000, 1), (49_999, 3)] {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        let original = vec![5_u8; 50_000];
        fs::write(&payload, &original).unwrap();
        let metainfo = torrent(&payload, CreateMode::V1);
        let mut changed = original;
        changed[offset] ^= 1;
        fs::write(&payload, changed).unwrap();
        let report = Verifier::new(&metainfo, &payload)
            .verify(&NoProgress)
            .unwrap();
        assert_eq!(report.mismatches().len(), 1);
        assert_eq!(report.mismatches()[0].kind(), MismatchKind::V1Hash);
        assert_eq!(report.mismatches()[0].piece(), Some(expected_piece));
    }
}

#[test]
fn reports_v2_file_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, vec![8_u8; 40_000]).unwrap();
    let metainfo = torrent(&payload, CreateMode::V2);
    let mut changed = vec![8_u8; 40_000];
    changed[20_000] ^= 1;
    fs::write(&payload, changed).unwrap();
    let report = Verifier::new(&metainfo, &payload)
        .verify(&NoProgress)
        .unwrap();
    assert_eq!(report.mismatches()[0].kind(), MismatchKind::V2Hash);
}

#[test]
fn missing_payload_root_is_a_reported_mismatch() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    fs::write(&source, b"data").unwrap();
    let metainfo = torrent(&source, CreateMode::V2);
    let report = Verifier::new(&metainfo, temp.path().join("missing"))
        .verify(&NoProgress)
        .unwrap();
    assert_eq!(report.mismatches()[0].kind(), MismatchKind::Missing);
}

#[test]
fn fail_fast_stops_after_first_mismatch_and_cancellation_propagates() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir(&payload).unwrap();
    fs::write(payload.join("a"), b"a").unwrap();
    fs::write(payload.join("b"), b"b").unwrap();
    let metainfo = torrent(&payload, CreateMode::V1);
    fs::remove_file(payload.join("a")).unwrap();
    fs::remove_file(payload.join("b")).unwrap();

    let report = Verifier::new(&metainfo, &payload)
        .options(
            VerifyOptions::builder()
                .mismatch_mode(MismatchMode::FailFast)
                .build(),
        )
        .verify(&NoProgress)
        .unwrap();
    assert_eq!(report.mismatches().len(), 1);

    let cancellation = btpc_core::create::CancellationToken::new();
    cancellation.cancel();
    assert!(
        Verifier::new(&metainfo, &payload)
            .cancellation(cancellation)
            .verify(&NoProgress)
            .is_err()
    );
}

#[cfg(unix)]
#[test]
fn rejects_symlink_escape_and_never_follows_it_by_default() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("file"), b"safe").unwrap();
    let metainfo = torrent(&source, CreateMode::V2);

    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("file"), b"safe").unwrap();
    let payload = temp.path().join("payload");
    symlink(&outside, &payload).unwrap();
    let report = Verifier::new(&metainfo, &payload)
        .verify(&NoProgress)
        .unwrap();
    assert_eq!(report.mismatches()[0].kind(), MismatchKind::UnsafePath);

    let source = temp.path().join("nested-source");
    fs::create_dir_all(source.join("nested")).unwrap();
    fs::write(source.join("nested/file"), b"safe").unwrap();
    let nested_metainfo = torrent(&source, CreateMode::V2);
    let nested_payload = temp.path().join("nested-payload");
    fs::create_dir(&nested_payload).unwrap();
    symlink(&outside, nested_payload.join("nested")).unwrap();
    let nested_report = Verifier::new(&nested_metainfo, &nested_payload)
        .verify(&NoProgress)
        .unwrap();
    assert_eq!(
        nested_report.mismatches()[0].kind(),
        MismatchKind::UnsafePath
    );
}

#[test]
fn progress_is_monotonic_and_can_cancel_active_verification() {
    use btpc_core::create::{CancellationToken, HashProgress, ProgressSink};
    use std::sync::Mutex;

    struct CancelAfterFirst {
        cancellation: CancellationToken,
        events: Mutex<Vec<HashProgress>>,
    }
    impl ProgressSink for CancelAfterFirst {
        fn on_progress(&self, progress: HashProgress) {
            self.events.lock().unwrap().push(progress);
            self.cancellation.cancel();
        }
    }

    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::create_dir(&payload).unwrap();
    fs::write(payload.join("a"), vec![1_u8; 100_000]).unwrap();
    fs::write(payload.join("b"), vec![2_u8; 100_000]).unwrap();
    let metainfo = torrent(&payload, CreateMode::V2);
    let cancellation = CancellationToken::new();
    let progress = CancelAfterFirst {
        cancellation: cancellation.clone(),
        events: Mutex::new(Vec::new()),
    };
    assert!(
        Verifier::new(&metainfo, &payload)
            .cancellation(cancellation)
            .verify(&progress)
            .is_err()
    );
    assert!(!progress.events.lock().unwrap().is_empty());
}

#[test]
fn single_file_accepts_the_file_itself_or_its_parent_directory() {
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, b"single").unwrap();
    let metainfo = torrent(&payload, CreateMode::Hybrid);
    assert!(
        Verifier::new(&metainfo, &payload)
            .verify(&NoProgress)
            .unwrap()
            .is_valid()
    );
    assert!(
        Verifier::new(&metainfo, temp.path())
            .verify(&NoProgress)
            .unwrap()
            .is_valid()
    );
}
