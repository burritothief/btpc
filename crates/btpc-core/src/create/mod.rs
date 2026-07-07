//! Deterministic payload discovery for torrent creation.

mod atomic_output;
mod piece_length;
mod progress;

pub use atomic_output::write_atomic;
pub use piece_length::{
    PIECE_LENGTH_POLICY_ID, PieceLengthMode, automatic_piece_length, validate_piece_length,
};
pub use progress::{CancellationToken, HashProgress, NoProgress, ProgressSink};

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use globset::{Glob, GlobSet, GlobSetBuilder};
use sha1::Digest as _;

use crate::metadata::{
    MetadataText, NodeHost, TrackerTier, WebSeed, validate_creation_date, validate_nodes,
    validate_tracker_tiers, validate_web_seeds,
};
use crate::{Error, Result};

const HASH_READ_BUFFER_LENGTH: usize = 64 * 1024;

/// Policy for dot-prefixed path components.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HiddenPolicy {
    /// Include hidden files and directories.
    #[default]
    Include,
    /// Exclude hidden files and directories.
    Exclude,
}

/// Policy for symbolic links.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SymlinkPolicy {
    /// Reject a manifest containing a symbolic link.
    #[default]
    Reject,
    /// Ignore symbolic links.
    Skip,
    /// Follow links that remain beneath the scanned root.
    Follow,
}

/// Policy for sockets, devices, FIFOs, and other non-file entries.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SpecialFilePolicy {
    /// Reject special files.
    #[default]
    Reject,
    /// Ignore special files.
    Skip,
}

/// Policy for zero-length files.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EmptyFilePolicy {
    /// Include zero-length files.
    #[default]
    Include,
    /// Ignore zero-length files.
    Exclude,
}

/// Policy for empty directories, which cannot be represented in metainfo.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EmptyDirectoryPolicy {
    /// Ignore empty directories.
    #[default]
    Ignore,
    /// Reject payloads containing empty directories.
    Reject,
}

/// How the torrent root name is selected.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum RootName {
    /// Use the scanned file or directory name.
    #[default]
    Automatic,
    /// Use caller-provided raw torrent name bytes.
    Override(Vec<u8>),
}

/// Options controlling deterministic payload discovery.
#[derive(Clone, Debug)]
pub struct ManifestOptions {
    hidden: HiddenPolicy,
    symlinks: SymlinkPolicy,
    special_files: SpecialFilePolicy,
    empty_files: EmptyFilePolicy,
    empty_directories: EmptyDirectoryPolicy,
    include: GlobSet,
    include_is_empty: bool,
    exclude: GlobSet,
    exclude_is_empty: bool,
    include_raw_paths: Vec<Vec<Vec<u8>>>,
    exclude_raw_paths: Vec<Vec<Vec<u8>>>,
    root_name: RootName,
}

impl Default for ManifestOptions {
    fn default() -> Self {
        Self::builder().build().expect("empty glob sets are valid")
    }
}

impl ManifestOptions {
    /// Starts an options builder.
    #[must_use]
    pub fn builder() -> ManifestOptionsBuilder {
        ManifestOptionsBuilder::default()
    }
}

/// Builder for [`ManifestOptions`].
#[derive(Clone, Debug, Default)]
pub struct ManifestOptionsBuilder {
    hidden: HiddenPolicy,
    symlinks: SymlinkPolicy,
    special_files: SpecialFilePolicy,
    empty_files: EmptyFilePolicy,
    empty_directories: EmptyDirectoryPolicy,
    include: Vec<String>,
    exclude: Vec<String>,
    include_raw_paths: Vec<Vec<Vec<u8>>>,
    exclude_raw_paths: Vec<Vec<Vec<u8>>>,
    root_name: RootName,
}

impl ManifestOptionsBuilder {
    /// Sets hidden-file handling.
    #[must_use]
    pub const fn hidden(mut self, policy: HiddenPolicy) -> Self {
        self.hidden = policy;
        self
    }

    /// Sets symbolic-link handling.
    #[must_use]
    pub const fn symlinks(mut self, policy: SymlinkPolicy) -> Self {
        self.symlinks = policy;
        self
    }

    /// Sets special-file handling.
    #[must_use]
    pub const fn special_files(mut self, policy: SpecialFilePolicy) -> Self {
        self.special_files = policy;
        self
    }

    /// Sets zero-length file handling.
    #[must_use]
    pub const fn empty_files(mut self, policy: EmptyFilePolicy) -> Self {
        self.empty_files = policy;
        self
    }

    /// Sets empty-directory handling.
    #[must_use]
    pub const fn empty_directories(mut self, policy: EmptyDirectoryPolicy) -> Self {
        self.empty_directories = policy;
        self
    }

    /// Replaces include glob patterns.
    #[must_use]
    pub fn include(mut self, patterns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.include = patterns.into_iter().map(Into::into).collect();
        self
    }

    /// Replaces exclude glob patterns.
    #[must_use]
    pub fn exclude(mut self, patterns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.exclude = patterns.into_iter().map(Into::into).collect();
        self
    }

    /// Replaces exact raw-byte torrent paths to include.
    #[must_use]
    pub fn include_raw_paths(mut self, paths: impl IntoIterator<Item = Vec<Vec<u8>>>) -> Self {
        self.include_raw_paths = paths.into_iter().collect();
        self
    }

    /// Replaces exact raw-byte torrent paths to exclude.
    #[must_use]
    pub fn exclude_raw_paths(mut self, paths: impl IntoIterator<Item = Vec<Vec<u8>>>) -> Self {
        self.exclude_raw_paths = paths.into_iter().collect();
        self
    }

    /// Sets root-name selection.
    #[must_use]
    pub fn root_name(mut self, root_name: RootName) -> Self {
        self.root_name = root_name;
        self
    }

    /// Compiles patterns and validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns a metainfo error for an invalid root name or glob pattern.
    pub fn build(self) -> Result<ManifestOptions> {
        let include_is_empty = self.include.is_empty();
        let exclude_is_empty = self.exclude.is_empty();
        let include = compile_globs(&self.include)?;
        let exclude = compile_globs(&self.exclude)?;
        if let RootName::Override(name) = &self.root_name {
            validate_component(name, "root name")?;
        }
        Ok(ManifestOptions {
            hidden: self.hidden,
            symlinks: self.symlinks,
            special_files: self.special_files,
            empty_files: self.empty_files,
            empty_directories: self.empty_directories,
            include,
            include_is_empty,
            exclude,
            exclude_is_empty,
            include_raw_paths: self.include_raw_paths,
            exclude_raw_paths: self.exclude_raw_paths,
            root_name: self.root_name,
        })
    }
}

/// Metadata needed to stream a payload file later without keeping it open.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestEntry {
    source_path: PathBuf,
    torrent_path: Vec<Vec<u8>>,
    length: u64,
    modified: Option<SystemTime>,
    snapshot: FileSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileSnapshot {
    is_file: bool,
    length: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    identity: Option<file_id::FileId>,
    #[cfg(unix)]
    change_seconds: i64,
    #[cfg(unix)]
    change_nanoseconds: i64,
}

impl FileSnapshot {
    fn from_path(path: &Path, metadata: &fs::Metadata) -> Result<Self> {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt as _;

        Ok(Self {
            is_file: metadata.is_file(),
            length: metadata.len(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            identity: Some(file_id::get_file_id(path).map_err(|source| Error::io(path, source))?),
            #[cfg(unix)]
            change_seconds: metadata.ctime(),
            #[cfg(unix)]
            change_nanoseconds: metadata.ctime_nsec(),
        })
    }

    #[cfg(test)]
    fn limited(length: u64, modified: Option<SystemTime>) -> Self {
        Self {
            is_file: true,
            length,
            modified,
            created: None,
            identity: None,
            #[cfg(unix)]
            change_seconds: 0,
            #[cfg(unix)]
            change_nanoseconds: 0,
        }
    }

    fn matches_metadata(&self, metadata: &fs::Metadata) -> bool {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt as _;

        if self.is_limited() {
            return metadata.is_file() == self.is_file
                && metadata.len() == self.length
                && self
                    .modified
                    .is_none_or(|modified| metadata.modified().ok() == Some(modified));
        }
        metadata.is_file() == self.is_file
            && metadata.len() == self.length
            && metadata.modified().ok() == self.modified
            && metadata.created().ok() == self.created
            && {
                #[cfg(unix)]
                {
                    metadata.ctime() == self.change_seconds
                        && metadata.ctime_nsec() == self.change_nanoseconds
                }
                #[cfg(not(unix))]
                {
                    true
                }
            }
    }

    fn is_limited(&self) -> bool {
        self.identity.is_none()
    }
}

impl ManifestEntry {
    /// Returns the source filesystem path.
    #[must_use]
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// Returns raw torrent-relative path components.
    #[must_use]
    pub fn torrent_path(&self) -> &[Vec<u8>] {
        &self.torrent_path
    }

    /// Returns the snapshotted file length.
    #[must_use]
    pub const fn length(&self) -> u64 {
        self.length
    }

    /// Returns the snapshotted modification time when available.
    #[must_use]
    pub const fn modified(&self) -> Option<SystemTime> {
        self.modified
    }

    /// Constructs an entry for crate-local deterministic sorting tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn for_test(
        source: impl Into<PathBuf>,
        torrent_path: Vec<Vec<u8>>,
        length: u64,
    ) -> Self {
        Self {
            source_path: source.into(),
            torrent_path,
            length,
            modified: None,
            snapshot: FileSnapshot::limited(length, None),
        }
    }

    /// Constructs a limited entry from previously snapshotted metadata.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn from_snapshot(
        source: impl Into<PathBuf>,
        torrent_path: Vec<Vec<u8>>,
        length: u64,
        modified: Option<SystemTime>,
    ) -> Self {
        Self {
            source_path: source.into(),
            torrent_path,
            length,
            modified,
            snapshot: FileSnapshot::limited(length, modified),
        }
    }
}

/// Deterministically sorted payload snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadManifest {
    root_name: Vec<u8>,
    entries: Vec<ManifestEntry>,
    total_length: u64,
}

impl PayloadManifest {
    /// Returns the raw torrent root name.
    #[must_use]
    pub fn root_name(&self) -> &[u8] {
        &self.root_name
    }

    /// Returns sorted payload files.
    #[must_use]
    pub fn entries(&self) -> &[ManifestEntry] {
        &self.entries
    }

    /// Returns checked total payload bytes.
    #[must_use]
    pub const fn total_length(&self) -> u64 {
        self.total_length
    }

    /// Returns a root-independent snapshot useful for deterministic comparisons.
    #[must_use]
    pub fn relative_snapshot(&self) -> Vec<(Vec<Vec<u8>>, u64)> {
        self.entries
            .iter()
            .map(|entry| (entry.torrent_path.clone(), entry.length))
            .collect()
    }
}

/// Scans a file or directory into a deterministic manifest.
///
/// On Unix, torrent components are the exact `OsStr` bytes. On Windows and
/// other platforms without byte-oriented names, components are UTF-8 encoding
/// of their Unicode representation. Entries are sorted lexicographically by
/// unsigned bytes per component.
///
/// # Errors
///
/// Returns contextual I/O, unsupported-policy, unsafe-component, collision,
/// mutation, or checked-size errors.
pub fn scan_manifest(path: impl AsRef<Path>, options: &ManifestOptions) -> Result<PayloadManifest> {
    let path = path.as_ref();
    let metadata = fs::symlink_metadata(path).map_err(|source| Error::io(path, source))?;
    let automatic_name = path
        .file_name()
        .ok_or_else(|| Error::metainfo_field("root name", "input has no file name"))?;
    let root_name = match &options.root_name {
        RootName::Automatic => os_component_bytes(automatic_name)?,
        RootName::Override(name) => name.clone(),
    };
    validate_component(&root_name, "root name")?;
    let canonical_root = path
        .canonicalize()
        .map_err(|source| Error::io(path, source))?;
    let mut entries = Vec::new();
    let mut visited = HashSet::new();
    if metadata.file_type().is_symlink() {
        match options.symlinks {
            SymlinkPolicy::Reject => {
                return Err(Error::unsupported(format!(
                    "symbolic link encountered: {}",
                    path.display()
                )));
            }
            SymlinkPolicy::Skip => {
                return Err(Error::unsupported(
                    "top-level symbolic link cannot produce an empty manifest",
                ));
            }
            SymlinkPolicy::Follow => {
                let target = fs::metadata(path).map_err(|source| Error::io(path, source))?;
                if target.is_file() {
                    collect_file(
                        path,
                        vec![root_name.clone()],
                        options,
                        &mut entries,
                        &target,
                    )?;
                } else if target.is_dir() {
                    visited.insert(canonical_root.clone());
                    walk_directory(
                        path,
                        &[],
                        &canonical_root,
                        options,
                        &mut entries,
                        &mut visited,
                    )?;
                } else {
                    handle_special(path, options)?;
                }
            }
        }
    } else if metadata.is_file() {
        collect_file(
            path,
            vec![root_name.clone()],
            options,
            &mut entries,
            &metadata,
        )?;
    } else if metadata.is_dir() {
        visited.insert(canonical_root.clone());
        walk_directory(
            path,
            &[],
            &canonical_root,
            options,
            &mut entries,
            &mut visited,
        )?;
    } else {
        handle_special(path, options)?;
    }
    let entries = sort_manifest_entries(entries)?;
    let total_length = entries.iter().try_fold(0_u64, |total, entry| {
        total.checked_add(entry.length).ok_or_else(|| {
            Error::metainfo_field("manifest length", "total payload length overflowed")
        })
    })?;
    Ok(PayloadManifest {
        root_name,
        entries,
        total_length,
    })
}

/// Sorts entries and rejects duplicate torrent paths.
///
/// # Errors
///
/// Returns a metainfo error if two entries map to the same torrent path.
pub fn sort_manifest_entries(mut entries: Vec<ManifestEntry>) -> Result<Vec<ManifestEntry>> {
    entries.sort_by(|left, right| left.torrent_path.cmp(&right.torrent_path));
    crate::metainfo::validate_torrent_path_graph(
        entries
            .iter()
            .map(|entry| entry.torrent_path.iter().map(Vec::as_slice).collect())
            .collect(),
        "manifest path",
    )?;
    Ok(entries)
}

#[cfg(test)]
mod manifest_sort_tests {
    use super::{ManifestEntry, sort_manifest_entries};
    use crate::ErrorCategory;

    #[test]
    fn rejects_duplicate_and_prefix_collisions() {
        let duplicate = vec![
            ManifestEntry::for_test("one", vec![b"same".to_vec()], 1),
            ManifestEntry::for_test("two", vec![b"same".to_vec()], 2),
        ];
        assert_eq!(
            sort_manifest_entries(duplicate).unwrap_err().category(),
            ErrorCategory::Metainfo
        );

        let prefix = vec![
            ManifestEntry::for_test("one", vec![b"a".to_vec()], 1),
            ManifestEntry::for_test("two", vec![b"a".to_vec(), b"b".to_vec()], 2),
        ];
        assert_eq!(
            sort_manifest_entries(prefix).unwrap_err().field(),
            Some("manifest path")
        );
    }
}

fn walk_directory(
    directory: &Path,
    relative: &[Vec<u8>],
    canonical_root: &Path,
    options: &ManifestOptions,
    entries: &mut Vec<ManifestEntry>,
    visited: &mut HashSet<PathBuf>,
) -> Result<bool> {
    let children = fs::read_dir(directory)
        .map_err(|source| Error::io(directory, source))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|source| Error::io(directory, source))?;
    let mut children = children
        .into_iter()
        .map(|entry| Ok((os_component_bytes(&entry.file_name())?, entry)))
        .collect::<Result<Vec<_>>>()?;
    children.sort_by(|(left, _), (right, _)| left.cmp(right));
    let mut represented = false;
    for (component, child) in children {
        validate_component(&component, "manifest path")?;
        if options.hidden == HiddenPolicy::Exclude && component.starts_with(b".") {
            continue;
        }
        let mut child_relative = relative.to_owned();
        child_relative.push(component);
        let child_path = child.path();
        let metadata =
            fs::symlink_metadata(&child_path).map_err(|source| Error::io(&child_path, source))?;
        if metadata.file_type().is_symlink() {
            represented |= handle_symlink(
                &child_path,
                child_relative,
                canonical_root,
                options,
                entries,
                visited,
            )?;
        } else if metadata.is_dir() {
            let canonical = child_path
                .canonicalize()
                .map_err(|source| Error::io(&child_path, source))?;
            if !visited.insert(canonical.clone()) {
                return Err(Error::unsupported("directory cycle during manifest scan"));
            }
            let child_represented = walk_directory(
                &child_path,
                &child_relative,
                canonical_root,
                options,
                entries,
                visited,
            )?;
            visited.remove(&canonical);
            if !child_represented && options.empty_directories == EmptyDirectoryPolicy::Reject {
                return Err(Error::unsupported(format!(
                    "empty directory cannot be represented: {}",
                    child_path.display()
                )));
            }
            represented |= child_represented;
        } else if metadata.is_file() {
            represented |= collect_file(&child_path, child_relative, options, entries, &metadata)?;
        } else {
            handle_special(&child_path, options)?;
        }
    }
    Ok(represented)
}

fn handle_symlink(
    path: &Path,
    relative: Vec<Vec<u8>>,
    canonical_root: &Path,
    options: &ManifestOptions,
    entries: &mut Vec<ManifestEntry>,
    visited: &mut HashSet<PathBuf>,
) -> Result<bool> {
    match options.symlinks {
        SymlinkPolicy::Reject => Err(Error::unsupported(format!(
            "symbolic link encountered: {}",
            path.display()
        ))),
        SymlinkPolicy::Skip => Ok(false),
        SymlinkPolicy::Follow => {
            let canonical = path
                .canonicalize()
                .map_err(|source| Error::io(path, source))?;
            if !canonical.starts_with(canonical_root) {
                return Err(Error::unsupported(format!(
                    "symbolic link escapes payload root: {}",
                    path.display()
                )));
            }
            let metadata = fs::metadata(path).map_err(|source| Error::io(path, source))?;
            if metadata.is_file() {
                collect_file(path, relative, options, entries, &metadata)
            } else if metadata.is_dir() {
                if !visited.insert(canonical.clone()) {
                    return Err(Error::unsupported("symbolic link directory cycle"));
                }
                let represented =
                    walk_directory(path, &relative, canonical_root, options, entries, visited)?;
                visited.remove(&canonical);
                Ok(represented)
            } else {
                handle_special(path, options)?;
                Ok(false)
            }
        }
    }
}

fn collect_file(
    path: &Path,
    relative: Vec<Vec<u8>>,
    options: &ManifestOptions,
    entries: &mut Vec<ManifestEntry>,
    before: &fs::Metadata,
) -> Result<bool> {
    if !options.include_is_empty || !options.exclude_is_empty {
        let mut match_path = String::new();
        for (index, component) in relative.iter().enumerate() {
            if index != 0 {
                match_path.push('/');
            }
            match_path.push_str(std::str::from_utf8(component).map_err(|_| {
                Error::metainfo_field(
                    "manifest filter",
                    "text glob patterns require UTF-8 payload paths; use raw path filters",
                )
            })?);
        }
        if (!options.include_is_empty && !options.include.is_match(&match_path))
            || (!options.exclude_is_empty && options.exclude.is_match(&match_path))
        {
            return Ok(false);
        }
    }
    if (!options.include_raw_paths.is_empty() && !options.include_raw_paths.contains(&relative))
        || options.exclude_raw_paths.contains(&relative)
    {
        return Ok(false);
    }
    if before.len() == 0 && options.empty_files == EmptyFilePolicy::Exclude {
        return Ok(false);
    }
    let snapshot = FileSnapshot::from_path(path, before)?;
    let modified = snapshot.modified;
    let after = fs::metadata(path).map_err(|source| Error::io(path, source))?;
    if !snapshot.matches_metadata(&after) || !snapshot_matches_path(&snapshot, path)? {
        return Err(Error::unsupported(format!(
            "file changed during manifest scan: {}",
            path.display()
        )));
    }
    entries.push(ManifestEntry {
        source_path: path.to_path_buf(),
        torrent_path: relative,
        length: before.len(),
        modified,
        snapshot,
    });
    Ok(true)
}

fn handle_special(path: &Path, options: &ManifestOptions) -> Result<()> {
    match options.special_files {
        SpecialFilePolicy::Reject => Err(Error::unsupported(format!(
            "special file encountered: {}",
            path.display()
        ))),
        SpecialFilePolicy::Skip => Ok(()),
    }
}

fn compile_globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(
            Glob::new(pattern)
                .map_err(|error| Error::metainfo_field("manifest glob", error.to_string()))?,
        );
    }
    builder
        .build()
        .map_err(|error| Error::metainfo_field("manifest glob", error.to_string()))
}

fn validate_component(component: &[u8], field: &'static str) -> Result<()> {
    if component.is_empty()
        || component == b"."
        || component == b".."
        || component.contains(&b'/')
        || component.contains(&b'\\')
        || component.contains(&0)
    {
        return Err(Error::metainfo_field(
            field,
            "unsafe torrent path component",
        ));
    }
    Ok(())
}

#[cfg(unix)]
#[allow(clippy::unnecessary_wraps)]
fn os_component_bytes(component: &OsStr) -> Result<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt as _;
    Ok(component.as_bytes().to_vec())
}

#[cfg(not(unix))]
fn os_component_bytes(component: &OsStr) -> Result<Vec<u8>> {
    component
        .to_str()
        .map(|text| text.as_bytes().to_vec())
        .ok_or_else(|| Error::unsupported("filesystem name is not valid Unicode on this platform"))
}

/// Ordered SHA-1 piece hashes and hashing metrics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V1HashResult {
    pieces: Vec<[u8; 20]>,
    total_bytes: u64,
}

impl V1HashResult {
    /// Returns ordered 20-byte piece digests.
    #[must_use]
    pub fn pieces(&self) -> &[[u8; 20]] {
        &self.pieces
    }

    /// Returns the number of completed pieces.
    #[must_use]
    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }

    /// Returns total payload bytes hashed.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Returns piece hashes concatenated for the v1 `pieces` field.
    #[must_use]
    pub fn concatenated_pieces(&self) -> Vec<u8> {
        self.pieces.iter().flatten().copied().collect()
    }
}

/// Sequential correctness oracle for v1 piece hashing.
///
/// Files are opened one at a time and read through a reusable 64 KiB buffer.
/// Pieces span file boundaries exactly as if all files formed one logical stream.
///
/// # Errors
///
/// Returns contextual I/O errors, cancellation, invalid piece lengths, short
/// reads, or file metadata changes relative to the manifest snapshot.
pub fn hash_v1_sequential(
    entries: &[ManifestEntry],
    piece_length: u64,
    cancellation: &CancellationToken,
    progress: &impl ProgressSink,
) -> Result<V1HashResult> {
    if piece_length == 0 {
        return Err(Error::metainfo_field("piece length", "must be positive"));
    }
    let piece_length = usize::try_from(piece_length)
        .map_err(|_| Error::metainfo_field("piece length", "cannot fit in memory size"))?;
    let total_bytes = entries.iter().try_fold(0_u64, |total, entry| {
        total.checked_add(entry.length).ok_or_else(|| {
            Error::metainfo_field("manifest length", "total payload length overflowed")
        })
    })?;
    let mut pieces = Vec::new();
    let mut piece = Vec::with_capacity(piece_length);
    let mut buffer = vec![0_u8; HASH_READ_BUFFER_LENGTH];
    let mut bytes_hashed = 0_u64;
    for entry in entries {
        cancellation.check()?;
        let mut file = open_snapshot(entry)?;
        let mut remaining = entry.length;
        while remaining > 0 {
            cancellation.check()?;
            let available = piece_length - piece.len();
            let read_limit = usize::try_from(remaining)
                .unwrap_or(usize::MAX)
                .min(available)
                .min(buffer.len());
            let read = std::io::Read::read(&mut file, &mut buffer[..read_limit])
                .map_err(|source| Error::io(&entry.source_path, source))?;
            if read == 0 {
                return Err(Error::io(
                    &entry.source_path,
                    std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "file became shorter than manifest snapshot",
                    ),
                ));
            }
            piece.extend_from_slice(&buffer[..read]);
            let read_u64 = u64::try_from(read)
                .map_err(|_| Error::metainfo_field("hashing", "read size cannot be represented"))?;
            remaining -= read_u64;
            bytes_hashed += read_u64;
            if piece.len() == piece_length {
                pieces.push(sha1::Sha1::digest(&piece).into());
                piece.clear();
            }
            progress.on_progress(HashProgress {
                bytes_hashed,
                total_bytes,
                pieces_hashed: u64::try_from(pieces.len()).unwrap_or(u64::MAX),
            });
        }
        let mut extra = [0_u8; 1];
        if std::io::Read::read(&mut file, &mut extra)
            .map_err(|source| Error::io(&entry.source_path, source))?
            != 0
        {
            return Err(Error::io(
                &entry.source_path,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "file became longer than manifest snapshot",
                ),
            ));
        }
        verify_open_snapshot(entry, &file)?;
    }
    if !piece.is_empty() {
        pieces.push(sha1::Sha1::digest(&piece).into());
        progress.on_progress(HashProgress {
            bytes_hashed,
            total_bytes,
            pieces_hashed: u64::try_from(pieces.len()).unwrap_or(u64::MAX),
        });
    }
    Ok(V1HashResult {
        pieces,
        total_bytes,
    })
}

/// Bounded per-operation v1 hashing pipeline settings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParallelHashOptions {
    workers: usize,
    queue_capacity: usize,
}

impl ParallelHashOptions {
    /// Validates an explicit worker count and bounded queue capacity.
    ///
    /// # Errors
    ///
    /// Returns a metainfo error when either value is zero.
    pub fn new(workers: usize, queue_capacity: usize) -> Result<Self> {
        if workers == 0 {
            return Err(Error::metainfo_field("threads", "must be positive"));
        }
        if queue_capacity == 0 {
            return Err(Error::metainfo_field(
                "hash queue capacity",
                "must be positive",
            ));
        }
        Ok(Self {
            workers,
            queue_capacity,
        })
    }

    /// Returns the fixed number of worker threads.
    #[must_use]
    pub const fn workers(self) -> usize {
        self.workers
    }

    /// Returns the maximum number of queued pieces awaiting a worker.
    #[must_use]
    pub const fn queue_capacity(self) -> usize {
        self.queue_capacity
    }

    /// Selects a conservative per-operation worker count from available CPUs.
    #[must_use]
    pub fn automatic() -> Self {
        let parallelism = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
        let workers = automatic_v1_workers(parallelism);
        Self {
            workers,
            queue_capacity: workers,
        }
    }
}

const fn automatic_v1_workers(parallelism: usize) -> usize {
    if parallelism <= 1 { 1 } else { 2 }
}

#[derive(Debug)]
struct PieceJob {
    sequence: usize,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct PieceHash {
    sequence: usize,
    digest: Result<[u8; 20]>,
    bytes: Vec<u8>,
}

/// Bounded parallel v1 piece hashing with ordered collection.
///
/// A single reader preserves the logical concatenated file stream. A fixed pool
/// of reusable piece buffers and a bounded job queue cap payload memory at
/// `(workers + queue capacity) * piece length`, plus one read buffer. Worker
/// threads and channels are scoped to this call.
///
/// # Errors
///
/// Returns contextual I/O errors, cancellation, invalid options, short reads,
/// worker failures, or file metadata changes relative to the manifest snapshot.
pub fn hash_v1_parallel(
    entries: &[ManifestEntry],
    piece_length: u64,
    options: ParallelHashOptions,
    cancellation: &CancellationToken,
    progress: &impl ProgressSink,
) -> Result<V1HashResult> {
    hash_v1_parallel_inner(
        entries,
        piece_length,
        options,
        cancellation,
        progress,
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn hash_v1_parallel_inner(
    entries: &[ManifestEntry],
    piece_length: u64,
    options: ParallelHashOptions,
    cancellation: &CancellationToken,
    progress: &impl ProgressSink,
    fail_sequence: Option<usize>,
    slow_sequence: Option<usize>,
) -> Result<V1HashResult> {
    let (piece_length, piece_length_u64, total_bytes, piece_count) =
        v1_parallel_shape(entries, piece_length)?;
    if piece_count == 0 {
        return Ok(V1HashResult {
            pieces: Vec::new(),
            total_bytes,
        });
    }

    let buffer_count = options
        .workers
        .checked_add(options.queue_capacity)
        .ok_or_else(|| Error::metainfo_field("hash buffers", "count overflowed"))?;
    let (buffer_tx, buffer_rx) = std::sync::mpsc::sync_channel(buffer_count);
    for _ in 0..buffer_count {
        buffer_tx
            .send(Vec::with_capacity(piece_length))
            .map_err(|_| Error::unsupported("cannot initialize hash buffer pool"))?;
    }
    let (job_tx, job_rx) = std::sync::mpsc::sync_channel(options.queue_capacity);
    let job_rx = std::sync::Arc::new(std::sync::Mutex::new(job_rx));
    let (result_tx, result_rx) = std::sync::mpsc::channel();

    std::thread::scope(|scope| {
        let producer = scope.spawn(move || {
            produce_v1_pieces(entries, piece_length, cancellation, &buffer_rx, &job_tx)
        });
        for _ in 0..options.workers {
            let receiver = std::sync::Arc::clone(&job_rx);
            let sender = result_tx.clone();
            scope.spawn(move || {
                hash_piece_jobs(&receiver, &sender, fail_sequence, slow_sequence);
            });
        }
        drop(result_tx);

        let mut pieces = vec![[0_u8; 20]; piece_count];
        let mut received = 0_usize;
        let mut completed = vec![false; piece_count];
        let mut next_progress = 0_usize;
        let mut worker_error = None;
        for result in result_rx {
            match result.digest {
                Ok(digest) if result.sequence < pieces.len() => {
                    pieces[result.sequence] = digest;
                    completed[result.sequence] = true;
                    received += 1;
                    while next_progress < completed.len() && completed[next_progress] {
                        next_progress += 1;
                        progress.on_progress(HashProgress {
                            bytes_hashed: total_bytes.min(
                                u64::try_from(next_progress)
                                    .unwrap_or(u64::MAX)
                                    .saturating_mul(piece_length_u64),
                            ),
                            total_bytes,
                            pieces_hashed: u64::try_from(next_progress).unwrap_or(u64::MAX),
                        });
                    }
                }
                Ok(_) => {
                    cancellation.cancel();
                    worker_error.get_or_insert_with(|| {
                        Error::metainfo_field(
                            "piece sequence",
                            "worker returned out-of-range index",
                        )
                    });
                }
                Err(error) => {
                    cancellation.cancel();
                    worker_error.get_or_insert(error);
                }
            }
            let mut bytes = result.bytes;
            bytes.clear();
            let _ = buffer_tx.try_send(bytes);
        }
        let producer_result = producer
            .join()
            .map_err(|_| Error::unsupported("v1 piece reader thread panicked"))?;
        if let Some(error) = worker_error {
            return Err(error);
        }
        producer_result?;
        cancellation.check()?;
        if received != piece_count {
            return Err(Error::metainfo_field(
                "piece count",
                format!("expected {piece_count} worker results, received {received}"),
            ));
        }
        Ok(V1HashResult {
            pieces,
            total_bytes,
        })
    })
}

fn v1_parallel_shape(
    entries: &[ManifestEntry],
    piece_length: u64,
) -> Result<(usize, u64, u64, usize)> {
    if piece_length == 0 {
        return Err(Error::metainfo_field("piece length", "must be positive"));
    }
    let piece_length_usize = usize::try_from(piece_length)
        .map_err(|_| Error::metainfo_field("piece length", "cannot fit in memory size"))?;
    let total_bytes = entries.iter().try_fold(0_u64, |total, entry| {
        total.checked_add(entry.length).ok_or_else(|| {
            Error::metainfo_field("manifest length", "total payload length overflowed")
        })
    })?;
    let piece_count = usize::try_from(total_bytes.div_ceil(piece_length))
        .map_err(|_| Error::metainfo_field("piece count", "cannot fit in memory size"))?;
    Ok((piece_length_usize, piece_length, total_bytes, piece_count))
}

fn produce_v1_pieces(
    entries: &[ManifestEntry],
    piece_length: usize,
    cancellation: &CancellationToken,
    buffer_rx: &std::sync::mpsc::Receiver<Vec<u8>>,
    job_tx: &std::sync::mpsc::SyncSender<PieceJob>,
) -> Result<()> {
    let mut sequence = 0_usize;
    let mut piece = buffer_rx
        .recv()
        .map_err(|_| Error::unsupported("hash buffer pool disconnected"))?;
    let mut read_buffer = vec![0_u8; HASH_READ_BUFFER_LENGTH];
    for entry in entries {
        cancellation.check()?;
        let mut file = open_snapshot(entry)?;
        let mut remaining = entry.length;
        while remaining > 0 {
            cancellation.check()?;
            let available = piece_length - piece.len();
            let read_limit = usize::try_from(remaining)
                .unwrap_or(usize::MAX)
                .min(available)
                .min(read_buffer.len());
            let read = std::io::Read::read(&mut file, &mut read_buffer[..read_limit])
                .map_err(|source| Error::io(&entry.source_path, source))?;
            if read == 0 {
                return Err(Error::io(
                    &entry.source_path,
                    std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "file became shorter than manifest snapshot",
                    ),
                ));
            }
            piece.extend_from_slice(&read_buffer[..read]);
            remaining -= u64::try_from(read)
                .map_err(|_| Error::metainfo_field("hashing", "read size cannot be represented"))?;
            if piece.len() == piece_length {
                job_tx
                    .send(PieceJob {
                        sequence,
                        bytes: piece,
                    })
                    .map_err(|_| Error::unsupported("hash workers disconnected"))?;
                sequence += 1;
                piece = buffer_rx
                    .recv()
                    .map_err(|_| Error::unsupported("hash buffer pool disconnected"))?;
            }
        }
        let mut extra = [0_u8; 1];
        if std::io::Read::read(&mut file, &mut extra)
            .map_err(|source| Error::io(&entry.source_path, source))?
            != 0
        {
            return Err(Error::io(
                &entry.source_path,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "file became longer than manifest snapshot",
                ),
            ));
        }
        verify_open_snapshot(entry, &file)?;
    }
    if !piece.is_empty() {
        job_tx
            .send(PieceJob {
                sequence,
                bytes: piece,
            })
            .map_err(|_| Error::unsupported("hash workers disconnected"))?;
    }
    Ok(())
}

fn hash_piece_jobs(
    receiver: &std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<PieceJob>>>,
    sender: &std::sync::mpsc::Sender<PieceHash>,
    fail_sequence: Option<usize>,
    slow_sequence: Option<usize>,
) {
    loop {
        let job = match receiver.lock() {
            Ok(receiver) => receiver.recv(),
            Err(_) => return,
        };
        let Ok(job) = job else {
            return;
        };
        if slow_sequence == Some(job.sequence) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let digest = if fail_sequence == Some(job.sequence) {
            Err(Error::io(
                format!("piece-{}", job.sequence),
                std::io::Error::other("injected worker failure"),
            ))
        } else {
            Ok(sha1::Sha1::digest(&job.bytes).into())
        };
        if sender
            .send(PieceHash {
                sequence: job.sequence,
                digest,
                bytes: job.bytes,
            })
            .is_err()
        {
            return;
        }
    }
}

/// BEP 52 leaf block length in bytes.
pub const V2_BLOCK_LENGTH: usize = 16 * 1024;

/// Per-file BEP 52 Merkle result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V2HashResult {
    pieces_root: Option<[u8; 32]>,
    piece_layer: Vec<[u8; 32]>,
    total_bytes: u64,
}

impl V2HashResult {
    /// Returns no root for an empty file and the BEP 52 root otherwise.
    #[must_use]
    pub const fn pieces_root(&self) -> Option<&[u8; 32]> {
        self.pieces_root.as_ref()
    }

    /// Returns piece-layer hashes only when the file exceeds one piece.
    #[must_use]
    pub fn piece_layer(&self) -> &[[u8; 32]] {
        &self.piece_layer
    }

    /// Returns payload bytes consumed from the file.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

/// Returns the BEP 52 all-zero subtree hash at `level` above leaves.
#[must_use]
pub fn v2_zero_hash(level: usize) -> [u8; 32] {
    let mut hash = [0; 32];
    for _ in 0..level {
        hash = hash_v2_pair(hash, hash);
    }
    hash
}

/// Hashes one file into a BEP 52 pieces root and optional piece layer.
///
/// # Errors
///
/// Returns an error for invalid piece lengths, cancellation, filesystem failures,
/// or when the file no longer matches the supplied manifest snapshot.
pub fn hash_v2_file_sequential(
    entry: &ManifestEntry,
    piece_length: u64,
    cancellation: &CancellationToken,
    progress: &impl ProgressSink,
) -> Result<V2HashResult> {
    let mut file = open_snapshot(entry)?;
    let result = hash_v2_open_file_sequential(
        &mut file,
        &entry.source_path,
        entry.length,
        piece_length,
        cancellation,
        progress,
    )?;
    verify_open_snapshot(entry, &file)?;
    Ok(result)
}

pub(crate) fn hash_v2_open_file_sequential(
    file: &mut fs::File,
    source_path: &Path,
    length: u64,
    piece_length: u64,
    cancellation: &CancellationToken,
    progress: &impl ProgressSink,
) -> Result<V2HashResult> {
    let piece_length = validate_piece_length(piece_length, PieceLengthMode::V2)?;
    let blocks_per_piece = usize::try_from(piece_length / V2_BLOCK_LENGTH as u64)
        .map_err(|_| Error::metainfo_field("piece length", "cannot be represented"))?;
    cancellation.check()?;
    let mut buffer = [0_u8; V2_BLOCK_LENGTH];
    let mut file_tree = MerkleAccumulator::default();
    let mut piece_tree = MerkleAccumulator::default();
    let mut piece_layer = Vec::new();
    let mut blocks_in_piece = 0_usize;
    let mut bytes_hashed = 0_u64;

    loop {
        cancellation.check()?;
        let mut block_length = 0_usize;
        while block_length < buffer.len() {
            let read = std::io::Read::read(&mut *file, &mut buffer[block_length..])
                .map_err(|source| Error::io(source_path, source))?;
            if read == 0 {
                break;
            }
            block_length += read;
        }
        if block_length == 0 {
            break;
        }
        let block_hash: [u8; 32] = sha2::Sha256::digest(&buffer[..block_length]).into();
        file_tree.push_leaf(block_hash);
        piece_tree.push_leaf(block_hash);
        blocks_in_piece += 1;
        bytes_hashed = bytes_hashed
            .checked_add(u64::try_from(block_length).unwrap_or(u64::MAX))
            .ok_or_else(|| Error::metainfo_field("hashing", "byte count overflowed"))?;
        if blocks_in_piece == blocks_per_piece {
            piece_layer.push(piece_tree.finish_power_of_two(blocks_per_piece));
            piece_tree = MerkleAccumulator::default();
            blocks_in_piece = 0;
        }
        progress.on_progress(HashProgress {
            bytes_hashed,
            total_bytes: length,
            pieces_hashed: u64::try_from(piece_layer.len()).unwrap_or(u64::MAX),
        });
        if block_length < buffer.len() {
            break;
        }
    }
    cancellation.check()?;
    if bytes_hashed != length {
        return Err(Error::io(
            source_path,
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "file length differs from manifest snapshot",
            ),
        ));
    }
    if bytes_hashed == 0 {
        return Ok(V2HashResult {
            pieces_root: None,
            piece_layer: Vec::new(),
            total_bytes: 0,
        });
    }
    if blocks_in_piece != 0 && bytes_hashed > piece_length {
        piece_layer.push(piece_tree.finish_power_of_two(blocks_per_piece));
    }
    if bytes_hashed <= piece_length {
        piece_layer.clear();
    }
    progress.on_progress(HashProgress {
        bytes_hashed,
        total_bytes: length,
        pieces_hashed: u64::try_from(piece_layer.len()).unwrap_or(u64::MAX),
    });
    cancellation.check()?;
    let pieces_root = file_tree.finish_next_power_of_two();
    Ok(V2HashResult {
        pieces_root: Some(pieces_root),
        piece_layer,
        total_bytes: bytes_hashed,
    })
}

#[derive(Default)]
struct MerkleAccumulator {
    levels: Vec<Option<[u8; 32]>>,
    leaves: usize,
}

impl MerkleAccumulator {
    fn push_leaf(&mut self, hash: [u8; 32]) {
        self.push_subtree(hash, 0);
    }

    fn push_subtree(&mut self, mut hash: [u8; 32], mut level: usize) {
        self.leaves += 1_usize << level;
        loop {
            if self.levels.len() <= level {
                self.levels.resize(level + 1, None);
            }
            if let Some(left) = self.levels[level].take() {
                hash = hash_v2_pair(left, hash);
                level += 1;
            } else {
                self.levels[level] = Some(hash);
                return;
            }
        }
    }

    fn finish_next_power_of_two(mut self) -> [u8; 32] {
        let target = self.leaves.next_power_of_two();
        self.pad_to(target);
        self.root(target)
    }

    fn finish_power_of_two(mut self, target: usize) -> [u8; 32] {
        self.pad_to(target);
        self.root(target)
    }

    fn pad_to(&mut self, target: usize) {
        while self.leaves < target {
            let remaining = target - self.leaves;
            let alignment_level = self.leaves.trailing_zeros() as usize;
            let remaining_level = usize::BITS as usize - 1 - remaining.leading_zeros() as usize;
            let level = alignment_level.min(remaining_level);
            self.push_subtree(v2_zero_hash(level), level);
        }
    }

    fn root(mut self, target: usize) -> [u8; 32] {
        let level = target.trailing_zeros() as usize;
        self.levels[level]
            .take()
            .expect("a completed power-of-two tree has one root")
    }
}

fn hash_v2_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    use sha2::Digest as _;
    let mut hasher = sha2::Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

fn hash_v2_files_parallel(
    entries: &[ManifestEntry],
    piece_length: u64,
    workers: usize,
    cancellation: &CancellationToken,
    progress: &impl ProgressSink,
) -> Result<Vec<V2HashResult>> {
    hash_v2_files_parallel_inner(entries, piece_length, workers, cancellation, progress, None)
}

fn hash_v2_files_parallel_inner(
    entries: &[ManifestEntry],
    piece_length: u64,
    workers: usize,
    cancellation: &CancellationToken,
    progress: &impl ProgressSink,
    worker_counts: Option<&WorkerCounts>,
) -> Result<Vec<V2HashResult>> {
    let total_bytes = entries.iter().try_fold(0_u64, |total, entry| {
        total
            .checked_add(entry.length())
            .ok_or_else(|| Error::metainfo_field("progress", "byte count overflowed"))
    })?;
    let next = std::sync::Arc::new(std::sync::Mutex::new(0_usize));
    let (sender, receiver) = std::sync::mpsc::sync_channel(workers);
    std::thread::scope(|scope| {
        for _ in 0..workers.min(entries.len()) {
            let next = std::sync::Arc::clone(&next);
            let sender = sender.clone();
            scope.spawn(move || {
                loop {
                    let index = match next.lock() {
                        Ok(mut next) if *next < entries.len() => {
                            let index = *next;
                            *next += 1;
                            index
                        }
                        _ => return,
                    };
                    let _active = ActiveV2Worker::new(worker_counts);
                    let result = hash_v2_file_sequential(
                        &entries[index],
                        piece_length,
                        cancellation,
                        &NoProgress,
                    );
                    if result.is_err() {
                        cancellation.cancel();
                    }
                    if sender.send((index, result)).is_err() {
                        return;
                    }
                }
            });
        }
        drop(sender);
        let mut ordered = vec![None; entries.len()];
        let mut first_error = None;
        let mut next_progress = 0_usize;
        let mut bytes = 0_u64;
        let mut pieces = 0_u64;
        for (index, result) in receiver {
            match result {
                Ok(hashes) => {
                    ordered[index] = Some(hashes);
                    while next_progress < ordered.len() && ordered[next_progress].is_some() {
                        bytes = bytes
                            .checked_add(entries[next_progress].length())
                            .ok_or_else(|| {
                                Error::metainfo_field("hash progress", "byte count overflowed")
                            })?;
                        pieces = pieces
                            .checked_add(entries[next_progress].length().div_ceil(piece_length))
                            .ok_or_else(|| {
                                Error::metainfo_field("hash progress", "piece count overflowed")
                            })?;
                        progress.on_progress(HashProgress {
                            bytes_hashed: bytes,
                            total_bytes,
                            pieces_hashed: pieces,
                        });
                        if let Err(error) = cancellation.check() {
                            prefer_operation_error(&mut first_error, error);
                            cancellation.cancel();
                        }
                        next_progress += 1;
                    }
                }
                Err(error) => {
                    prefer_operation_error(&mut first_error, error);
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        cancellation.check()?;
        let mut output = Vec::with_capacity(entries.len());
        for hashes in ordered {
            let hashes = hashes.ok_or_else(|| {
                Error::unsupported("v2 worker exited without returning a file result")
            })?;
            output.push(hashes);
        }
        Ok(output)
    })
}

fn prefer_operation_error(current: &mut Option<Error>, candidate: Error) {
    if current
        .as_ref()
        .is_none_or(|error| error.category() == crate::ErrorCategory::Cancelled)
        || candidate.category() != crate::ErrorCategory::Cancelled
    {
        *current = Some(candidate);
    }
}

struct WorkerCounts {
    active: std::sync::atomic::AtomicUsize,
    peak: std::sync::atomic::AtomicUsize,
}

struct ActiveV2Worker<'a> {
    active: Option<&'a std::sync::atomic::AtomicUsize>,
}

impl<'a> ActiveV2Worker<'a> {
    fn new(counts: Option<&'a WorkerCounts>) -> Self {
        let active = counts.map(|counts| {
            let active = counts
                .active
                .fetch_add(1, std::sync::atomic::Ordering::AcqRel)
                + 1;
            counts
                .peak
                .fetch_max(active, std::sync::atomic::Ordering::AcqRel);
            &counts.active
        });
        Self { active }
    }
}

impl Drop for ActiveV2Worker<'_> {
    fn drop(&mut self) {
        if let Some(active) = self.active {
            active.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        }
    }
}

fn verify_snapshot(entry: &ManifestEntry) -> Result<()> {
    let metadata =
        fs::metadata(&entry.source_path).map_err(|source| Error::io(&entry.source_path, source))?;
    verify_snapshot_metadata(entry, &metadata)?;
    if snapshot_matches_path(&entry.snapshot, &entry.source_path)? {
        Ok(())
    } else {
        snapshot_changed(entry)
    }
}

fn verify_snapshot_metadata(entry: &ManifestEntry, metadata: &fs::Metadata) -> Result<()> {
    if !entry.snapshot.matches_metadata(metadata) {
        return snapshot_changed(entry);
    }
    Ok(())
}

fn snapshot_matches_path(snapshot: &FileSnapshot, path: &Path) -> Result<bool> {
    let Some(expected) = &snapshot.identity else {
        return Ok(true);
    };
    let current = file_id::get_file_id(path).map_err(|source| Error::io(path, source))?;
    Ok(current == *expected)
}

fn snapshot_changed(entry: &ManifestEntry) -> Result<()> {
    Err(Error::io(
        &entry.source_path,
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "file metadata changed since manifest scan",
        ),
    ))
}

fn open_snapshot(entry: &ManifestEntry) -> Result<fs::File> {
    verify_snapshot(entry)?;
    let file = fs::File::open(&entry.source_path)
        .map_err(|source| Error::io(&entry.source_path, source))?;
    let metadata = file
        .metadata()
        .map_err(|source| Error::io(&entry.source_path, source))?;
    verify_snapshot_metadata(entry, &metadata)?;
    verify_snapshot_identity(entry, &file)?;
    verify_snapshot(entry)?;
    Ok(file)
}

fn verify_open_snapshot(entry: &ManifestEntry, file: &fs::File) -> Result<()> {
    let metadata = file
        .metadata()
        .map_err(|source| Error::io(&entry.source_path, source))?;
    verify_snapshot_metadata(entry, &metadata)?;
    verify_snapshot_identity(entry, file)?;
    verify_snapshot(entry)
}

fn verify_snapshot_identity(entry: &ManifestEntry, file: &fs::File) -> Result<()> {
    if entry.snapshot.identity.is_none() {
        return Ok(());
    }
    let open = same_file::Handle::from_file(
        file.try_clone()
            .map_err(|source| Error::io(&entry.source_path, source))?,
    )
    .map_err(|source| Error::io(&entry.source_path, source))?;
    let path = same_file::Handle::from_path(&entry.source_path)
        .map_err(|source| Error::io(&entry.source_path, source))?;
    if open == path {
        Ok(())
    } else {
        snapshot_changed(entry)
    }
}

/// Automatic or caller-selected piece length.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PieceLength {
    /// Select using [`PIECE_LENGTH_POLICY_ID`].
    #[default]
    Automatic,
    /// Use an explicit byte count after validation.
    Exact(u64),
    /// Select automatically while targeting a maximum piece count and length.
    Target { pieces: u64, maximum: u64 },
}

/// Torrent protocol representation to create.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CreateMode {
    /// Create classic SHA-1 piece metadata.
    #[default]
    V1,
    /// Create BEP 52 per-file Merkle metadata.
    V2,
    /// Create matching v1 and v2 metadata with v1 alignment padding.
    Hybrid,
}

/// Destination replacement policy for atomic creation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverwritePolicy {
    /// Fail if the destination already exists.
    #[default]
    Deny,
    /// Atomically replace an existing destination where supported by the OS.
    Replace,
}

/// Durability requested after atomic destination publication.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DurabilityPolicy {
    /// Sync file contents before publication only.
    #[default]
    File,
    /// Also sync the parent directory where the platform supports it.
    FileAndDirectory,
}

/// Per-operation v1 hashing worker selection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HashThreads {
    /// Select a conservative bounded worker count for v1 creation.
    #[default]
    Automatic,
    /// Use exactly this many workers; one preserves the sequential oracle.
    Exact(usize),
}

/// Reproducible v1 creation options.
#[derive(Clone, Debug)]
pub struct CreateOptions {
    manifest: ManifestOptions,
    mode: CreateMode,
    piece_length: PieceLength,
    hash_threads: HashThreads,
    trackers: Vec<TrackerTier>,
    web_seeds: Vec<WebSeed>,
    nodes: Vec<(NodeHost, u16)>,
    private: Option<bool>,
    source: Option<MetadataText>,
    comment: Option<MetadataText>,
    created_by: CreatorIdentity,
    creation_date: Option<i64>,
    entropy: Option<Vec<u8>>,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self::builder()
            .build()
            .expect("default creation options are valid")
    }
}

impl CreateOptions {
    /// Starts an options builder.
    #[must_use]
    pub fn builder() -> CreateOptionsBuilder {
        CreateOptionsBuilder::default()
    }
}

/// Builder for [`CreateOptions`].
#[derive(Clone, Debug)]
pub struct CreateOptionsBuilder {
    manifest: ManifestOptions,
    mode: CreateMode,
    piece_length: PieceLength,
    hash_threads: HashThreads,
    trackers: Vec<Vec<Vec<u8>>>,
    web_seeds: Vec<Vec<u8>>,
    nodes: Vec<(Vec<u8>, u16)>,
    private: Option<bool>,
    source: Option<Vec<u8>>,
    comment: Option<Vec<u8>>,
    created_by: CreatorIdentity,
    creation_date: Option<i64>,
    entropy: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CreatorIdentity {
    Default,
    Explicit(Vec<u8>),
    Omit,
}

impl Default for CreateOptionsBuilder {
    fn default() -> Self {
        Self {
            manifest: ManifestOptions::default(),
            mode: CreateMode::default(),
            piece_length: PieceLength::default(),
            hash_threads: HashThreads::default(),
            trackers: Vec::new(),
            web_seeds: Vec::new(),
            nodes: Vec::new(),
            private: None,
            source: None,
            comment: None,
            created_by: CreatorIdentity::Default,
            creation_date: None,
            entropy: None,
        }
    }
}

impl CreateOptionsBuilder {
    /// Replaces manifest scanning options.
    #[must_use]
    pub fn manifest(mut self, manifest: ManifestOptions) -> Self {
        self.manifest = manifest;
        self
    }

    /// Selects the protocol representation to create.
    #[must_use]
    pub const fn mode(mut self, mode: CreateMode) -> Self {
        self.mode = mode;
        self
    }

    /// Selects automatic or explicit piece length.
    #[must_use]
    pub const fn piece_length(mut self, piece_length: PieceLength) -> Self {
        self.piece_length = piece_length;
        self
    }

    /// Selects automatic or explicit v1 hashing workers.
    #[must_use]
    pub const fn hash_threads(mut self, hash_threads: HashThreads) -> Self {
        self.hash_threads = hash_threads;
        self
    }

    /// Replaces tracker tiers.
    #[must_use]
    pub fn trackers(mut self, trackers: impl IntoIterator<Item = Vec<Vec<u8>>>) -> Self {
        self.trackers = trackers.into_iter().collect();
        self
    }

    /// Replaces web seed URLs.
    #[must_use]
    pub fn web_seeds(mut self, web_seeds: impl IntoIterator<Item = Vec<u8>>) -> Self {
        self.web_seeds = web_seeds.into_iter().collect();
        self
    }

    /// Replaces DHT bootstrap nodes.
    #[must_use]
    pub fn nodes(mut self, nodes: impl IntoIterator<Item = (Vec<u8>, u16)>) -> Self {
        self.nodes = nodes.into_iter().collect();
        self
    }

    /// Sets the private flag.
    #[must_use]
    pub const fn private(mut self, private: bool) -> Self {
        self.private = Some(private);
        self
    }

    /// Sets the source field in the info dictionary.
    #[must_use]
    pub fn source(mut self, source: Vec<u8>) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the top-level comment.
    #[must_use]
    pub fn comment(mut self, comment: Vec<u8>) -> Self {
        self.comment = Some(comment);
        self
    }

    /// Sets the top-level creator string.
    #[must_use]
    pub fn created_by(mut self, created_by: Vec<u8>) -> Self {
        self.created_by = CreatorIdentity::Explicit(created_by);
        self
    }

    /// Omits the top-level creator field instead of using the versioned default.
    #[must_use]
    pub fn omit_created_by(mut self) -> Self {
        self.created_by = CreatorIdentity::Omit;
        self
    }

    /// Includes an explicit Unix creation timestamp.
    #[must_use]
    pub const fn creation_date(mut self, creation_date: i64) -> Self {
        self.creation_date = Some(creation_date);
        self
    }

    /// Sets explicit cross-seeding entropy in the info dictionary.
    #[must_use]
    pub fn entropy(mut self, entropy: Vec<u8>) -> Self {
        self.entropy = Some(entropy);
        self
    }

    /// Validates metadata and builds options.
    ///
    /// # Errors
    ///
    /// Returns a metainfo error for empty tracker URLs, web seeds, node hosts, or
    /// other invalid metadata inputs.
    pub fn build(self) -> Result<CreateOptions> {
        if self.hash_threads == HashThreads::Exact(0) {
            return Err(Error::metainfo_field("threads", "must be positive"));
        }
        validate_tracker_tiers(&self.trackers)?;
        validate_web_seeds(&self.web_seeds)?;
        validate_nodes(&self.nodes)?;
        validate_creation_date(self.creation_date)?;
        Ok(CreateOptions {
            manifest: self.manifest,
            mode: self.mode,
            piece_length: self.piece_length,
            hash_threads: self.hash_threads,
            trackers: self.trackers.into_iter().map(TrackerTier::new).collect(),
            web_seeds: self.web_seeds.into_iter().map(WebSeed::new).collect(),
            nodes: self
                .nodes
                .into_iter()
                .map(|(host, port)| (NodeHost::new(host), port))
                .collect(),
            private: self.private,
            source: self.source.map(MetadataText::new),
            comment: self.comment.map(MetadataText::new),
            created_by: self.created_by,
            creation_date: self.creation_date,
            entropy: self.entropy,
        })
    }
}

/// Per-phase elapsed time for creation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CreateMetrics {
    scan: std::time::Duration,
    hash: std::time::Duration,
    serialize: std::time::Duration,
}

impl CreateMetrics {
    /// Returns manifest scan time.
    #[must_use]
    pub const fn scan(self) -> std::time::Duration {
        self.scan
    }

    /// Returns payload hash time.
    #[must_use]
    pub const fn hash(self) -> std::time::Duration {
        self.hash
    }

    /// Returns bencode construction and serialization time.
    #[must_use]
    pub const fn serialize(self) -> std::time::Duration {
        self.serialize
    }
}

/// Canonical creation result.
#[derive(Clone, Debug)]
pub struct CreateResult {
    bytes: Vec<u8>,
    mode: CreateMode,
    info_hash_v1: Option<crate::metainfo::InfoHashV1>,
    info_hash_v2: Option<crate::metainfo::InfoHashV2>,
    file_count: usize,
    payload_bytes: u64,
    piece_count: usize,
    piece_length: u64,
    piece_length_policy: Option<&'static str>,
    metrics: CreateMetrics,
}

impl CreateResult {
    /// Returns canonical metainfo bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the created protocol representation.
    #[must_use]
    pub const fn mode(&self) -> CreateMode {
        self.mode
    }

    /// Returns the SHA-1 hash for v1 or hybrid output.
    #[must_use]
    pub const fn info_hash_v1(&self) -> Option<crate::metainfo::InfoHashV1> {
        self.info_hash_v1
    }

    /// Returns the SHA-256 hash for v2 or hybrid output.
    #[must_use]
    pub const fn info_hash_v2(&self) -> Option<crate::metainfo::InfoHashV2> {
        self.info_hash_v2
    }

    /// Returns payload file count.
    #[must_use]
    pub const fn file_count(&self) -> usize {
        self.file_count
    }

    /// Returns total payload bytes.
    #[must_use]
    pub const fn payload_bytes(&self) -> u64 {
        self.payload_bytes
    }

    /// Returns v1 piece count.
    #[must_use]
    pub const fn piece_count(&self) -> usize {
        self.piece_count
    }

    /// Returns selected piece length.
    #[must_use]
    pub const fn piece_length(&self) -> u64 {
        self.piece_length
    }

    /// Returns the automatic policy identifier, or `None` for explicit length.
    #[must_use]
    pub const fn piece_length_policy(&self) -> Option<&'static str> {
        self.piece_length_policy
    }

    /// Returns phase metrics.
    #[must_use]
    pub const fn metrics(&self) -> CreateMetrics {
        self.metrics
    }
}

/// Torrent creator over one payload path.
#[derive(Clone, Debug)]
pub struct Creator {
    input: PathBuf,
    options: CreateOptions,
    cancellation: CancellationToken,
}

impl Creator {
    /// Creates a v1 creator using default reproducible options.
    #[must_use]
    pub fn new(input: impl Into<PathBuf>) -> Self {
        Self {
            input: input.into(),
            options: CreateOptions::default(),
            cancellation: CancellationToken::new(),
        }
    }

    /// Replaces creation options.
    #[must_use]
    pub fn options(mut self, options: CreateOptions) -> Self {
        self.options = options;
        self
    }

    /// Replaces the cancellation token.
    #[must_use]
    pub fn cancellation(mut self, cancellation: CancellationToken) -> Self {
        self.cancellation = cancellation;
        self
    }

    /// Creates canonical metainfo in memory.
    ///
    /// # Errors
    ///
    /// Returns scanning, hashing, cancellation, metadata, or serialization errors.
    pub fn create(&self, progress: &impl ProgressSink) -> Result<CreateResult> {
        let scan_started = std::time::Instant::now();
        let manifest = scan_manifest(&self.input, &self.options.manifest)?;
        let scan = scan_started.elapsed();
        let piece_length_mode = match self.options.mode {
            CreateMode::V1 => PieceLengthMode::V1,
            CreateMode::V2 | CreateMode::Hybrid => PieceLengthMode::V2,
        };
        let (piece_length, piece_length_policy) = match self.options.piece_length {
            PieceLength::Automatic => (
                automatic_piece_length(manifest.total_length()),
                Some(PIECE_LENGTH_POLICY_ID),
            ),
            PieceLength::Exact(piece_length) => (
                validate_piece_length(piece_length, piece_length_mode)?,
                None,
            ),
            PieceLength::Target { pieces, maximum } => {
                if pieces == 0 {
                    return Err(Error::metainfo_field("target pieces", "must be positive"));
                }
                let minimum = match piece_length_mode {
                    PieceLengthMode::V1 => 1024,
                    PieceLengthMode::V2 | PieceLengthMode::Hybrid => 16 * 1024,
                };
                let maximum = validate_piece_length(maximum, piece_length_mode)?;
                let needed = manifest.total_length().div_ceil(pieces).max(minimum);
                let selected = needed.next_power_of_two().min(maximum);
                (
                    validate_piece_length(selected, piece_length_mode)?,
                    Some("btpc-target-pieces-v1"),
                )
            }
        };
        let hash_started = std::time::Instant::now();
        let (info, piece_layers, piece_count) = match self.options.mode {
            CreateMode::V1 => {
                let hashes = self.hash_v1(&manifest, piece_length, progress)?;
                let piece_count = hashes.piece_count();
                (
                    build_v1_info(&manifest, piece_length, &hashes, &self.options)?,
                    None,
                    piece_count,
                )
            }
            CreateMode::V2 => {
                let (info, layers, piece_count) =
                    self.build_v2(&manifest, piece_length, progress)?;
                (info, layers, piece_count)
            }
            CreateMode::Hybrid => {
                validate_hybrid_manifest(&manifest)?;
                let (v2_info, layers, _) = self.build_v2(&manifest, piece_length, progress)?;
                let (hashes, padding) = self.hash_hybrid_v1(&manifest, piece_length)?;
                let piece_count = hashes.piece_count();
                (
                    build_hybrid_info(v2_info, &manifest, &hashes, &padding)?,
                    layers,
                    piece_count,
                )
            }
        };
        let hash = hash_started.elapsed();
        let serialize_started = std::time::Instant::now();
        let info_bytes = info.to_vec()?;
        let info_hash_v1 = (self.options.mode != CreateMode::V2)
            .then(|| crate::metainfo::InfoHashV1::new(sha1::Sha1::digest(&info_bytes).into()));
        let info_hash_v2 = (self.options.mode != CreateMode::V1)
            .then(|| crate::metainfo::InfoHashV2::new(sha2::Sha256::digest(&info_bytes).into()));
        let metainfo = build_metainfo(info, piece_layers, &self.options)?;
        let bytes = metainfo.to_vec()?;
        let serialize = serialize_started.elapsed();
        Ok(CreateResult {
            bytes,
            mode: self.options.mode,
            info_hash_v1,
            info_hash_v2,
            file_count: manifest.entries().len(),
            payload_bytes: manifest.total_length(),
            piece_count,
            piece_length,
            piece_length_policy,
            metrics: CreateMetrics {
                scan,
                hash,
                serialize,
            },
        })
    }

    fn hash_v1(
        &self,
        manifest: &PayloadManifest,
        piece_length: u64,
        progress: &impl ProgressSink,
    ) -> Result<V1HashResult> {
        match self.options.hash_threads {
            HashThreads::Exact(1) => hash_v1_sequential(
                manifest.entries(),
                piece_length,
                &self.cancellation,
                progress,
            ),
            HashThreads::Automatic => {
                let options = ParallelHashOptions::automatic();
                if options.workers() == 1 {
                    hash_v1_sequential(
                        manifest.entries(),
                        piece_length,
                        &self.cancellation,
                        progress,
                    )
                } else {
                    hash_v1_parallel(
                        manifest.entries(),
                        piece_length,
                        options,
                        &self.cancellation,
                        progress,
                    )
                }
            }
            HashThreads::Exact(workers) => hash_v1_parallel(
                manifest.entries(),
                piece_length,
                ParallelHashOptions::new(workers, workers)?,
                &self.cancellation,
                progress,
            ),
        }
    }

    fn hash_hybrid_v1(
        &self,
        manifest: &PayloadManifest,
        piece_length: u64,
    ) -> Result<(V1HashResult, Vec<HybridPadding>)> {
        let workers = match self.options.hash_threads {
            HashThreads::Exact(1) => 1,
            HashThreads::Exact(workers) => workers,
            HashThreads::Automatic => ParallelHashOptions::automatic().workers(),
        };
        if workers == 1 || manifest.entries().len() <= 1 {
            return hash_hybrid_v1_sequential(
                manifest.entries(),
                piece_length,
                &self.cancellation,
                &NoProgress,
            );
        }
        hash_hybrid_v1_parallel(
            manifest.entries(),
            piece_length,
            workers,
            &self.cancellation,
        )
    }

    fn build_v2(
        &self,
        manifest: &PayloadManifest,
        piece_length: u64,
        progress: &impl ProgressSink,
    ) -> Result<(
        crate::bencode::OwnedValue,
        Option<crate::bencode::OwnedValue>,
        usize,
    )> {
        let hashed = self.hash_v2_files(manifest, piece_length, progress)?;
        let mut files = Vec::with_capacity(manifest.entries().len());
        let mut layers = std::collections::BTreeMap::new();
        let mut piece_count = 0_usize;
        for (entry, hashes) in manifest.entries().iter().zip(hashed) {
            piece_count = piece_count
                .checked_add(
                    usize::try_from(entry.length().div_ceil(piece_length)).unwrap_or(usize::MAX),
                )
                .ok_or_else(|| Error::metainfo_field("piece count", "overflowed"))?;
            if !hashes.piece_layer().is_empty() {
                let root = *hashes
                    .pieces_root()
                    .expect("non-empty piece layers have roots");
                let layer = hashes
                    .piece_layer()
                    .iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                if let Some(existing) = layers.insert(root.to_vec(), layer.clone()) {
                    debug_assert_eq!(existing, layer, "equal roots have equal piece layers");
                }
            }
            files.push((entry, hashes));
        }
        let layers = (!layers.is_empty()).then(|| {
            crate::bencode::OwnedValue::Dictionary(
                layers
                    .into_iter()
                    .map(|(root, layer)| (root, crate::bencode::OwnedValue::bytes(layer)))
                    .collect(),
            )
        });
        Ok((
            build_v2_info(manifest, piece_length, &files, &self.options)?,
            layers,
            piece_count,
        ))
    }

    fn hash_v2_files(
        &self,
        manifest: &PayloadManifest,
        piece_length: u64,
        progress: &impl ProgressSink,
    ) -> Result<Vec<V2HashResult>> {
        let workers = match self.options.hash_threads {
            HashThreads::Exact(1) => 1,
            HashThreads::Exact(workers) => workers,
            HashThreads::Automatic => ParallelHashOptions::automatic().workers(),
        };
        if workers == 1 || manifest.entries().len() <= 1 {
            let mut output = Vec::with_capacity(manifest.entries().len());
            let mut bytes_before = 0_u64;
            let mut pieces_before = 0_u64;
            for entry in manifest.entries() {
                let aggregate = AggregateProgress {
                    inner: progress,
                    bytes_before,
                    pieces_before,
                    total_bytes: manifest.total_length(),
                };
                output.push(hash_v2_file_sequential(
                    entry,
                    piece_length,
                    &self.cancellation,
                    &aggregate,
                )?);
                bytes_before = bytes_before.checked_add(entry.length()).ok_or_else(|| {
                    Error::metainfo_field("hash progress", "byte count overflowed")
                })?;
                pieces_before = pieces_before
                    .checked_add(entry.length().div_ceil(piece_length))
                    .ok_or_else(|| {
                        Error::metainfo_field("hash progress", "piece count overflowed")
                    })?;
            }
            return Ok(output);
        }
        hash_v2_files_parallel(
            manifest.entries(),
            piece_length,
            workers,
            &self.cancellation,
            progress,
        )
    }

    /// Creates metainfo and atomically writes it beside the destination.
    ///
    /// # Errors
    ///
    /// Returns creation errors, an existing-destination error under deny policy,
    /// or contextual temporary-file/write/rename errors. Temporary files are
    /// removed after failure.
    pub fn create_to_path(
        &self,
        destination: impl AsRef<Path>,
        overwrite: OverwritePolicy,
        progress: &impl ProgressSink,
    ) -> Result<CreateResult> {
        self.create_to_path_with_durability(
            destination,
            overwrite,
            DurabilityPolicy::File,
            progress,
        )
    }

    /// Creates metainfo and atomically publishes it with explicit durability.
    ///
    /// # Errors
    ///
    /// Returns creation, temporary-file, sync, or atomic publication errors.
    pub fn create_to_path_with_durability(
        &self,
        destination: impl AsRef<Path>,
        overwrite: OverwritePolicy,
        durability: DurabilityPolicy,
        progress: &impl ProgressSink,
    ) -> Result<CreateResult> {
        let result = self.create(progress)?;
        write_atomic(destination.as_ref(), result.bytes(), overwrite, durability)?;
        Ok(result)
    }
}

struct AggregateProgress<'a, P> {
    inner: &'a P,
    bytes_before: u64,
    pieces_before: u64,
    total_bytes: u64,
}

impl<P: ProgressSink> ProgressSink for AggregateProgress<'_, P> {
    fn on_progress(&self, progress: HashProgress) {
        self.inner.on_progress(HashProgress {
            bytes_hashed: self
                .bytes_before
                .checked_add(progress.bytes_hashed())
                .expect("creation byte progress fits the checked manifest total"),
            total_bytes: self.total_bytes,
            pieces_hashed: self
                .pieces_before
                .checked_add(progress.pieces_hashed())
                .expect("creation piece progress fits the checked aggregate"),
        });
    }
}

fn build_v1_info(
    manifest: &PayloadManifest,
    piece_length: u64,
    hashes: &V1HashResult,
    options: &CreateOptions,
) -> Result<crate::bencode::OwnedValue> {
    use crate::bencode::OwnedValue;
    let piece_length = i64::try_from(piece_length)
        .map_err(|_| Error::metainfo_field("piece length", "cannot fit bencode integer"))?;
    let mut entries = vec![
        (
            b"name".to_vec(),
            OwnedValue::bytes(manifest.root_name().to_vec()),
        ),
        (b"piece length".to_vec(), OwnedValue::integer(piece_length)),
        (
            b"pieces".to_vec(),
            OwnedValue::bytes(hashes.concatenated_pieces()),
        ),
    ];
    if manifest.entries().len() == 1
        && manifest.entries()[0].torrent_path() == [manifest.root_name().to_vec()]
    {
        entries.push((
            b"length".to_vec(),
            OwnedValue::integer(length_to_i64(manifest.entries()[0].length())?),
        ));
    } else {
        let files = manifest
            .entries()
            .iter()
            .map(|entry| {
                OwnedValue::dictionary([
                    (
                        b"length".to_vec(),
                        OwnedValue::integer(length_to_i64(entry.length())?),
                    ),
                    (
                        b"path".to_vec(),
                        OwnedValue::list(
                            entry.torrent_path().iter().cloned().map(OwnedValue::bytes),
                        ),
                    ),
                ])
            })
            .collect::<Result<Vec<_>>>()?;
        entries.push((b"files".to_vec(), OwnedValue::list(files)));
    }
    if let Some(private) = options.private {
        entries.push((b"private".to_vec(), OwnedValue::integer(i64::from(private))));
    }
    if let Some(source) = &options.source {
        entries.push((
            b"source".to_vec(),
            OwnedValue::bytes(source.as_bytes().to_vec()),
        ));
    }
    if let Some(entropy) = &options.entropy {
        entries.push((b"entropy".to_vec(), OwnedValue::bytes(entropy.clone())));
    }
    OwnedValue::dictionary(entries)
}

fn build_v2_info(
    manifest: &PayloadManifest,
    piece_length: u64,
    files: &[(&ManifestEntry, V2HashResult)],
    options: &CreateOptions,
) -> Result<crate::bencode::OwnedValue> {
    use crate::bencode::OwnedValue;
    let mut tree = std::collections::BTreeMap::new();
    for (entry, hashes) in files {
        let relative_path = v2_relative_path(manifest, entry);
        insert_v2_file(
            &mut tree,
            relative_path,
            entry.length(),
            hashes.pieces_root(),
        )?;
    }
    let piece_length = i64::try_from(piece_length)
        .map_err(|_| Error::metainfo_field("piece length", "cannot fit bencode integer"))?;
    let mut entries = vec![
        (b"file tree".to_vec(), OwnedValue::Dictionary(tree)),
        (b"meta version".to_vec(), OwnedValue::integer(2)),
        (
            b"name".to_vec(),
            OwnedValue::bytes(manifest.root_name().to_vec()),
        ),
        (b"piece length".to_vec(), OwnedValue::integer(piece_length)),
    ];
    if let Some(private) = options.private {
        entries.push((b"private".to_vec(), OwnedValue::integer(i64::from(private))));
    }
    if let Some(source) = &options.source {
        entries.push((
            b"source".to_vec(),
            OwnedValue::bytes(source.as_bytes().to_vec()),
        ));
    }
    if let Some(entropy) = &options.entropy {
        entries.push((b"entropy".to_vec(), OwnedValue::bytes(entropy.clone())));
    }
    OwnedValue::dictionary(entries)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HybridPadding {
    after_file: usize,
    length: u64,
    offset: u64,
}

fn validate_hybrid_manifest(manifest: &PayloadManifest) -> Result<()> {
    if manifest.entries().len() > 1
        && manifest.entries().iter().any(|entry| {
            entry
                .torrent_path()
                .first()
                .is_some_and(|part| part == b".pad")
        })
    {
        return Err(Error::metainfo_field(
            "manifest path",
            "hybrid creation reserves the .pad path component",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct HybridFileHash {
    pieces: Vec<[u8; 20]>,
    padding: u64,
}

fn hash_hybrid_v1_parallel(
    entries: &[ManifestEntry],
    piece_length: u64,
    workers: usize,
    cancellation: &CancellationToken,
) -> Result<(V1HashResult, Vec<HybridPadding>)> {
    let next = std::sync::Arc::new(std::sync::Mutex::new(0_usize));
    let (sender, receiver) = std::sync::mpsc::sync_channel(workers);
    std::thread::scope(|scope| {
        for _ in 0..workers.min(entries.len()) {
            let next = std::sync::Arc::clone(&next);
            let sender = sender.clone();
            scope.spawn(move || {
                loop {
                    let index = match next.lock() {
                        Ok(mut next) if *next < entries.len() => {
                            let index = *next;
                            *next += 1;
                            index
                        }
                        _ => return,
                    };
                    let result = hash_hybrid_file(
                        &entries[index],
                        piece_length,
                        index + 1 < entries.len(),
                        cancellation,
                    );
                    if result.is_err() {
                        cancellation.cancel();
                    }
                    if sender.send((index, result)).is_err() {
                        return;
                    }
                }
            });
        }
        drop(sender);
        let mut ordered = vec![None; entries.len()];
        let mut first_error = None;
        for (index, result) in receiver {
            match result {
                Ok(hashes) => ordered[index] = Some(hashes),
                Err(error) => {
                    prefer_operation_error(&mut first_error, error);
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        cancellation.check()?;
        let mut pieces = Vec::new();
        let mut padding = Vec::new();
        let mut logical_offset = 0_u64;
        for (index, hashes) in ordered.into_iter().enumerate() {
            let hashes = hashes.ok_or_else(|| {
                Error::unsupported("hybrid worker exited without returning a file result")
            })?;
            pieces.extend(hashes.pieces);
            logical_offset = logical_offset
                .checked_add(entries[index].length())
                .ok_or_else(|| Error::metainfo_field("hybrid length", "overflowed"))?;
            if hashes.padding != 0 {
                padding.push(HybridPadding {
                    after_file: index,
                    length: hashes.padding,
                    offset: logical_offset,
                });
                logical_offset = logical_offset
                    .checked_add(hashes.padding)
                    .ok_or_else(|| Error::metainfo_field("hybrid length", "overflowed"))?;
            }
        }
        Ok((
            V1HashResult {
                pieces,
                total_bytes: logical_offset,
            },
            padding,
        ))
    })
}

fn hash_hybrid_file(
    entry: &ManifestEntry,
    piece_length: u64,
    pad_to_boundary: bool,
    cancellation: &CancellationToken,
) -> Result<HybridFileHash> {
    let piece_length = usize::try_from(piece_length)
        .map_err(|_| Error::metainfo_field("piece length", "cannot be represented"))?;
    cancellation.check()?;
    let mut file = open_snapshot(entry)?;
    let mut piece = Vec::with_capacity(piece_length);
    let mut pieces = Vec::new();
    let mut buffer = vec![0_u8; HASH_READ_BUFFER_LENGTH];
    let mut remaining = entry.length();
    while remaining > 0 {
        cancellation.check()?;
        let limit = usize::try_from(remaining)
            .unwrap_or(usize::MAX)
            .min(piece_length - piece.len())
            .min(buffer.len());
        let read = std::io::Read::read(&mut file, &mut buffer[..limit])
            .map_err(|source| Error::io(&entry.source_path, source))?;
        if read == 0 {
            return Err(Error::io(
                &entry.source_path,
                std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "file became shorter"),
            ));
        }
        piece.extend_from_slice(&buffer[..read]);
        remaining -= u64::try_from(read)
            .map_err(|_| Error::metainfo_field("hashing", "read size cannot be represented"))?;
        if piece.len() == piece_length {
            pieces.push(sha1::Sha1::digest(&piece).into());
            piece.clear();
        }
    }
    let mut extra = [0_u8; 1];
    if std::io::Read::read(&mut file, &mut extra)
        .map_err(|source| Error::io(&entry.source_path, source))?
        != 0
    {
        return Err(Error::io(
            &entry.source_path,
            std::io::Error::new(std::io::ErrorKind::InvalidData, "file became longer"),
        ));
    }
    verify_open_snapshot(entry, &file)?;
    let padding = if pad_to_boundary && !piece.is_empty() {
        let padding = piece_length - piece.len();
        piece.resize(piece_length, 0);
        u64::try_from(padding)
            .map_err(|_| Error::metainfo_field("padding", "cannot be represented"))?
    } else {
        0
    };
    if !piece.is_empty() {
        pieces.push(sha1::Sha1::digest(&piece).into());
    }
    Ok(HybridFileHash { pieces, padding })
}

fn hash_hybrid_v1_sequential(
    entries: &[ManifestEntry],
    piece_length: u64,
    cancellation: &CancellationToken,
    progress: &impl ProgressSink,
) -> Result<(V1HashResult, Vec<HybridPadding>)> {
    let piece_length = usize::try_from(piece_length)
        .map_err(|_| Error::metainfo_field("piece length", "cannot be represented"))?;
    let total_real = entries.iter().try_fold(0_u64, |total, entry| {
        total
            .checked_add(entry.length())
            .ok_or_else(|| Error::metainfo_field("manifest length", "overflowed"))
    })?;
    let mut piece = Vec::with_capacity(piece_length);
    let mut pieces = Vec::new();
    let mut padding = Vec::new();
    let mut logical_offset = 0_u64;
    let mut real_hashed = 0_u64;
    let mut buffer = vec![0_u8; 64 * 1024];
    for (index, entry) in entries.iter().enumerate() {
        cancellation.check()?;
        let mut file = open_snapshot(entry)?;
        let mut remaining = entry.length();
        while remaining > 0 {
            cancellation.check()?;
            let read_limit = usize::try_from(remaining)
                .unwrap_or(usize::MAX)
                .min(buffer.len());
            let read = std::io::Read::read(&mut file, &mut buffer[..read_limit])
                .map_err(|source| Error::io(&entry.source_path, source))?;
            if read == 0 {
                return Err(Error::io(
                    &entry.source_path,
                    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "file became shorter"),
                ));
            }
            append_v1_bytes(&buffer[..read], piece_length, &mut piece, &mut pieces);
            let read = u64::try_from(read).unwrap_or(u64::MAX);
            remaining -= read;
            real_hashed += read;
            logical_offset += read;
            progress.on_progress(HashProgress {
                bytes_hashed: real_hashed,
                total_bytes: total_real,
                pieces_hashed: u64::try_from(pieces.len()).unwrap_or(u64::MAX),
            });
        }
        verify_open_snapshot(entry, &file)?;
        if index + 1 < entries.len() {
            let piece_length_u64 = u64::try_from(piece_length).unwrap_or(u64::MAX);
            let gap = (piece_length_u64 - (logical_offset % piece_length_u64)) % piece_length_u64;
            if gap != 0 {
                let zeros = vec![0_u8; usize::try_from(gap).unwrap_or(usize::MAX)];
                append_v1_bytes(&zeros, piece_length, &mut piece, &mut pieces);
                padding.push(HybridPadding {
                    after_file: index,
                    length: gap,
                    offset: logical_offset,
                });
                logical_offset += gap;
            }
        }
    }
    if !piece.is_empty() {
        pieces.push(sha1::Sha1::digest(&piece).into());
    }
    cancellation.check()?;
    Ok((
        V1HashResult {
            pieces,
            total_bytes: logical_offset,
        },
        padding,
    ))
}

fn append_v1_bytes(
    mut bytes: &[u8],
    piece_length: usize,
    piece: &mut Vec<u8>,
    pieces: &mut Vec<[u8; 20]>,
) {
    while !bytes.is_empty() {
        let take = (piece_length - piece.len()).min(bytes.len());
        piece.extend_from_slice(&bytes[..take]);
        bytes = &bytes[take..];
        if piece.len() == piece_length {
            pieces.push(sha1::Sha1::digest(&*piece).into());
            piece.clear();
        }
    }
}

fn build_hybrid_info(
    mut v2_info: crate::bencode::OwnedValue,
    manifest: &PayloadManifest,
    hashes: &V1HashResult,
    padding: &[HybridPadding],
) -> Result<crate::bencode::OwnedValue> {
    use crate::bencode::OwnedValue;
    let OwnedValue::Dictionary(entries) = &mut v2_info else {
        unreachable!("v2 info is a dictionary");
    };
    entries.insert(
        b"pieces".to_vec(),
        OwnedValue::bytes(hashes.concatenated_pieces()),
    );
    if manifest.entries().len() == 1
        && manifest.entries()[0].torrent_path() == [manifest.root_name().to_vec()]
    {
        entries.insert(
            b"length".to_vec(),
            OwnedValue::integer(length_to_i64(manifest.entries()[0].length())?),
        );
        return Ok(v2_info);
    }
    let mut files = Vec::with_capacity(manifest.entries().len() + padding.len());
    for (index, entry) in manifest.entries().iter().enumerate() {
        files.push(v1_file_entry(entry)?);
        if let Some(pad) = padding.iter().find(|pad| pad.after_file == index) {
            files.push(v1_padding_entry(pad)?);
        }
    }
    entries.insert(b"files".to_vec(), OwnedValue::list(files));
    Ok(v2_info)
}

fn v1_file_entry(entry: &ManifestEntry) -> Result<crate::bencode::OwnedValue> {
    use crate::bencode::OwnedValue;
    OwnedValue::dictionary([
        (
            b"length".to_vec(),
            OwnedValue::integer(length_to_i64(entry.length())?),
        ),
        (
            b"path".to_vec(),
            OwnedValue::list(entry.torrent_path().iter().cloned().map(OwnedValue::bytes)),
        ),
    ])
}

fn v1_padding_entry(padding: &HybridPadding) -> Result<crate::bencode::OwnedValue> {
    use crate::bencode::OwnedValue;
    OwnedValue::dictionary([
        (b"attr".to_vec(), OwnedValue::bytes(b"p".to_vec())),
        (
            b"length".to_vec(),
            OwnedValue::integer(length_to_i64(padding.length)?),
        ),
        (
            b"path".to_vec(),
            OwnedValue::list([
                OwnedValue::bytes(b".pad".to_vec()),
                OwnedValue::bytes(format!("{}-{}", padding.offset, padding.length).into_bytes()),
            ]),
        ),
    ])
}

fn v2_relative_path<'a>(manifest: &PayloadManifest, entry: &'a ManifestEntry) -> &'a [Vec<u8>] {
    if entry.torrent_path() == [manifest.root_name().to_vec()] {
        return entry.torrent_path();
    }
    if entry
        .torrent_path()
        .first()
        .is_some_and(|component| component == manifest.root_name())
    {
        return &entry.torrent_path()[1..];
    }
    entry.torrent_path()
}

fn insert_v2_file(
    tree: &mut std::collections::BTreeMap<Vec<u8>, crate::bencode::OwnedValue>,
    path: &[Vec<u8>],
    length: u64,
    pieces_root: Option<&[u8; 32]>,
) -> Result<()> {
    use crate::bencode::OwnedValue;
    let Some((component, remaining)) = path.split_first() else {
        return Err(Error::metainfo_field("file tree", "file path is empty"));
    };
    if component.is_empty() {
        return Err(Error::metainfo_field(
            "file tree",
            "path component is empty",
        ));
    }
    if remaining.is_empty() {
        let mut properties = vec![(
            b"length".to_vec(),
            OwnedValue::integer(length_to_i64(length)?),
        )];
        if let Some(root) = pieces_root {
            properties.push((b"pieces root".to_vec(), OwnedValue::bytes(root.to_vec())));
        }
        let leaf = OwnedValue::dictionary([(Vec::new(), OwnedValue::dictionary(properties)?)])?;
        if tree.insert(component.clone(), leaf).is_some() {
            return Err(Error::metainfo_field("file tree", "path collision"));
        }
        return Ok(());
    }
    let node = tree
        .entry(component.clone())
        .or_insert_with(|| OwnedValue::Dictionary(std::collections::BTreeMap::new()));
    let OwnedValue::Dictionary(children) = node else {
        return Err(Error::metainfo_field("file tree", "path prefix collision"));
    };
    insert_v2_file(children, remaining, length, pieces_root)
}

fn build_metainfo(
    info: crate::bencode::OwnedValue,
    piece_layers: Option<crate::bencode::OwnedValue>,
    options: &CreateOptions,
) -> Result<crate::bencode::OwnedValue> {
    use crate::bencode::OwnedValue;
    let mut entries = vec![(b"info".to_vec(), info)];
    if let Some(piece_layers) = piece_layers {
        entries.push((b"piece layers".to_vec(), piece_layers));
    }
    if let Some(announce) = options
        .trackers
        .first()
        .and_then(|tier| tier.urls().first())
    {
        entries.push((
            b"announce".to_vec(),
            OwnedValue::bytes(announce.as_bytes().to_vec()),
        ));
    }
    if !options.trackers.is_empty() {
        entries.push((
            b"announce-list".to_vec(),
            OwnedValue::list(options.trackers.iter().map(|tier| {
                OwnedValue::list(
                    tier.urls()
                        .iter()
                        .cloned()
                        .map(|url| OwnedValue::bytes(url.into_bytes())),
                )
            })),
        ));
    }
    if !options.web_seeds.is_empty() {
        entries.push((
            b"url-list".to_vec(),
            OwnedValue::list(
                options
                    .web_seeds
                    .iter()
                    .cloned()
                    .map(|seed| OwnedValue::bytes(seed.into_bytes())),
            ),
        ));
    }
    if !options.nodes.is_empty() {
        entries.push((
            b"nodes".to_vec(),
            OwnedValue::list(options.nodes.iter().map(|(host, port)| {
                OwnedValue::list([
                    OwnedValue::bytes(host.as_bytes().to_vec()),
                    OwnedValue::integer(i64::from(*port)),
                ])
            })),
        ));
    }
    if let Some(comment) = &options.comment {
        entries.push((
            b"comment".to_vec(),
            OwnedValue::bytes(comment.as_bytes().to_vec()),
        ));
    }
    let created_by = match &options.created_by {
        CreatorIdentity::Default => {
            Some(format!("btpc/{}", env!("CARGO_PKG_VERSION")).into_bytes())
        }
        CreatorIdentity::Explicit(value) => Some(value.clone()),
        CreatorIdentity::Omit => None,
    };
    if let Some(created_by) = created_by {
        entries.push((b"created by".to_vec(), OwnedValue::bytes(created_by)));
    }
    if let Some(creation_date) = options.creation_date {
        entries.push((
            b"creation date".to_vec(),
            OwnedValue::integer(creation_date),
        ));
    }
    OwnedValue::dictionary(entries)
}

fn length_to_i64(length: u64) -> Result<i64> {
    i64::try_from(length).map_err(|_| Error::metainfo_field("length", "cannot fit bencode integer"))
}

#[cfg(test)]
mod parallel_hash_tests {
    use std::fs;

    use super::{
        CancellationToken, ManifestEntry, NoProgress, ParallelHashOptions, automatic_v1_workers,
        hash_v1_parallel_inner, hash_v1_sequential,
    };
    use crate::ErrorCategory;
    use tempfile::TempDir;

    #[test]
    fn pipeline_restores_forced_out_of_order_completion() {
        let (temp, entry) = fixture();
        let entries = [entry];
        let oracle =
            hash_v1_sequential(&entries, 16 * 1024, &CancellationToken::new(), &NoProgress)
                .unwrap();
        let actual = hash_v1_parallel_inner(
            &entries,
            16 * 1024,
            ParallelHashOptions::new(4, 2).unwrap(),
            &CancellationToken::new(),
            &NoProgress,
            None,
            Some(0),
        )
        .unwrap();
        assert_eq!(actual, oracle);
        drop(temp);
    }

    #[test]
    fn injected_worker_failure_is_returned_without_hanging() {
        let (_temp, entry) = fixture();
        let error = hash_v1_parallel_inner(
            &[entry],
            16 * 1024,
            ParallelHashOptions::new(4, 2).unwrap(),
            &CancellationToken::new(),
            &NoProgress,
            Some(5),
            None,
        )
        .unwrap_err();
        assert_eq!(error.category(), ErrorCategory::Io);
    }

    #[test]
    fn automatic_worker_heuristic_is_conservative() {
        assert_eq!(automatic_v1_workers(1), 1);
        assert_eq!(automatic_v1_workers(2), 2);
        assert_eq!(automatic_v1_workers(64), 2);
    }

    fn fixture() -> (TempDir, ManifestEntry) {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("payload");
        fs::write(&path, vec![0x5a; 512 * 1024]).unwrap();
        let metadata = fs::metadata(&path).unwrap();
        let entry = ManifestEntry::from_snapshot(
            path,
            vec![b"payload".to_vec()],
            metadata.len(),
            metadata.modified().ok(),
        );
        (temp, entry)
    }
}

#[cfg(test)]
mod v2_parallel_tests {
    use std::fs;
    use std::sync::atomic::Ordering;

    use super::{
        CancellationToken, HashProgress, ManifestEntry, NoProgress, ProgressSink, WorkerCounts,
        hash_hybrid_v1_parallel, hash_hybrid_v1_sequential, hash_v2_file_sequential,
        hash_v2_files_parallel, hash_v2_files_parallel_inner,
    };
    use crate::ErrorCategory;
    use tempfile::TempDir;

    #[derive(Default)]
    struct RecordingProgress {
        events: std::sync::Mutex<Vec<HashProgress>>,
    }

    struct CancellingProgress {
        cancellation: CancellationToken,
        seen: std::sync::atomic::AtomicBool,
    }

    impl ProgressSink for CancellingProgress {
        fn on_progress(&self, _progress: HashProgress) {
            if !self.seen.swap(true, std::sync::atomic::Ordering::AcqRel) {
                self.cancellation.cancel();
            }
        }
    }

    impl ProgressSink for RecordingProgress {
        fn on_progress(&self, progress: HashProgress) {
            self.events.lock().unwrap().push(progress);
        }
    }

    #[test]
    fn parallel_v2_restores_manifest_order_and_progress() {
        let (temp, entries) = fixture(32, 32 * 1024);
        let progress = RecordingProgress::default();
        let actual =
            hash_v2_files_parallel(&entries, 16 * 1024, 2, &CancellationToken::new(), &progress)
                .unwrap();
        let expected = entries
            .iter()
            .map(|entry| {
                hash_v2_file_sequential(entry, 16 * 1024, &CancellationToken::new(), &NoProgress)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
        let events = progress.events.lock().unwrap();
        assert_eq!(events.len(), entries.len());
        assert!(
            events
                .windows(2)
                .all(|pair| pair[0].bytes_hashed() < pair[1].bytes_hashed())
        );
        drop(temp);
    }

    #[test]
    fn parallel_file_workers_propagate_errors_and_cancellation() {
        let (_temp, mut entries) = fixture(16, 32 * 1024);
        fs::remove_file(entries[5].source_path()).unwrap();
        let error = hash_v2_files_parallel(
            &entries,
            16 * 1024,
            2,
            &CancellationToken::new(),
            &NoProgress,
        )
        .unwrap_err();
        assert_eq!(error.category(), ErrorCategory::Io);

        entries.remove(5);
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let error = hash_hybrid_v1_parallel(&entries, 16 * 1024, 2, &cancellation).unwrap_err();
        assert_eq!(error.category(), ErrorCategory::Cancelled);
    }

    #[test]
    fn parallel_v2_progress_can_cancel_midflight() {
        let (_temp, entries) = fixture(64, 128 * 1024);
        let cancellation = CancellationToken::new();
        let progress = CancellingProgress {
            cancellation: cancellation.clone(),
            seen: std::sync::atomic::AtomicBool::new(false),
        };
        let error =
            hash_v2_files_parallel(&entries, 16 * 1024, 2, &cancellation, &progress).unwrap_err();
        assert_eq!(error.category(), ErrorCategory::Cancelled);
    }

    #[test]
    fn parallel_hybrid_matches_oracle_for_many_files() {
        let (temp, entries) = fixture(64, 4_097);
        let oracle =
            hash_hybrid_v1_sequential(&entries, 16 * 1024, &CancellationToken::new(), &NoProgress)
                .unwrap();
        let actual =
            hash_hybrid_v1_parallel(&entries, 16 * 1024, 2, &CancellationToken::new()).unwrap();
        assert_eq!(actual, oracle);
        drop(temp);
    }

    #[test]
    fn worker_count_is_the_descriptor_bound() {
        let counts = WorkerCounts {
            active: std::sync::atomic::AtomicUsize::new(0),
            peak: std::sync::atomic::AtomicUsize::new(0),
        };
        let (_temp, entries) = fixture(128, 128 * 1024);
        let workers = 2;
        hash_v2_files_parallel_inner(
            &entries,
            16 * 1024,
            workers,
            &CancellationToken::new(),
            &NoProgress,
            Some(&counts),
        )
        .unwrap();
        assert_eq!(counts.active.load(Ordering::Acquire), 0);
        assert!(counts.peak.load(Ordering::Acquire) <= workers);
    }

    fn fixture(count: usize, size: usize) -> (TempDir, Vec<ManifestEntry>) {
        let temp = TempDir::new().unwrap();
        let entries = (0..count)
            .map(|index| {
                let path = temp.path().join(format!("file-{index:03}"));
                fs::write(&path, vec![u8::try_from(index % 256).unwrap(); size]).unwrap();
                let metadata = fs::metadata(&path).unwrap();
                ManifestEntry::from_snapshot(
                    path,
                    vec![format!("file-{index:03}").into_bytes()],
                    metadata.len(),
                    metadata.modified().ok(),
                )
            })
            .collect();
        (temp, entries)
    }
}
