use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

use btpc_core::ErrorCategory;
use btpc_core::create::{
    CancellationToken, HashProgress, ManifestOptions, NoProgress, ProgressSink, hash_v1_sequential,
    hash_v2_file_sequential, scan_manifest,
};
use tempfile::TempDir;

fn write_tree(root: &Path) -> PathBuf {
    let payload = root.join("payload");
    fs::create_dir_all(&payload).unwrap();
    for index in 0..256 {
        fs::write(payload.join(format!("file-{index:03}")), vec![0x5a; 4096]).unwrap();
    }
    payload
}

#[test]
fn concurrent_size_mutation_during_scan_is_rejected_or_snapshotted_consistently() {
    let temp = TempDir::new().unwrap();
    let payload = write_tree(temp.path());
    let target = payload.join("file-255");
    let running = Arc::new(AtomicBool::new(true));
    let writer_running = Arc::clone(&running);
    let writer_target = target.clone();
    let writer = thread::spawn(move || {
        let mut long = false;
        while writer_running.load(Ordering::Acquire) {
            let _ = fs::remove_file(&writer_target);
            thread::yield_now();
            let bytes = if long {
                vec![0xa5; 8192]
            } else {
                vec![0x5a; 4096]
            };
            fs::write(&writer_target, bytes).unwrap();
            long = !long;
            thread::yield_now();
        }
    });

    let mut rejected = 0;
    for _ in 0..100 {
        match scan_manifest(&payload, &ManifestOptions::default()) {
            Ok(manifest) => {
                let entry = manifest
                    .entries()
                    .iter()
                    .find(|entry| entry.source_path() == target);
                assert!(entry.is_none_or(|entry| entry.length() <= 8192));
            }
            Err(error) => {
                assert!(matches!(
                    error.category(),
                    ErrorCategory::Io | ErrorCategory::Unsupported
                ));
                rejected += 1;
            }
        }
    }
    running.store(false, Ordering::Release);
    writer.join().unwrap();
    assert!(rejected > 0, "stress loop never observed a scan mutation");
}

struct OverwriteOnProgress {
    path: PathBuf,
    replacement: Vec<u8>,
    fired: Mutex<bool>,
}

impl ProgressSink for OverwriteOnProgress {
    fn on_progress(&self, _progress: HashProgress) {
        let mut fired = self.fired.lock().unwrap();
        if !*fired {
            fs::write(&self.path, &self.replacement).unwrap();
            *fired = true;
        }
    }
}

#[test]
fn same_length_mutation_during_v1_and_v2_hashing_is_not_silently_accepted() {
    for v2 in [false, true] {
        let temp = TempDir::new().unwrap();
        let payload = temp.path().join("payload");
        fs::write(&payload, vec![0x11; 256 * 1024]).unwrap();
        let manifest = scan_manifest(&payload, &ManifestOptions::default()).unwrap();
        let progress = OverwriteOnProgress {
            path: payload,
            replacement: vec![0x22; 256 * 1024],
            fired: Mutex::new(false),
        };
        let error = if v2 {
            hash_v2_file_sequential(
                &manifest.entries()[0],
                16 * 1024,
                &CancellationToken::new(),
                &progress,
            )
            .unwrap_err()
        } else {
            hash_v1_sequential(
                manifest.entries(),
                16 * 1024,
                &CancellationToken::new(),
                &progress,
            )
            .unwrap_err()
        };
        assert_eq!(error.category(), ErrorCategory::Io);
    }
}

#[cfg(unix)]
#[test]
fn same_size_replacement_with_restored_mtime_is_rejected() {
    use std::os::unix::fs::MetadataExt as _;

    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, vec![0x11; 64 * 1024]).unwrap();
    let manifest = scan_manifest(&payload, &ManifestOptions::default()).unwrap();
    let original = fs::metadata(&payload).unwrap();
    let replacement = temp.path().join("replacement");
    fs::write(&replacement, vec![0x22; 64 * 1024]).unwrap();
    let times = fs::FileTimes::new().set_modified(original.modified().unwrap());
    fs::File::open(&replacement)
        .unwrap()
        .set_times(times)
        .unwrap();
    fs::rename(&replacement, &payload).unwrap();
    assert_ne!(fs::metadata(&payload).unwrap().ino(), original.ino());

    let error = hash_v1_sequential(
        manifest.entries(),
        16 * 1024,
        &CancellationToken::new(),
        &NoProgress,
    )
    .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Io);
}

struct RenameOverOpen {
    path: PathBuf,
    replacement: PathBuf,
    fired: Mutex<bool>,
}

impl ProgressSink for RenameOverOpen {
    fn on_progress(&self, _progress: HashProgress) {
        let mut fired = self.fired.lock().unwrap();
        if !*fired {
            fs::rename(&self.replacement, &self.path).unwrap();
            *fired = true;
        }
    }
}

#[cfg(unix)]
#[test]
fn rename_over_open_hashes_a_coherent_handle_but_reports_path_replacement() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    let replacement = temp.path().join("replacement");
    fs::write(&payload, vec![0x11; 256 * 1024]).unwrap();
    fs::write(&replacement, vec![0x22; 256 * 1024]).unwrap();
    let manifest = scan_manifest(&payload, &ManifestOptions::default()).unwrap();
    let error = hash_v1_sequential(
        manifest.entries(),
        16 * 1024,
        &CancellationToken::new(),
        &RenameOverOpen {
            path: payload,
            replacement,
            fired: Mutex::new(false),
        },
    )
    .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Io);
}

struct TruncateAndRegrow {
    path: PathBuf,
    original_length: u64,
    fired: Mutex<bool>,
}

impl ProgressSink for TruncateAndRegrow {
    fn on_progress(&self, _progress: HashProgress) {
        let mut fired = self.fired.lock().unwrap();
        if !*fired {
            fs::write(&self.path, []).unwrap();
            let file = fs::OpenOptions::new().write(true).open(&self.path).unwrap();
            file.set_len(self.original_length).unwrap();
            let times = fs::FileTimes::new().set_modified(SystemTime::UNIX_EPOCH);
            file.set_times(times).unwrap();
            *fired = true;
        }
    }
}

#[test]
fn truncation_and_regrowth_during_hashing_is_rejected() {
    let temp = TempDir::new().unwrap();
    let payload = temp.path().join("payload");
    fs::write(&payload, vec![0x11; 256 * 1024]).unwrap();
    let manifest = scan_manifest(&payload, &ManifestOptions::default()).unwrap();
    let error = hash_v1_sequential(
        manifest.entries(),
        16 * 1024,
        &CancellationToken::new(),
        &TruncateAndRegrow {
            path: payload,
            original_length: 256 * 1024,
            fired: Mutex::new(false),
        },
    )
    .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Io);
}

#[test]
fn stable_files_still_scan_and_hash_after_mutation_stress() {
    let temp = TempDir::new().unwrap();
    let payload = write_tree(temp.path());
    let manifest = scan_manifest(&payload, &ManifestOptions::default()).unwrap();
    let result = hash_v1_sequential(
        manifest.entries(),
        16 * 1024,
        &CancellationToken::new(),
        &NoProgress,
    )
    .unwrap();
    assert_eq!(result.total_bytes(), manifest.total_length());
}
