use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

use cap_fs_ext::{DirExt as _, FollowSymlinks, MetadataExt as _, OpenOptionsFollowExt as _};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};

use crate::{Error, Result};

pub(super) enum SafePathError {
    Missing,
    Unsafe,
    Io(Error),
}

pub(super) struct SafeRoot {
    path: PathBuf,
    inner: RootKind,
    names: RefCell<BTreeMap<(u64, u64), BTreeSet<OsString>>>,
}

enum RootKind {
    Directory(Dir),
    File {
        parent: Dir,
        name: OsString,
        opened: OpenedFile,
    },
}

pub(super) struct OpenedFile {
    file: fs::File,
    snapshot: FileSnapshot,
}

#[derive(Clone, Eq, PartialEq)]
pub(super) struct FileSnapshot {
    identity: (u64, u64),
    state: FileState,
}

impl FileSnapshot {
    fn from_file(file: &fs::File, display_path: &Path) -> Result<Self> {
        let metadata = file
            .metadata()
            .map_err(|source| Error::io(display_path, source))?;
        let identity = cap_std::fs::Metadata::from_file(file)
            .map_err(|source| Error::io(display_path, source))?;
        Ok(Self {
            identity: (identity.dev(), identity.ino()),
            state: FileState::from_metadata(&metadata),
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
struct FileState {
    length: u64,
    modified: Option<std::time::SystemTime>,
    created: Option<std::time::SystemTime>,
    #[cfg(unix)]
    change_seconds: i64,
    #[cfg(unix)]
    change_nanoseconds: i64,
}

impl FileState {
    fn from_metadata(metadata: &fs::Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt as _;

        Self {
            length: metadata.len(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            #[cfg(unix)]
            change_seconds: metadata.ctime(),
            #[cfg(unix)]
            change_nanoseconds: metadata.ctime_nsec(),
        }
    }
}

impl OpenedFile {
    fn new(file: fs::File, display_path: &Path) -> Result<Self> {
        let metadata = file
            .metadata()
            .map_err(|source| Error::io(display_path, source))?;
        if !metadata.is_file() {
            return Err(Error::io(
                display_path,
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "payload is not a file"),
            ));
        }
        let snapshot = FileSnapshot::from_file(&file, display_path)?;
        Ok(Self { file, snapshot })
    }

    pub(super) fn length(&self) -> u64 {
        self.snapshot.state.length
    }

    pub(super) fn file_mut(&mut self) -> &mut fs::File {
        &mut self.file
    }

    pub(super) fn snapshot(&self) -> FileSnapshot {
        self.snapshot.clone()
    }

    pub(super) fn rewind(&mut self, display_path: &Path) -> Result<()> {
        std::io::Seek::seek(&mut self.file, std::io::SeekFrom::Start(0))
            .map(|_| ())
            .map_err(|source| Error::io(display_path, source))
    }

    pub(super) fn unchanged(&self, display_path: &Path) -> Result<bool> {
        FileSnapshot::from_file(&self.file, display_path).map(|snapshot| snapshot == self.snapshot)
    }
}

impl SafeRoot {
    pub(super) fn open(path: &Path) -> std::result::Result<Self, SafePathError> {
        let metadata = fs::symlink_metadata(path).map_err(|source| classify_io(path, source))?;
        if metadata.file_type().is_symlink() {
            return Err(SafePathError::Unsafe);
        }
        if metadata.is_dir() && path.parent().is_none() {
            let directory = Dir::open_ambient_dir(path, ambient_authority())
                .map_err(|source| classify_io(path, source))?;
            return Ok(Self {
                path: path.to_path_buf(),
                inner: RootKind::Directory(directory),
                names: RefCell::default(),
            });
        }
        let parent_path = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let name = path
            .file_name()
            .ok_or(SafePathError::Unsafe)?
            .to_os_string();
        let parent = Dir::open_ambient_dir(parent_path, ambient_authority())
            .map_err(|source| classify_io(parent_path, source))?;
        if metadata.is_dir() {
            let directory = parent
                .open_dir_nofollow(&name)
                .map_err(|source| classify_open(&parent, &name, path, source))?;
            Ok(Self {
                path: path.to_path_buf(),
                inner: RootKind::Directory(directory),
                names: RefCell::default(),
            })
        } else if metadata.is_file() {
            let file = open_file_nofollow(&parent, &name, path)?;
            let opened = OpenedFile::new(file, path).map_err(SafePathError::Io)?;
            Ok(Self {
                path: path.to_path_buf(),
                inner: RootKind::File {
                    parent,
                    name,
                    opened,
                },
                names: RefCell::default(),
            })
        } else {
            Err(SafePathError::Missing)
        }
    }

    pub(super) fn is_file(&self) -> bool {
        matches!(self.inner, RootKind::File { .. })
    }

    pub(super) fn is_directory(&self) -> bool {
        matches!(self.inner, RootKind::Directory(_))
    }

    pub(super) fn display_path(&self, relative: &Path) -> PathBuf {
        if self.is_file() {
            self.path.clone()
        } else {
            self.path.join(relative)
        }
    }

    pub(super) fn open_file(
        &self,
        relative: &Path,
    ) -> std::result::Result<OpenedFile, SafePathError> {
        match &self.inner {
            RootKind::File { opened, .. } => {
                let file = opened
                    .file
                    .try_clone()
                    .map_err(|source| SafePathError::Io(Error::io(&self.path, source)))?;
                OpenedFile::new(file, &self.path).map_err(SafePathError::Io)
            }
            RootKind::Directory(directory) => {
                let (parent, name) = self.open_parent(directory, relative)?;
                let display = self.path.join(relative);
                let file = open_file_nofollow(&parent, &name, &display)?;
                OpenedFile::new(file, &display).map_err(SafePathError::Io)
            }
        }
    }

    pub(super) fn same_file(
        &self,
        relative: &Path,
        snapshot: &FileSnapshot,
    ) -> std::result::Result<bool, SafePathError> {
        let current = match &self.inner {
            RootKind::File { parent, name, .. } => open_file_nofollow(parent, name, &self.path)?,
            RootKind::Directory(directory) => {
                let (parent, name) = self.open_parent(directory, relative)?;
                open_file_nofollow(&parent, &name, &self.path.join(relative))?
            }
        };
        let current = FileSnapshot::from_file(&current, &self.display_path(relative))
            .map_err(SafePathError::Io)?;
        Ok(current == *snapshot)
    }

    pub(super) fn refresh_names(&self) {
        self.names.borrow_mut().clear();
    }

    fn exact_name(
        &self,
        directory: &Dir,
        name: &std::ffi::OsStr,
        display: &Path,
    ) -> std::result::Result<(), SafePathError> {
        let metadata = directory
            .dir_metadata()
            .map_err(|source| SafePathError::Io(Error::io(display, source)))?;
        let identity = (metadata.dev(), metadata.ino());
        let mut names = self.names.borrow_mut();
        if let std::collections::btree_map::Entry::Vacant(entry) = names.entry(identity) {
            let entries = directory
                .entries()
                .map_err(|source| SafePathError::Io(Error::io(display, source)))?;
            let found = entries
                .map(|entry| entry.map(|entry| entry.file_name()))
                .collect::<std::io::Result<BTreeSet<_>>>()
                .map_err(|source| SafePathError::Io(Error::io(display, source)))?;
            entry.insert(found);
        }
        if names[&identity].contains(name) {
            return Ok(());
        }
        // A lookup that succeeds without the exact name aliases a directory entry.
        match directory.symlink_metadata(name) {
            Ok(_) => Err(SafePathError::Unsafe),
            Err(source) => Err(classify_io(display, source)),
        }
    }

    fn open_parent(
        &self,
        root: &Dir,
        relative: &Path,
    ) -> std::result::Result<(Dir, OsString), SafePathError> {
        let mut components = relative.components().peekable();
        let mut directory = root
            .try_clone()
            .map_err(|source| SafePathError::Io(Error::io(&self.path, source)))?;
        while let Some(component) = components.next() {
            let Component::Normal(name) = component else {
                return Err(SafePathError::Unsafe);
            };
            let display = self.path.join(relative);
            self.exact_name(&directory, name, &display)?;
            if components.peek().is_none() {
                return Ok((directory, name.to_os_string()));
            }
            directory = directory
                .open_dir_nofollow(name)
                .map_err(|source| classify_open(&directory, name, &display, source))?;
        }
        Err(SafePathError::Unsafe)
    }

    pub(super) fn collect_files(
        &self,
        before_open: &impl Fn(&Path),
    ) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        let RootKind::Directory(directory) = &self.inner else {
            return Ok((Vec::new(), Vec::new()));
        };
        let mut files = Vec::new();
        let mut unsafe_paths = Vec::new();
        collect_directory(
            directory,
            Path::new(""),
            &self.path,
            before_open,
            &mut files,
            &mut unsafe_paths,
        )?;
        Ok((files, unsafe_paths))
    }
}

fn open_file_nofollow(
    directory: &Dir,
    name: &std::ffi::OsStr,
    display_path: &Path,
) -> std::result::Result<fs::File, SafePathError> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    directory
        .open_with(name, &options)
        .map(cap_std::fs::File::into_std)
        .map_err(|source| classify_open(directory, name, display_path, source))
}

fn collect_directory(
    directory: &Dir,
    relative: &Path,
    root_path: &Path,
    before_open: &impl Fn(&Path),
    files: &mut Vec<PathBuf>,
    unsafe_paths: &mut Vec<PathBuf>,
) -> Result<()> {
    let display = root_path.join(relative);
    let mut entries = directory
        .entries()
        .map_err(|source| Error::io(&display, source))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|source| Error::io(&display, source))?;
    entries.sort_by_key(cap_std::fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name();
        let path = relative.join(&name);
        let file_type = entry
            .file_type()
            .map_err(|source| Error::io(root_path.join(&path), source))?;
        if file_type.is_symlink() {
            continue;
        }
        before_open(&path);
        if file_type.is_dir() {
            match directory.open_dir_nofollow(&name) {
                Ok(child) => {
                    collect_directory(&child, &path, root_path, before_open, files, unsafe_paths)?;
                }
                Err(source) => {
                    match classify_open(directory, &name, &root_path.join(&path), source) {
                        SafePathError::Unsafe | SafePathError::Missing => unsafe_paths.push(path),
                        SafePathError::Io(error) => return Err(error),
                    }
                }
            }
        } else if file_type.is_file() {
            match open_file_nofollow(directory, &name, &root_path.join(&path)) {
                Ok(_) => files.push(path),
                Err(SafePathError::Unsafe | SafePathError::Missing) => unsafe_paths.push(path),
                Err(SafePathError::Io(error)) => return Err(error),
            }
        }
    }
    Ok(())
}

fn classify_open(
    directory: &Dir,
    name: &std::ffi::OsStr,
    display_path: &Path,
    source: std::io::Error,
) -> SafePathError {
    match directory.symlink_metadata(name) {
        Ok(metadata) if metadata.file_type().is_symlink() => SafePathError::Unsafe,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => SafePathError::Missing,
        _ if source.kind() == std::io::ErrorKind::NotFound => SafePathError::Missing,
        _ => SafePathError::Io(Error::io(display_path, source)),
    }
}

fn classify_io(path: &Path, source: std::io::Error) -> SafePathError {
    if source.kind() == std::io::ErrorKind::NotFound {
        SafePathError::Missing
    } else {
        SafePathError::Io(Error::io(path, source))
    }
}
