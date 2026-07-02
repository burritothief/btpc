use std::fs;
use std::path::Path;

use crate::{Error, Result};

use super::{DurabilityPolicy, OverwritePolicy};

/// Atomically writes bytes using BTPC's overwrite and durability policies.
///
/// # Errors
///
/// Returns an I/O error while creating, syncing, or publishing the destination.
// Spec: CLI-WRITE-001
pub fn write_atomic(
    destination: &Path,
    bytes: &[u8],
    overwrite: OverwritePolicy,
    durability: DurabilityPolicy,
) -> Result<()> {
    atomic_write_with_hook(
        destination,
        bytes,
        overwrite,
        durability,
        &NoAtomicWriteHook,
    )
}

#[derive(Clone, Copy)]
enum AtomicWriteStage {
    Write,
    SyncFile,
    Publish,
    SyncDirectory,
}

trait AtomicWriteHook {
    fn before(
        &self,
        _stage: AtomicWriteStage,
        _temporary: &Path,
        _destination: &Path,
    ) -> std::io::Result<()> {
        Ok(())
    }
}

struct NoAtomicWriteHook;

impl AtomicWriteHook for NoAtomicWriteHook {}

fn atomic_write_with_hook(
    destination: &Path,
    bytes: &[u8],
    overwrite: OverwritePolicy,
    durability: DurabilityPolicy,
    hook: &impl AtomicWriteHook,
) -> Result<()> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("torrent");
    let mut temporary = tempfile::Builder::new()
        .prefix(&format!(".{file_name}.btpc-tmp-"))
        .tempfile_in(parent)
        .map_err(|source| Error::io(parent, source))?;
    let temporary_path = temporary.path().to_path_buf();
    preserve_replaced_file_permissions(&temporary, destination, overwrite)?;
    {
        let file = temporary.as_file_mut();
        hook.before(AtomicWriteStage::Write, &temporary_path, destination)
            .map_err(|source| Error::io(&temporary_path, source))?;
        std::io::Write::write_all(file, bytes)
            .map_err(|source| Error::io(&temporary_path, source))?;
        hook.before(AtomicWriteStage::SyncFile, &temporary_path, destination)
            .map_err(|source| Error::io(&temporary_path, source))?;
        file.sync_all()
            .map_err(|source| Error::io(&temporary_path, source))?;
    }
    hook.before(AtomicWriteStage::Publish, &temporary_path, destination)
        .map_err(|source| Error::io(destination, source))?;
    publish(temporary, destination, overwrite)?;
    if durability == DurabilityPolicy::FileAndDirectory {
        hook.before(
            AtomicWriteStage::SyncDirectory,
            &temporary_path,
            destination,
        )
        .map_err(|source| Error::io(parent, source))?;
        sync_parent(parent)?;
    }
    Ok(())
}

fn preserve_replaced_file_permissions(
    temporary: &tempfile::NamedTempFile,
    destination: &Path,
    overwrite: OverwritePolicy,
) -> Result<()> {
    if overwrite != OverwritePolicy::Replace {
        return Ok(());
    }
    match fs::symlink_metadata(destination) {
        Ok(metadata) if metadata.file_type().is_file() => temporary
            .as_file()
            .set_permissions(metadata.permissions())
            .map_err(|source| Error::io(temporary.path(), source)),
        Ok(_) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(Error::io(destination, source)),
    }
}

fn publish(
    temporary: tempfile::NamedTempFile,
    destination: &Path,
    overwrite: OverwritePolicy,
) -> Result<()> {
    match overwrite {
        OverwritePolicy::Deny => temporary
            .persist_noclobber(destination)
            .map(|_| ())
            .map_err(|error| {
                let source = if error.error.kind() == std::io::ErrorKind::AlreadyExists {
                    std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "destination already exists",
                    )
                } else {
                    error.error
                };
                Error::io(destination, source)
            }),
        OverwritePolicy::Replace => temporary
            .persist(destination)
            .map(|_| ())
            .map_err(|error| Error::io(destination, error.error)),
    }
}

#[cfg(unix)]
fn sync_parent(parent: &Path) -> Result<()> {
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| Error::io(parent, source))
}

#[cfg(not(unix))]
fn sync_parent(_parent: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod atomic_write_tests {
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};

    use super::{
        AtomicWriteHook, AtomicWriteStage, DurabilityPolicy, OverwritePolicy,
        atomic_write_with_hook,
    };
    use tempfile::TempDir;

    struct FailAt(AtomicWriteStage);

    impl AtomicWriteHook for FailAt {
        fn before(
            &self,
            stage: AtomicWriteStage,
            _temporary: &Path,
            _destination: &Path,
        ) -> io::Result<()> {
            if std::mem::discriminant(&stage) == std::mem::discriminant(&self.0) {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied, "injected"));
            }
            Ok(())
        }
    }

    struct CreateDestinationBeforePublish(Vec<u8>);

    impl AtomicWriteHook for CreateDestinationBeforePublish {
        fn before(
            &self,
            stage: AtomicWriteStage,
            _temporary: &Path,
            destination: &Path,
        ) -> io::Result<()> {
            if matches!(stage, AtomicWriteStage::Publish) {
                fs::write(destination, &self.0)?;
            }
            Ok(())
        }
    }

    #[test]
    fn destination_created_immediately_before_publish_is_never_clobbered() {
        let temp = TempDir::new().unwrap();
        let destination = temp.path().join("output.torrent");
        let sentinel = b"other process".to_vec();
        let error = atomic_write_with_hook(
            &destination,
            b"new output",
            OverwritePolicy::Deny,
            DurabilityPolicy::File,
            &CreateDestinationBeforePublish(sentinel.clone()),
        )
        .unwrap_err();
        assert_eq!(error.category(), crate::ErrorCategory::Io);
        assert_eq!(fs::read(&destination).unwrap(), sentinel);
        assert_eq!(temporary_paths(temp.path()), Vec::<PathBuf>::new());
    }

    #[test]
    fn write_sync_and_publish_failures_remove_temporary_files() {
        for stage in [
            AtomicWriteStage::Write,
            AtomicWriteStage::SyncFile,
            AtomicWriteStage::Publish,
        ] {
            let temp = TempDir::new().unwrap();
            let destination = temp.path().join("output.torrent");
            assert!(
                atomic_write_with_hook(
                    &destination,
                    b"new output",
                    OverwritePolicy::Deny,
                    DurabilityPolicy::File,
                    &FailAt(stage),
                )
                .is_err()
            );
            assert!(!destination.exists());
            assert_eq!(temporary_paths(temp.path()), Vec::<PathBuf>::new());
        }
    }

    #[test]
    fn directory_sync_failure_reports_after_complete_publication() {
        let temp = TempDir::new().unwrap();
        let destination = temp.path().join("output.torrent");
        assert!(
            atomic_write_with_hook(
                &destination,
                b"complete output",
                OverwritePolicy::Deny,
                DurabilityPolicy::FileAndDirectory,
                &FailAt(AtomicWriteStage::SyncDirectory),
            )
            .is_err()
        );
        assert_eq!(fs::read(&destination).unwrap(), b"complete output");
        assert_eq!(temporary_paths(temp.path()), Vec::<PathBuf>::new());
    }

    fn temporary_paths(directory: &Path) -> Vec<PathBuf> {
        fs::read_dir(directory)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.to_string_lossy().contains(".btpc-tmp"))
            .collect()
    }
}
