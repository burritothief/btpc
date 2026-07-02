use std::fs;
use std::path::Path;

use btpc_core::ErrorCategory;
use btpc_core::create::{
    EmptyDirectoryPolicy, EmptyFilePolicy, HiddenPolicy, ManifestEntry, ManifestOptions, RootName,
    SpecialFilePolicy, SymlinkPolicy, scan_manifest,
};
use proptest::prelude::*;

// Spec: CREATE-MANIFEST-001
use tempfile::TempDir;

fn write(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, bytes).unwrap();
}

#[test]
fn scan_is_deterministic_across_creation_order_and_nested_unicode() {
    let first = TempDir::new().unwrap();
    let second = TempDir::new().unwrap();
    for root in [first.path(), second.path()] {
        fs::create_dir_all(root.join("payload")).unwrap();
    }
    write(&first.path().join("payload/z"), b"z");
    write(&first.path().join("payload/a/b"), b"bb");
    write(&first.path().join("payload/é"), b"utf8");
    write(&second.path().join("payload/é"), b"utf8");
    write(&second.path().join("payload/a/b"), b"bb");
    write(&second.path().join("payload/z"), b"z");

    let first_manifest =
        scan_manifest(first.path().join("payload"), &ManifestOptions::default()).unwrap();
    let second_manifest =
        scan_manifest(second.path().join("payload"), &ManifestOptions::default()).unwrap();
    assert_eq!(first_manifest.root_name(), b"payload");
    assert_eq!(first_manifest.total_length(), 7);
    assert_eq!(
        first_manifest
            .entries()
            .iter()
            .map(ManifestEntry::torrent_path)
            .collect::<Vec<_>>(),
        vec![
            &[b"a".to_vec(), b"b".to_vec()][..],
            &[b"z".to_vec()][..],
            &["é".as_bytes().to_vec()][..],
        ]
    );
    assert_eq!(
        first_manifest.relative_snapshot(),
        second_manifest.relative_snapshot()
    );
    assert_eq!(
        first_manifest.relative_snapshot(),
        scan_manifest(first.path().join("payload"), &ManifestOptions::default())
            .unwrap()
            .relative_snapshot()
    );
}

#[test]
fn policies_cover_hidden_empty_exclusions_and_root_override() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("payload");
    write(&root.join("keep.txt"), b"keep");
    write(&root.join("skip.log"), b"skip");
    write(&root.join(".hidden"), b"hidden");
    write(&root.join("empty"), b"");
    fs::create_dir_all(root.join("empty-dir")).unwrap();

    let options = ManifestOptions::builder()
        .hidden(HiddenPolicy::Exclude)
        .empty_files(EmptyFilePolicy::Exclude)
        .empty_directories(EmptyDirectoryPolicy::Ignore)
        .exclude(["**/*.log", "*.log"])
        .root_name(RootName::Override(b"renamed".to_vec()))
        .build()
        .unwrap();
    let manifest = scan_manifest(&root, &options).unwrap();
    assert_eq!(manifest.root_name(), b"renamed");
    assert_eq!(manifest.entries().len(), 1);
    assert_eq!(
        manifest.entries()[0].torrent_path(),
        &[b"keep.txt".to_vec()]
    );

    let include_only = ManifestOptions::builder()
        .include(["keep.txt"])
        .build()
        .unwrap();
    assert_eq!(
        scan_manifest(&root, &include_only).unwrap().entries().len(),
        1
    );

    let reject_empty_dir = ManifestOptions::builder()
        .empty_directories(EmptyDirectoryPolicy::Reject)
        .build()
        .unwrap();
    assert_eq!(
        scan_manifest(&root, &reject_empty_dir)
            .unwrap_err()
            .category(),
        ErrorCategory::Unsupported
    );
}

#[test]
fn unfiltered_and_match_all_scans_are_identical() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("payload");
    for index in 0..128 {
        write(
            &root
                .join(format!("dir-{:02}", index % 8))
                .join(format!("file-{index:03}.bin")),
            &[u8::try_from(index).unwrap()],
        );
    }
    let plain = scan_manifest(&root, &ManifestOptions::default()).unwrap();
    let filtered = scan_manifest(
        &root,
        &ManifestOptions::builder().include(["**"]).build().unwrap(),
    )
    .unwrap();
    assert_eq!(plain.relative_snapshot(), filtered.relative_snapshot());
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[test]
fn non_utf8_names_preserve_raw_bytes() {
    use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("payload");
    fs::create_dir_all(&root).unwrap();
    write(
        &root.join(std::ffi::OsString::from_vec(vec![b'f', 0xff])),
        b"x",
    );
    let manifest = scan_manifest(root, &ManifestOptions::default()).unwrap();
    assert_eq!(manifest.entries()[0].torrent_path(), &[vec![b'f', 0xff]]);
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[test]
fn text_filters_reject_non_utf8_paths_and_raw_filters_are_lossless() {
    use std::os::unix::ffi::OsStringExt as _;

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("payload");
    fs::create_dir_all(&root).unwrap();
    let first = vec![b'f', 0x80];
    let second = vec![b'f', 0x81];
    write(
        &root.join(std::ffi::OsString::from_vec(first.clone())),
        b"a",
    );
    write(
        &root.join(std::ffi::OsString::from_vec(second.clone())),
        b"b",
    );

    let text = ManifestOptions::builder().include(["f*"]).build().unwrap();
    assert_eq!(
        scan_manifest(&root, &text).unwrap_err().category(),
        ErrorCategory::Metainfo
    );

    let raw = ManifestOptions::builder()
        .include_raw_paths([vec![first.clone()]])
        .build()
        .unwrap();
    let manifest = scan_manifest(&root, &raw).unwrap();
    assert_eq!(manifest.entries().len(), 1);
    assert_eq!(manifest.entries()[0].torrent_path(), &[first]);

    let excluded = ManifestOptions::builder()
        .exclude_raw_paths([vec![second]])
        .build()
        .unwrap();
    assert_eq!(scan_manifest(&root, &excluded).unwrap().entries().len(), 1);
}

#[cfg(unix)]
#[test]
fn symlink_and_special_file_policies_are_explicit() {
    use std::os::unix::fs::{FileTypeExt as _, symlink};

    let temp = TempDir::new().unwrap();
    let root = temp.path().join("payload");
    write(&root.join("target"), b"x");
    symlink("target", root.join("link")).unwrap();

    assert_eq!(
        scan_manifest(&root, &ManifestOptions::default())
            .unwrap_err()
            .category(),
        ErrorCategory::Unsupported
    );
    let skip = ManifestOptions::builder()
        .symlinks(SymlinkPolicy::Skip)
        .build()
        .unwrap();
    assert_eq!(scan_manifest(&root, &skip).unwrap().entries().len(), 1);

    let fifo = root.join("fifo");
    let status = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(fs::symlink_metadata(&fifo).unwrap().file_type().is_fifo());
    assert_eq!(
        scan_manifest(&root, &skip).unwrap_err().category(),
        ErrorCategory::Unsupported
    );
    let skip_special = ManifestOptions::builder()
        .symlinks(SymlinkPolicy::Skip)
        .special_files(SpecialFilePolicy::Skip)
        .build()
        .unwrap();
    assert_eq!(
        scan_manifest(&root, &skip_special).unwrap().entries().len(),
        1
    );
}

#[cfg(unix)]
#[test]
fn top_level_symlink_policies_never_emit_an_empty_or_unnamed_manifest() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let file_target = temp.path().join("file-target");
    let file_link = temp.path().join("file-link");
    write(&file_target, b"file");
    symlink(&file_target, &file_link).unwrap();

    assert!(scan_manifest(&file_link, &ManifestOptions::default()).is_err());
    let skip = ManifestOptions::builder()
        .symlinks(SymlinkPolicy::Skip)
        .build()
        .unwrap();
    assert!(scan_manifest(&file_link, &skip).is_err());
    let follow = ManifestOptions::builder()
        .symlinks(SymlinkPolicy::Follow)
        .build()
        .unwrap();
    let file_manifest = scan_manifest(&file_link, &follow).unwrap();
    assert_eq!(file_manifest.root_name(), b"file-link");
    assert_eq!(
        file_manifest.entries()[0].torrent_path(),
        &[b"file-link".to_vec()]
    );

    let directory_target = temp.path().join("directory-target");
    let directory_link = temp.path().join("directory-link");
    write(&directory_target.join("child"), b"child");
    symlink(&directory_target, &directory_link).unwrap();
    let directory_manifest = scan_manifest(&directory_link, &follow).unwrap();
    assert_eq!(directory_manifest.root_name(), b"directory-link");
    assert_eq!(
        directory_manifest.entries()[0].torrent_path(),
        &[b"child".to_vec()]
    );

    let outside = temp.path().join("outside");
    write(&outside.join("escaped"), b"escaped");
    symlink(&outside, directory_target.join("escape")).unwrap();
    assert!(scan_manifest(&directory_link, &follow).is_err());
    fs::remove_file(directory_target.join("escape")).unwrap();

    symlink(&directory_target, directory_target.join("cycle")).unwrap();
    assert!(scan_manifest(&directory_link, &follow).is_err());

    let broken = temp.path().join("broken");
    symlink(temp.path().join("missing"), &broken).unwrap();
    assert!(scan_manifest(&broken, &follow).is_err());
}

proptest! {
    #[test]
    fn randomized_enumeration_produces_identical_sorted_manifest(mut names in proptest::collection::vec("[a-z]{1,8}", 1..50)) {
        names.sort();
        names.dedup();
        let forward_root = TempDir::new().unwrap();
        let reverse_root = TempDir::new().unwrap();
        for name in &names {
            write(&forward_root.path().join(name), name.as_bytes());
        }
        for name in names.iter().rev() {
            write(&reverse_root.path().join(name), name.as_bytes());
        }
        let forward = scan_manifest(forward_root.path(), &ManifestOptions::default()).unwrap();
        let reverse = scan_manifest(reverse_root.path(), &ManifestOptions::default()).unwrap();
        prop_assert_eq!(forward.entries().iter().map(ManifestEntry::torrent_path).collect::<Vec<_>>(), reverse.entries().iter().map(ManifestEntry::torrent_path).collect::<Vec<_>>());
    }
}
