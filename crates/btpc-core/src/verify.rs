//! Safe, deterministic payload verification.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use sha1::Digest as _;

use crate::create::{
    CancellationToken, HashProgress, ManifestEntry, ProgressSink, hash_v2_file_sequential,
};
use crate::metainfo::{RawMetainfo, V1Metainfo, V2Metainfo};
use crate::{Error, Metainfo, Result, TorrentMode};

/// Whether verification stops after its first mismatch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MismatchMode {
    /// Return the first deterministic mismatch.
    FailFast,
    /// Return every mismatch found by enabled checks.
    #[default]
    CollectAll,
}

/// Policy for payload files absent from metainfo.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ExtraFilePolicy {
    /// Do not enumerate unrelated payload files.
    #[default]
    Ignore,
    /// Report unrelated regular files.
    Report,
}

/// Payload mismatch category.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MismatchKind {
    /// Expected path does not exist.
    Missing,
    /// Expected path has a different byte length.
    WrongSize,
    /// Payload contains an unrelated file.
    Extra,
    /// Path crosses a symlink or otherwise escapes safe mapping.
    UnsafePath,
    /// V1 SHA-1 piece data differs.
    V1Hash,
    /// V2 SHA-256 file Merkle root differs.
    V2Hash,
}

/// One deterministic verification mismatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mismatch {
    kind: MismatchKind,
    path: PathBuf,
    piece: Option<u64>,
}

impl Mismatch {
    /// Returns the mismatch category.
    #[must_use]
    pub const fn kind(&self) -> MismatchKind {
        self.kind
    }

    /// Returns the payload-relative path, or the selected root for root-level issues.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the v1 piece index when available.
    #[must_use]
    pub const fn piece(&self) -> Option<u64> {
        self.piece
    }

    /// Returns the stable ordering key used by reports.
    #[must_use]
    pub fn sort_key(&self) -> (PathBuf, MismatchKind, Option<u64>) {
        (self.path.clone(), self.kind, self.piece)
    }
}

/// Completed verification report.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VerificationReport {
    mismatches: Vec<Mismatch>,
}

impl VerificationReport {
    /// Returns true when every enabled check passed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.mismatches.is_empty()
    }

    /// Returns deterministic mismatches.
    #[must_use]
    pub fn mismatches(&self) -> &[Mismatch] {
        &self.mismatches
    }
}

/// Verification configuration.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VerifyOptions {
    mismatch_mode: MismatchMode,
    extra_files: ExtraFilePolicy,
}

impl VerifyOptions {
    /// Starts an options builder.
    #[must_use]
    pub const fn builder() -> VerifyOptionsBuilder {
        VerifyOptionsBuilder(VerifyOptions {
            mismatch_mode: MismatchMode::CollectAll,
            extra_files: ExtraFilePolicy::Ignore,
        })
    }
}

/// Builder for [`VerifyOptions`].
#[derive(Clone, Copy, Debug)]
pub struct VerifyOptionsBuilder(VerifyOptions);

impl VerifyOptionsBuilder {
    /// Selects fail-fast or collect-all reporting.
    #[must_use]
    pub const fn mismatch_mode(mut self, mode: MismatchMode) -> Self {
        self.0.mismatch_mode = mode;
        self
    }

    /// Selects extra-file handling.
    #[must_use]
    pub const fn extra_files(mut self, policy: ExtraFilePolicy) -> Self {
        self.0.extra_files = policy;
        self
    }

    /// Builds verification options.
    #[must_use]
    pub const fn build(self) -> VerifyOptions {
        self.0
    }
}

/// Verifies one validated metainfo object against a payload root.
pub struct Verifier<'a> {
    metainfo: &'a Metainfo,
    payload: PathBuf,
    options: VerifyOptions,
    cancellation: CancellationToken,
}

impl<'a> Verifier<'a> {
    /// Creates a verifier using safe default policies.
    #[must_use]
    pub fn new(metainfo: &'a Metainfo, payload: impl Into<PathBuf>) -> Self {
        Self {
            metainfo,
            payload: payload.into(),
            options: VerifyOptions::builder().build(),
            cancellation: CancellationToken::new(),
        }
    }

    /// Replaces verification options.
    #[must_use]
    pub const fn options(mut self, options: VerifyOptions) -> Self {
        self.options = options;
        self
    }

    /// Replaces the cancellation token.
    #[must_use]
    pub fn cancellation(mut self, cancellation: CancellationToken) -> Self {
        self.cancellation = cancellation;
        self
    }

    /// Verifies structural and applicable hash domains.
    ///
    /// # Errors
    ///
    /// Returns cancellation, metainfo reparse, or operational filesystem errors.
    pub fn verify(&self, progress: &impl ProgressSink) -> Result<VerificationReport> {
        self.check_cancelled()?;
        let raw = RawMetainfo::from_bytes_with_options(
            self.metainfo.original_bytes(),
            self.metainfo.parse_options(),
        )?;
        if let Some(report) = self.root_mismatch()? {
            return Ok(report);
        }
        let payload_files = self
            .metainfo
            .files()
            .iter()
            .filter(|file| !file.is_padding())
            .collect::<Vec<_>>();
        let single_file = payload_files.len() == 1
            && payload_files[0].path_components() == [self.metainfo.name().to_vec()];
        let base = self.payload.clone();
        let mut mismatches = Vec::new();
        let mut expected = BTreeSet::new();
        for file in payload_files {
            let relative = path_from_components(file.path_components())?;
            expected.insert(relative.clone());
            let path = if single_file && base.is_file() {
                base.clone()
            } else {
                base.join(&relative)
            };
            match safe_metadata(&base, &path) {
                Ok(metadata) if metadata.is_file() => {
                    if metadata.len() != file.length() {
                        push_mismatch(
                            &mut mismatches,
                            self.options.mismatch_mode,
                            MismatchKind::WrongSize,
                            relative,
                            None,
                        );
                    }
                }
                Ok(_) | Err(SafePathError::Missing) => push_mismatch(
                    &mut mismatches,
                    self.options.mismatch_mode,
                    MismatchKind::Missing,
                    relative,
                    None,
                ),
                Err(SafePathError::Unsafe) => push_mismatch(
                    &mut mismatches,
                    self.options.mismatch_mode,
                    MismatchKind::UnsafePath,
                    relative,
                    None,
                ),
                Err(SafePathError::Io(error)) => return Err(error),
            }
            if should_stop(&mismatches, self.options.mismatch_mode) {
                return Ok(finish(mismatches));
            }
        }
        if self.options.extra_files == ExtraFilePolicy::Report && base.is_dir() {
            let mut actual = Vec::new();
            collect_files(&base, &base, &mut actual)?;
            for path in actual {
                if !expected.contains(&path) {
                    push_mismatch(
                        &mut mismatches,
                        self.options.mismatch_mode,
                        MismatchKind::Extra,
                        path,
                        None,
                    );
                    if should_stop(&mismatches, self.options.mismatch_mode) {
                        return Ok(finish(mismatches));
                    }
                }
            }
        }
        let structural_mismatch = mismatches.iter().any(|mismatch| {
            matches!(
                mismatch.kind,
                MismatchKind::Missing | MismatchKind::WrongSize | MismatchKind::UnsafePath
            )
        });
        match self.metainfo.mode() {
            TorrentMode::V1 => {
                self.verify_v1(&raw, &base, structural_mismatch, &mut mismatches, progress)?;
            }
            TorrentMode::V2 => self.verify_v2(&raw, &base, &mut mismatches, progress)?,
            TorrentMode::Hybrid => {
                self.verify_v1(&raw, &base, structural_mismatch, &mut mismatches, progress)?;
                if !should_stop(&mismatches, self.options.mismatch_mode) {
                    self.verify_v2(&raw, &base, &mut mismatches, progress)?;
                }
            }
        }
        Ok(finish(mismatches))
    }

    fn verify_v1(
        &self,
        raw: &RawMetainfo<'_>,
        base: &Path,
        structural_mismatch: bool,
        mismatches: &mut Vec<Mismatch>,
        progress: &impl ProgressSink,
    ) -> Result<()> {
        let v1 = V1Metainfo::from_raw(raw)?;
        if structural_mismatch {
            return Ok(());
        }
        let actual = hash_v1_payload(&v1, base, &self.cancellation, progress)?;
        let expected_count = v1.pieces().len() / 20;
        if actual.len() != expected_count {
            push_mismatch(
                mismatches,
                self.options.mismatch_mode,
                MismatchKind::V1Hash,
                PathBuf::new(),
                Some(u64::try_from(actual.len().min(expected_count)).unwrap_or(u64::MAX)),
            );
            if should_stop(mismatches, self.options.mismatch_mode) {
                return Ok(());
            }
        }
        for (index, (expected, actual)) in
            v1.pieces().chunks_exact(20).zip(actual.iter()).enumerate()
        {
            if expected != actual {
                push_mismatch(
                    mismatches,
                    self.options.mismatch_mode,
                    MismatchKind::V1Hash,
                    PathBuf::new(),
                    Some(u64::try_from(index).unwrap_or(u64::MAX)),
                );
                if should_stop(mismatches, self.options.mismatch_mode) {
                    break;
                }
            }
        }
        Ok(())
    }

    fn verify_v2(
        &self,
        raw: &RawMetainfo<'_>,
        base: &Path,
        mismatches: &mut Vec<Mismatch>,
        progress: &impl ProgressSink,
    ) -> Result<()> {
        let v2 = V2Metainfo::from_raw(raw)?;
        let single_file = v2.files().len() == 1 && base.is_file();
        let total_bytes = v2.total_length();
        let mut bytes_before = 0_u64;
        let mut pieces_before = 0_u64;
        for file in v2.files() {
            let relative = path_from_borrowed_components(file.path_components())?;
            let path = if single_file {
                base.to_path_buf()
            } else {
                base.join(&relative)
            };
            let metadata = match safe_metadata(base, &path) {
                Ok(metadata) if metadata.is_file() => metadata,
                Ok(_) | Err(SafePathError::Missing) => {
                    push_mismatch(
                        mismatches,
                        self.options.mismatch_mode,
                        MismatchKind::Missing,
                        relative,
                        None,
                    );
                    if should_stop(mismatches, self.options.mismatch_mode) {
                        break;
                    }
                    continue;
                }
                Err(SafePathError::Unsafe) => {
                    push_mismatch(
                        mismatches,
                        self.options.mismatch_mode,
                        MismatchKind::UnsafePath,
                        relative,
                        None,
                    );
                    if should_stop(mismatches, self.options.mismatch_mode) {
                        break;
                    }
                    continue;
                }
                Err(SafePathError::Io(error)) => return Err(error),
            };
            if metadata.len() != file.length() {
                push_mismatch(
                    mismatches,
                    self.options.mismatch_mode,
                    MismatchKind::WrongSize,
                    relative,
                    None,
                );
                if should_stop(mismatches, self.options.mismatch_mode) {
                    break;
                }
                continue;
            }
            let entry = ManifestEntry::from_verified_path(
                &path,
                file.path_components()
                    .iter()
                    .map(|part| part.to_vec())
                    .collect(),
                &metadata,
            )?;
            let aggregate = VerifyProgress {
                inner: progress,
                bytes_before,
                pieces_before,
                total_bytes,
            };
            let actual =
                hash_v2_file_sequential(&entry, v2.piece_length(), &self.cancellation, &aggregate)?;
            bytes_before = bytes_before.checked_add(file.length()).ok_or_else(|| {
                Error::metainfo_field("verification progress", "byte count overflowed")
            })?;
            pieces_before = pieces_before
                .checked_add(file.length().div_ceil(v2.piece_length()))
                .ok_or_else(|| {
                    Error::metainfo_field("verification progress", "piece count overflowed")
                })?;
            if actual.pieces_root() != file.pieces_root() {
                push_mismatch(
                    mismatches,
                    self.options.mismatch_mode,
                    MismatchKind::V2Hash,
                    relative,
                    None,
                );
                if should_stop(mismatches, self.options.mismatch_mode) {
                    break;
                }
            }
        }
        Ok(())
    }

    fn root_mismatch(&self) -> Result<Option<VerificationReport>> {
        match fs::symlink_metadata(&self.payload) {
            Ok(metadata) if metadata.file_type().is_symlink() => Ok(Some(finish(vec![Mismatch {
                kind: MismatchKind::UnsafePath,
                path: PathBuf::new(),
                piece: None,
            }]))),
            Ok(_) => Ok(None),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                Ok(Some(finish(vec![Mismatch {
                    kind: MismatchKind::Missing,
                    path: PathBuf::new(),
                    piece: None,
                }])))
            }
            Err(source) => Err(Error::io(&self.payload, source)),
        }
    }

    fn check_cancelled(&self) -> Result<()> {
        if self.cancellation.is_cancelled() {
            Err(Error::cancelled())
        } else {
            Ok(())
        }
    }
}

fn hash_v1_payload(
    v1: &V1Metainfo<'_>,
    base: &Path,
    cancellation: &CancellationToken,
    progress: &impl ProgressSink,
) -> Result<Vec<[u8; 20]>> {
    let piece_length = usize::try_from(v1.piece_length())
        .map_err(|_| Error::metainfo_field("piece length", "cannot be represented"))?;
    let single_file = v1.is_single_file() && base.is_file();
    let total_real = crate::metainfo::checked_total_length(
        v1.files()
            .iter()
            .filter(|file| !file.is_padding())
            .map(crate::metainfo::V1File::length),
        "verification payload length",
    )?;
    let mut piece = Vec::with_capacity(piece_length);
    let mut pieces = Vec::new();
    let mut bytes_hashed = 0_u64;
    let mut buffer = vec![0_u8; 64 * 1024];
    for file in v1.files() {
        if cancellation.is_cancelled() {
            return Err(Error::cancelled());
        }
        if file.is_padding() {
            let mut remaining = file.length();
            while remaining > 0 {
                let take = usize::try_from(remaining)
                    .unwrap_or(usize::MAX)
                    .min(buffer.len());
                buffer[..take].fill(0);
                append_piece_bytes(&buffer[..take], piece_length, &mut piece, &mut pieces);
                remaining -= u64::try_from(take).unwrap_or(u64::MAX);
            }
            continue;
        }
        let relative = path_from_borrowed_components(file.path_components())?;
        let path = if single_file {
            base.to_path_buf()
        } else {
            base.join(relative)
        };
        let metadata = safe_metadata(base, &path).map_err(|error| match error {
            SafePathError::Missing => Error::io(
                &path,
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "payload disappeared before hashing",
                ),
            ),
            SafePathError::Unsafe => Error::io(
                &path,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "payload path became unsafe before hashing",
                ),
            ),
            SafePathError::Io(error) => error,
        })?;
        let entry = ManifestEntry::from_verified_path(
            &path,
            file.path_components()
                .iter()
                .map(|part| part.to_vec())
                .collect(),
            &metadata,
        )?;
        let mut input = entry.open_verified()?;
        loop {
            if cancellation.is_cancelled() {
                return Err(Error::cancelled());
            }
            let read = std::io::Read::read(&mut input, &mut buffer)
                .map_err(|source| Error::io(&path, source))?;
            if read == 0 {
                break;
            }
            append_piece_bytes(&buffer[..read], piece_length, &mut piece, &mut pieces);
            bytes_hashed += u64::try_from(read).unwrap_or(u64::MAX);
            progress.on_progress(HashProgress::new(
                bytes_hashed,
                total_real,
                u64::try_from(pieces.len()).unwrap_or(u64::MAX),
            ));
        }
        entry.verify_opened(&input)?;
    }
    if !piece.is_empty() {
        pieces.push(sha1::Sha1::digest(&piece).into());
    }
    Ok(pieces)
}

struct VerifyProgress<'a, P> {
    inner: &'a P,
    bytes_before: u64,
    pieces_before: u64,
    total_bytes: u64,
}

impl<P: ProgressSink> ProgressSink for VerifyProgress<'_, P> {
    fn on_progress(&self, progress: HashProgress) {
        self.inner.on_progress(HashProgress::new(
            self.bytes_before
                .checked_add(progress.bytes_hashed())
                .expect("verified byte progress fits the checked aggregate"),
            self.total_bytes,
            self.pieces_before
                .checked_add(progress.pieces_hashed())
                .expect("verified piece progress fits the checked aggregate"),
        ));
    }
}

fn append_piece_bytes(
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

fn safe_metadata(base: &Path, path: &Path) -> std::result::Result<fs::Metadata, SafePathError> {
    let base_metadata = fs::symlink_metadata(base).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            SafePathError::Missing
        } else {
            SafePathError::Io(Error::io(base, source))
        }
    })?;
    if base_metadata.file_type().is_symlink() {
        return Err(SafePathError::Unsafe);
    }
    if base_metadata.is_file() {
        return fs::metadata(base).map_err(|source| SafePathError::Io(Error::io(base, source)));
    }
    let relative = path.strip_prefix(base).map_err(|_| SafePathError::Unsafe)?;
    let mut current = base.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current).map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                SafePathError::Missing
            } else {
                SafePathError::Io(Error::io(&current, source))
            }
        })?;
        if metadata.file_type().is_symlink() {
            return Err(SafePathError::Unsafe);
        }
    }
    fs::metadata(path).map_err(|source| SafePathError::Io(Error::io(path, source)))
}

enum SafePathError {
    Missing,
    Unsafe,
    Io(Error),
}

fn collect_files(base: &Path, directory: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .map_err(|source| Error::io(directory, source))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|source| Error::io(directory, source))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| Error::io(&path, source))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_files(base, &path, output)?;
        } else if metadata.is_file() {
            output.push(
                path.strip_prefix(base)
                    .expect("walk remains under base")
                    .to_path_buf(),
            );
        }
    }
    Ok(())
}

fn path_from_components(components: &[Vec<u8>]) -> Result<PathBuf> {
    crate::TorrentPath::from_raw(components).to_path_buf()
}

fn path_from_borrowed_components(components: &[&[u8]]) -> Result<PathBuf> {
    components
        .iter()
        .try_fold(PathBuf::new(), |mut path, component| {
            path.push(os_string(component));
            Ok(path)
        })
}

#[cfg(unix)]
fn os_string(bytes: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStringExt as _;
    OsString::from_vec(bytes.to_vec())
}

#[cfg(not(unix))]
fn os_string(bytes: &[u8]) -> OsString {
    OsString::from(std::str::from_utf8(bytes).expect("validated platform-compatible path"))
}

fn push_mismatch(
    mismatches: &mut Vec<Mismatch>,
    mode: MismatchMode,
    kind: MismatchKind,
    path: PathBuf,
    piece: Option<u64>,
) {
    if mode == MismatchMode::CollectAll || mismatches.is_empty() {
        mismatches.push(Mismatch { kind, path, piece });
    }
}

fn should_stop(mismatches: &[Mismatch], mode: MismatchMode) -> bool {
    mode == MismatchMode::FailFast && !mismatches.is_empty()
}

fn finish(mut mismatches: Vec<Mismatch>) -> VerificationReport {
    mismatches.sort_by_key(Mismatch::sort_key);
    mismatches.dedup();
    VerificationReport { mismatches }
}
