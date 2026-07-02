use crate::{Error, Result};

use super::validate_path_component;

/// Owned torrent byte string whose identity is the exact raw bytes.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TorrentBytes(Vec<u8>);

impl TorrentBytes {
    /// Creates a raw byte value without text decoding.
    #[must_use]
    pub fn new(raw: impl Into<Vec<u8>>) -> Self {
        Self(raw.into())
    }

    /// Returns the exact raw bytes.
    #[must_use]
    pub fn raw(&self) -> &[u8] {
        &self.0
    }

    /// Returns a UTF-8 view when decoding is lossless.
    #[must_use]
    pub fn utf8(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }
}

impl AsRef<[u8]> for TorrentBytes {
    fn as_ref(&self) -> &[u8] {
        self.raw()
    }
}

/// Owned torrent path preserving raw component identity and ordering.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TorrentPath(Vec<TorrentBytes>);

impl TorrentPath {
    /// Creates a path after validating every raw component.
    ///
    /// # Errors
    ///
    /// Returns a metainfo error when the path is empty or a component is unsafe.
    pub fn new(components: impl IntoIterator<Item = TorrentBytes>) -> Result<Self> {
        let components = components.into_iter().collect::<Vec<_>>();
        for component in &components {
            validate_path_component(component.raw(), "torrent path")?;
        }
        if components.is_empty() {
            return Err(Error::metainfo_field("torrent path", "path is empty"));
        }
        Ok(Self(components))
    }

    pub(crate) fn from_raw(path: &[Vec<u8>]) -> Self {
        Self(path.iter().cloned().map(TorrentBytes::new).collect())
    }

    /// Returns raw path components.
    #[must_use]
    pub fn components(&self) -> &[TorrentBytes] {
        &self.0
    }

    /// Returns UTF-8 components only when every component decodes losslessly.
    #[must_use]
    pub fn utf8_components(&self) -> Option<Vec<&str>> {
        self.0.iter().map(TorrentBytes::utf8).collect()
    }

    /// Converts to a platform path only when the conversion is lossless.
    ///
    /// # Errors
    ///
    /// Returns unsupported on platforms that cannot represent a non-UTF-8
    /// component without replacement.
    pub fn to_path_buf(&self) -> Result<std::path::PathBuf> {
        self.0
            .iter()
            .try_fold(std::path::PathBuf::new(), |mut path, component| {
                path.push(component_to_os_string(component.raw())?);
                Ok(path)
            })
    }
}

#[cfg(unix)]
#[allow(clippy::unnecessary_wraps)]
fn component_to_os_string(bytes: &[u8]) -> Result<std::ffi::OsString> {
    use std::os::unix::ffi::OsStringExt as _;
    Ok(std::ffi::OsString::from_vec(bytes.to_vec()))
}

#[cfg(not(unix))]
fn component_to_os_string(bytes: &[u8]) -> Result<std::ffi::OsString> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        Error::unsupported("non-UTF-8 torrent paths cannot be represented on this platform")
    })?;
    Ok(std::ffi::OsString::from(text))
}
