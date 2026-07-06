//! Safe, deterministic payload verification.

mod safe_fs;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use sha1::Digest as _;

use crate::create::{CancellationToken, HashProgress, ProgressSink, hash_v2_open_file_sequential};
use crate::metainfo::{RawMetainfo, V1Metainfo, V2Metainfo};
use crate::{Error, Metainfo, Result, TorrentMode};
use safe_fs::{OpenedFile, SafePathError, SafeRoot};

#[cfg(test)]
type TestHook = std::sync::Arc<dyn Fn(TestEvent) + Send + Sync>;

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
enum TestEvent {
    AfterStructure,
    BeforeExpectedOpen(PathBuf),
    BeforeExtraOpen(PathBuf),
}

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
    #[cfg(test)]
    test_hook: Option<TestHook>,
}

struct OpenPayload {
    expected: BTreeSet<PathBuf>,
    opened: BTreeMap<PathBuf, OpenedFile>,
    mismatches: Vec<Mismatch>,
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
            #[cfg(test)]
            test_hook: None,
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

    #[cfg(test)]
    fn test_hook(mut self, hook: TestHook) -> Self {
        self.test_hook = Some(hook);
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
        let root = match self.open_root()? {
            Ok(root) => root,
            Err(report) => return Ok(report),
        };
        let OpenPayload {
            expected,
            mut opened,
            mut mismatches,
        } = self.open_expected_files(&root)?;
        self.report_extras(&root, &expected, &mut mismatches)?;
        #[cfg(test)]
        if let Some(hook) = &self.test_hook {
            hook(TestEvent::AfterStructure);
        }
        self.revalidate_opened(&root, &opened, &mut mismatches)?;
        let structural_mismatch = mismatches.iter().any(|mismatch| {
            matches!(
                mismatch.kind,
                MismatchKind::Missing | MismatchKind::WrongSize | MismatchKind::UnsafePath
            )
        });
        match self.metainfo.mode() {
            TorrentMode::V1 => {
                self.verify_v1(
                    &raw,
                    &root,
                    &mut opened,
                    structural_mismatch,
                    &mut mismatches,
                    progress,
                )?;
            }
            TorrentMode::V2 => {
                self.verify_v2(&raw, &root, &mut opened, &mut mismatches, progress)?;
            }
            TorrentMode::Hybrid => {
                self.verify_v1(
                    &raw,
                    &root,
                    &mut opened,
                    structural_mismatch,
                    &mut mismatches,
                    progress,
                )?;
                if !should_stop(&mismatches, self.options.mismatch_mode) {
                    self.verify_v2(&raw, &root, &mut opened, &mut mismatches, progress)?;
                }
            }
        }
        Ok(finish(mismatches))
    }

    fn open_root(&self) -> Result<std::result::Result<SafeRoot, VerificationReport>> {
        match SafeRoot::open(&self.payload) {
            Ok(root) => Ok(Ok(root)),
            Err(SafePathError::Missing) => Ok(Err(root_report(MismatchKind::Missing))),
            Err(SafePathError::Unsafe) => Ok(Err(root_report(MismatchKind::UnsafePath))),
            Err(SafePathError::Io(error)) => Err(error),
        }
    }

    fn open_expected_files(&self, root: &SafeRoot) -> Result<OpenPayload> {
        let payload_files = self
            .metainfo
            .files()
            .iter()
            .filter(|file| !file.is_padding())
            .collect::<Vec<_>>();
        let single_file = payload_files.len() == 1
            && payload_files[0].path_components() == [self.metainfo.name().to_vec()];
        if root.is_file() && !single_file {
            return Ok(OpenPayload {
                expected: BTreeSet::new(),
                opened: BTreeMap::new(),
                mismatches: root_mismatches(MismatchKind::Missing),
            });
        }
        let mut expected = BTreeSet::new();
        let mut opened = BTreeMap::new();
        let mut mismatches = Vec::new();
        for file in payload_files {
            let relative = path_from_components(file.path_components())?;
            expected.insert(relative.clone());
            #[cfg(test)]
            if let Some(hook) = &self.test_hook {
                hook(TestEvent::BeforeExpectedOpen(relative.clone()));
            }
            match root.open_file(&relative) {
                Ok(payload_file) if payload_file.length() == file.length() => {
                    opened.insert(relative, payload_file);
                }
                Ok(_) => push_mismatch(
                    &mut mismatches,
                    self.options.mismatch_mode,
                    MismatchKind::WrongSize,
                    relative,
                    None,
                ),
                Err(SafePathError::Missing) => push_mismatch(
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
                break;
            }
        }
        Ok(OpenPayload {
            expected,
            opened,
            mismatches,
        })
    }

    fn report_extras(
        &self,
        root: &SafeRoot,
        expected: &BTreeSet<PathBuf>,
        mismatches: &mut Vec<Mismatch>,
    ) -> Result<()> {
        if self.options.extra_files != ExtraFilePolicy::Report || !root.is_directory() {
            return Ok(());
        }
        let (actual, unsafe_paths) = root.collect_files(&|path| {
            #[cfg(not(test))]
            let _ = path;
            #[cfg(test)]
            if let Some(hook) = &self.test_hook {
                hook(TestEvent::BeforeExtraOpen(path.to_path_buf()));
            }
        })?;
        for path in unsafe_paths {
            push_mismatch(
                mismatches,
                self.options.mismatch_mode,
                MismatchKind::UnsafePath,
                path,
                None,
            );
        }
        for path in actual.into_iter().filter(|path| !expected.contains(path)) {
            push_mismatch(
                mismatches,
                self.options.mismatch_mode,
                MismatchKind::Extra,
                path,
                None,
            );
            if should_stop(mismatches, self.options.mismatch_mode) {
                break;
            }
        }
        Ok(())
    }

    fn revalidate_opened(
        &self,
        root: &SafeRoot,
        opened: &BTreeMap<PathBuf, OpenedFile>,
        mismatches: &mut Vec<Mismatch>,
    ) -> Result<()> {
        for (relative, payload_file) in opened {
            let kind = match root.same_file(relative, payload_file) {
                Ok(true) => continue,
                Ok(false) | Err(SafePathError::Unsafe) => MismatchKind::UnsafePath,
                Err(SafePathError::Missing) => MismatchKind::Missing,
                Err(SafePathError::Io(error)) => return Err(error),
            };
            push_mismatch(
                mismatches,
                self.options.mismatch_mode,
                kind,
                relative.clone(),
                None,
            );
        }
        Ok(())
    }

    fn verify_v1(
        &self,
        raw: &RawMetainfo<'_>,
        root: &SafeRoot,
        opened: &mut BTreeMap<PathBuf, OpenedFile>,
        structural_mismatch: bool,
        mismatches: &mut Vec<Mismatch>,
        progress: &impl ProgressSink,
    ) -> Result<()> {
        let v1 = V1Metainfo::from_raw(raw)?;
        if structural_mismatch {
            return Ok(());
        }
        let actual = hash_v1_payload(&v1, root, opened, &self.cancellation, progress)?;
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
        root: &SafeRoot,
        opened: &mut BTreeMap<PathBuf, OpenedFile>,
        mismatches: &mut Vec<Mismatch>,
        progress: &impl ProgressSink,
    ) -> Result<()> {
        let v2 = V2Metainfo::from_raw(raw)?;
        let total_bytes = v2.total_length();
        let mut bytes_before = 0_u64;
        let mut pieces_before = 0_u64;
        for file in v2.files() {
            let relative = path_from_borrowed_components(file.path_components())?;
            let Some(payload_file) = opened.get_mut(&relative) else {
                continue;
            };
            let path = root.display_path(&relative);
            payload_file.rewind(&path)?;
            let aggregate = VerifyProgress {
                inner: progress,
                bytes_before,
                pieces_before,
                total_bytes,
            };
            let actual = hash_v2_open_file_sequential(
                payload_file.file_mut(),
                &path,
                file.length(),
                v2.piece_length(),
                &self.cancellation,
                &aggregate,
            )?;
            if !payload_file.unchanged(&path)? || !same_opened_file(root, &relative, payload_file)?
            {
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
    root: &SafeRoot,
    opened: &mut BTreeMap<PathBuf, OpenedFile>,
    cancellation: &CancellationToken,
    progress: &impl ProgressSink,
) -> Result<Vec<[u8; 20]>> {
    let piece_length = usize::try_from(v1.piece_length())
        .map_err(|_| Error::metainfo_field("piece length", "cannot be represented"))?;
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
        let path = root.display_path(&relative);
        let input = opened.get_mut(&relative).ok_or_else(|| {
            Error::io(
                &path,
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "payload disappeared before hashing",
                ),
            )
        })?;
        input.rewind(&path)?;
        loop {
            if cancellation.is_cancelled() {
                return Err(Error::cancelled());
            }
            let read = std::io::Read::read(input.file_mut(), &mut buffer)
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
        if !input.unchanged(&path)? || !same_opened_file(root, &relative, input)? {
            return Err(Error::io(
                &path,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "payload changed during verification",
                ),
            ));
        }
    }
    if !piece.is_empty() {
        pieces.push(sha1::Sha1::digest(&piece).into());
    }
    Ok(pieces)
}

fn same_opened_file(root: &SafeRoot, relative: &Path, opened: &OpenedFile) -> Result<bool> {
    match root.same_file(relative, opened) {
        Ok(same) => Ok(same),
        Err(SafePathError::Missing | SafePathError::Unsafe) => Ok(false),
        Err(SafePathError::Io(error)) => Err(error),
    }
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

fn root_mismatches(kind: MismatchKind) -> Vec<Mismatch> {
    vec![Mismatch {
        kind,
        path: PathBuf::new(),
        piece: None,
    }]
}

fn root_report(kind: MismatchKind) -> VerificationReport {
    finish(root_mismatches(kind))
}

fn should_stop(mismatches: &[Mismatch], mode: MismatchMode) -> bool {
    mode == MismatchMode::FailFast && !mismatches.is_empty()
}

fn finish(mut mismatches: Vec<Mismatch>) -> VerificationReport {
    mismatches.sort_by_key(Mismatch::sort_key);
    mismatches.dedup();
    VerificationReport { mismatches }
}

#[cfg(test)]
mod race_tests {
    use std::fs;
    use std::sync::{Arc, Mutex};

    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    use crate::Metainfo;
    use crate::create::{CreateMode, CreateOptions, Creator, NoProgress, PieceLength};

    #[cfg(unix)]
    use super::{ExtraFilePolicy, VerifyOptions};
    use super::{MismatchKind, TestEvent, Verifier};

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

    #[cfg(unix)]
    #[test]
    fn replacing_verified_file_with_outside_symlink_never_returns_valid() {
        for mode in [CreateMode::V1, CreateMode::V2, CreateMode::Hybrid] {
            let temp = tempfile::tempdir().unwrap();
            let payload = temp.path().join("payload");
            fs::create_dir(&payload).unwrap();
            fs::write(payload.join("file"), b"safe payload").unwrap();
            let metainfo = torrent(&payload, mode);
            let outside = temp.path().join("outside");
            fs::write(&outside, b"safe payload").unwrap();
            let target = payload.join("file");
            let hook = Arc::new(move |event| {
                if event == TestEvent::AfterStructure {
                    fs::remove_file(&target).unwrap();
                    symlink(&outside, &target).unwrap();
                }
            });

            let report = Verifier::new(&metainfo, &payload)
                .test_hook(hook)
                .verify(&NoProgress)
                .unwrap();
            assert!(!report.is_valid(), "{mode:?}");
            assert!(
                report
                    .mismatches()
                    .iter()
                    .any(|mismatch| mismatch.kind() == MismatchKind::UnsafePath),
                "{mode:?}: {:?}",
                report.mismatches()
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn replacing_intermediate_directory_before_open_is_rejected_repeatedly() {
        for iteration in 0..16 {
            for mode in [CreateMode::V1, CreateMode::V2, CreateMode::Hybrid] {
                let temp = tempfile::tempdir().unwrap();
                let payload = temp.path().join("payload");
                fs::create_dir_all(payload.join("nested")).unwrap();
                fs::write(payload.join("nested/file"), b"safe payload").unwrap();
                let metainfo = torrent(&payload, mode);
                let outside = temp.path().join("outside");
                fs::create_dir(&outside).unwrap();
                fs::write(outside.join("file"), b"safe payload").unwrap();
                let nested = payload.join("nested");
                let original = payload.join("original");
                let replaced = Arc::new(Mutex::new(false));
                let hook_replaced = Arc::clone(&replaced);
                let hook = Arc::new(move |event| {
                    if event
                        == TestEvent::BeforeExpectedOpen(std::path::PathBuf::from("nested/file"))
                        && !*hook_replaced.lock().unwrap()
                    {
                        fs::rename(&nested, &original).unwrap();
                        symlink(&outside, &nested).unwrap();
                        *hook_replaced.lock().unwrap() = true;
                    }
                });
                let report = Verifier::new(&metainfo, &payload)
                    .test_hook(hook)
                    .verify(&NoProgress)
                    .unwrap();
                assert!(!report.is_valid(), "iteration {iteration}, {mode:?}");
                assert!(
                    report
                        .mismatches()
                        .iter()
                        .any(|mismatch| mismatch.kind() == MismatchKind::UnsafePath),
                    "iteration {iteration}, {mode:?}: {:?}",
                    report.mismatches()
                );
            }
        }
    }

    #[cfg(windows)]
    #[test]
    fn replacing_intermediate_directory_with_junction_is_rejected_repeatedly() {
        for iteration in 0..16 {
            for mode in [CreateMode::V1, CreateMode::V2, CreateMode::Hybrid] {
                let temp = tempfile::tempdir().unwrap();
                let payload = temp.path().join("payload");
                fs::create_dir_all(payload.join("nested")).unwrap();
                fs::write(payload.join("nested/file"), b"safe payload").unwrap();
                let metainfo = torrent(&payload, mode);
                let outside = temp.path().join("outside");
                fs::create_dir(&outside).unwrap();
                fs::write(outside.join("file"), b"safe payload").unwrap();
                let nested = payload.join("nested");
                let original = payload.join("original");
                let replaced = Arc::new(Mutex::new(false));
                let hook_replaced = Arc::clone(&replaced);
                let hook = Arc::new(move |event| {
                    if event
                        == TestEvent::BeforeExpectedOpen(std::path::PathBuf::from("nested/file"))
                        && !*hook_replaced.lock().unwrap()
                    {
                        fs::rename(&nested, &original).unwrap();
                        junction::create(&outside, &nested).unwrap();
                        *hook_replaced.lock().unwrap() = true;
                    }
                });
                let report = Verifier::new(&metainfo, &payload)
                    .test_hook(hook)
                    .verify(&NoProgress)
                    .unwrap();
                assert!(!report.is_valid(), "iteration {iteration}, {mode:?}");
                assert!(
                    report
                        .mismatches()
                        .iter()
                        .any(|mismatch| mismatch.kind() == MismatchKind::UnsafePath),
                    "iteration {iteration}, {mode:?}: {:?}",
                    report.mismatches()
                );
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn replacing_enumerated_directory_never_reports_outside_extra_file() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload");
        fs::create_dir_all(payload.join("nested")).unwrap();
        fs::write(payload.join("expected"), b"expected").unwrap();
        fs::write(payload.join("nested/local"), b"local").unwrap();
        let metainfo = torrent(&payload, CreateMode::V2);
        fs::remove_file(payload.join("nested/local")).unwrap();
        let outside = temp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("secret"), b"outside").unwrap();
        let nested = payload.join("nested");
        let replaced = Arc::new(Mutex::new(false));
        let hook_replaced = Arc::clone(&replaced);
        let hook = Arc::new(move |event| {
            if event == TestEvent::BeforeExtraOpen(std::path::PathBuf::from("nested"))
                && !*hook_replaced.lock().unwrap()
            {
                fs::remove_dir(&nested).unwrap();
                symlink(&outside, &nested).unwrap();
                *hook_replaced.lock().unwrap() = true;
            }
        });
        let options = VerifyOptions::builder()
            .extra_files(ExtraFilePolicy::Report)
            .build();

        let report = Verifier::new(&metainfo, &payload)
            .options(options)
            .test_hook(hook)
            .verify(&NoProgress)
            .unwrap();
        assert!(!report.is_valid());
        assert!(
            report
                .mismatches()
                .iter()
                .any(|mismatch| mismatch.kind() == MismatchKind::UnsafePath)
        );
        assert!(
            report
                .mismatches()
                .iter()
                .all(|mismatch| mismatch.path() != std::path::Path::new("nested/secret"))
        );
    }
}
