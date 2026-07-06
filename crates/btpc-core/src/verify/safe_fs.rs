use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

use cap_fs_ext::{DirExt as _, FollowSymlinks, OpenOptionsFollowExt as _};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use same_file::Handle;

use crate::{Error, Result};

pub(super) enum SafePathError {
    Missing,
    Unsafe,
    Io(Error),
}

pub(super) struct SafeRoot {
    path: PathBuf,
    inner: RootKind,
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
    identity: Handle,
    state: FileState,
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
        let identity = Handle::from_file(
            file.try_clone()
                .map_err(|source| Error::io(display_path, source))?,
        )
        .map_err(|source| Error::io(display_path, source))?;
        Ok(Self {
            file,
            identity,
            state: FileState::from_metadata(&metadata),
        })
    }

    pub(super) fn length(&self) -> u64 {
        self.state.length
    }

    pub(super) fn file_mut(&mut self) -> &mut fs::File {
        &mut self.file
    }

    pub(super) fn rewind(&mut self, display_path: &Path) -> Result<()> {
        std::io::Seek::seek(&mut self.file, std::io::SeekFrom::Start(0))
            .map(|_| ())
            .map_err(|source| Error::io(display_path, source))
    }

    pub(super) fn unchanged(&self, display_path: &Path) -> Result<bool> {
        self.file
            .metadata()
            .map(|metadata| FileState::from_metadata(&metadata) == self.state)
            .map_err(|source| Error::io(display_path, source))
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
            });
        }
        let parent_path = path.parent().unwrap_or_else(|| Path::new("."));
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
                let (parent, name) = open_parent(directory, relative, &self.path)?;
                let display = self.path.join(relative);
                let file = open_file_nofollow(&parent, &name, &display)?;
                OpenedFile::new(file, &display).map_err(SafePathError::Io)
            }
        }
    }

    pub(super) fn same_file(
        &self,
        relative: &Path,
        opened: &OpenedFile,
    ) -> std::result::Result<bool, SafePathError> {
        let current = match &self.inner {
            RootKind::File { parent, name, .. } => open_file_nofollow(parent, name, &self.path)?,
            RootKind::Directory(directory) => {
                let (parent, name) = open_parent(directory, relative, &self.path)?;
                open_file_nofollow(&parent, &name, &self.path.join(relative))?
            }
        };
        let identity = Handle::from_file(current)
            .map_err(|source| SafePathError::Io(Error::io(self.display_path(relative), source)))?;
        Ok(identity == opened.identity)
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

fn open_parent(
    root: &Dir,
    relative: &Path,
    root_path: &Path,
) -> std::result::Result<(Dir, OsString), SafePathError> {
    let mut components = relative.components().peekable();
    let mut directory = root
        .try_clone()
        .map_err(|source| SafePathError::Io(Error::io(root_path, source)))?;
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Err(SafePathError::Unsafe);
        };
        if components.peek().is_none() {
            return Ok((directory, name.to_os_string()));
        }
        let display = root_path.join(relative);
        directory = directory
            .open_dir_nofollow(name)
            .map_err(|source| classify_open(&directory, name, &display, source))?;
    }
    Err(SafePathError::Unsafe)
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
