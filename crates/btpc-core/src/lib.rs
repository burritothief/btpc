//! Core library for BTPC.
// Spec: SEC-PATH-001

mod error;
mod limits;
mod metadata;

pub mod bencode;
pub mod create;
pub mod edit;
pub mod magnet;
pub mod metainfo;
pub mod verify;

pub use error::{Error, ErrorCategory, Result};
pub use limits::{ParseLimits, ParseOptions};
pub use metadata::{DhtNode, OptionalMetadata};
pub use metainfo::{
    Canonicality, InfoHashV1, InfoHashV2, Metainfo, TorrentBytes, TorrentFile, TorrentMode,
    TorrentPath, UnknownField, ValidationReport, ValidationWarning,
};

/// Returns the BTPC core crate version.
#[must_use]
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_matches_workspace_package() {
        assert_eq!(super::version(), env!("CARGO_PKG_VERSION"));
    }
}
