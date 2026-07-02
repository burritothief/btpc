use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// Broad, stable categories used by adapters to map core failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorCategory {
    /// Filesystem or other I/O failure.
    Io,
    /// Invalid bencode syntax.
    BencodeSyntax,
    /// Parseable but non-canonical bencode.
    BencodeCanonical,
    /// Invalid metainfo field or cross-field relationship.
    Metainfo,
    /// A configured resource limit was exceeded.
    ResourceLimit,
    /// A requested feature or policy is unsupported.
    Unsupported,
    /// Payload data did not match metainfo.
    Verification,
    /// The operation was cancelled.
    Cancelled,
}

/// Errors returned by the BTPC core library.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An I/O operation failed for a path.
    Io {
        /// Path associated with the failed operation.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },
    /// Bencode syntax is invalid.
    BencodeSyntax {
        /// Byte offset where the violation was detected.
        offset: usize,
        /// Human-readable explanation.
        message: String,
    },
    /// Bencode is parseable but not canonical.
    BencodeCanonical {
        /// Byte offset where the violation was detected.
        offset: usize,
        /// Human-readable explanation.
        message: String,
    },
    /// A metainfo field is invalid.
    Metainfo {
        /// Dot-separated bencode field path.
        field: String,
        /// Human-readable explanation.
        message: String,
    },
    /// A parser or allocation resource limit was exceeded.
    ResourceLimit {
        /// Stable name of the configured limit.
        limit: &'static str,
        /// Observed or requested amount.
        actual: usize,
        /// Configured maximum amount.
        maximum: usize,
    },
    /// A requested feature or policy is unsupported.
    Unsupported {
        /// Human-readable explanation.
        message: String,
    },
    /// Payload verification found a mismatch.
    Verification {
        /// Optional payload path associated with the mismatch.
        path: Option<PathBuf>,
        /// Human-readable explanation.
        message: String,
    },
    /// The operation was cancelled.
    Cancelled,
}

impl Error {
    /// Creates an I/O error with path context.
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    /// Creates a bencode syntax error.
    pub fn bencode_syntax(offset: usize, message: impl Into<String>) -> Self {
        Self::BencodeSyntax {
            offset,
            message: message.into(),
        }
    }

    /// Creates a canonical bencode error.
    pub fn bencode_canonical(offset: usize, message: impl Into<String>) -> Self {
        Self::BencodeCanonical {
            offset,
            message: message.into(),
        }
    }

    /// Creates a metainfo field error.
    pub fn metainfo_field(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Metainfo {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Creates a resource-limit error.
    pub(crate) const fn resource_limit(limit: &'static str, actual: usize, maximum: usize) -> Self {
        Self::ResourceLimit {
            limit,
            actual,
            maximum,
        }
    }

    /// Creates an unsupported-feature error.
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported {
            message: message.into(),
        }
    }

    /// Creates a payload verification mismatch.
    pub fn verification_mismatch(path: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self::Verification {
            path,
            message: message.into(),
        }
    }

    /// Creates a cancellation error.
    #[must_use]
    pub const fn cancelled() -> Self {
        Self::Cancelled
    }

    /// Returns the stable category for this error.
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::Io { .. } => ErrorCategory::Io,
            Self::BencodeSyntax { .. } => ErrorCategory::BencodeSyntax,
            Self::BencodeCanonical { .. } => ErrorCategory::BencodeCanonical,
            Self::Metainfo { .. } => ErrorCategory::Metainfo,
            Self::ResourceLimit { .. } => ErrorCategory::ResourceLimit,
            Self::Unsupported { .. } => ErrorCategory::Unsupported,
            Self::Verification { .. } => ErrorCategory::Verification,
            Self::Cancelled => ErrorCategory::Cancelled,
        }
    }

    /// Returns the associated byte offset, when present.
    #[must_use]
    pub const fn offset(&self) -> Option<usize> {
        match self {
            Self::BencodeSyntax { offset, .. } | Self::BencodeCanonical { offset, .. } => {
                Some(*offset)
            }
            _ => None,
        }
    }

    /// Returns the associated metainfo field path, when present.
    #[must_use]
    pub fn field(&self) -> Option<&str> {
        match self {
            Self::Metainfo { field, .. } => Some(field),
            _ => None,
        }
    }

    /// Returns the associated filesystem path, when present.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Io { path, .. } => Some(path),
            Self::Verification { path, .. } => path.as_deref(),
            _ => None,
        }
    }

    /// Returns the stable resource-limit name, when present.
    #[must_use]
    pub const fn limit(&self) -> Option<&'static str> {
        match self {
            Self::ResourceLimit { limit, .. } => Some(limit),
            _ => None,
        }
    }

    /// Returns the observed and maximum resource amounts, when present.
    #[must_use]
    pub const fn actual_and_maximum(&self) -> Option<(usize, usize)> {
        match self {
            Self::ResourceLimit {
                actual, maximum, ..
            } => Some((*actual, *maximum)),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "I/O error for {}: {source}", path.display())
            }
            Self::BencodeSyntax { offset, message } => {
                write!(
                    formatter,
                    "bencode syntax error at byte {offset}: {message}"
                )
            }
            Self::BencodeCanonical { offset, message } => write!(
                formatter,
                "non-canonical bencode at byte {offset}: {message}"
            ),
            Self::Metainfo { field, message } => {
                write!(formatter, "invalid metainfo field {field}: {message}")
            }
            Self::ResourceLimit {
                limit,
                actual,
                maximum,
            } => write!(
                formatter,
                "resource limit exceeded for {limit}: {actual} > {maximum}"
            ),
            Self::Unsupported { message } => write!(formatter, "unsupported: {message}"),
            Self::Verification { path, message } => {
                if let Some(path) = path {
                    write!(
                        formatter,
                        "verification mismatch for {}: {message}",
                        path.display()
                    )
                } else {
                    write!(formatter, "verification mismatch: {message}")
                }
            }
            Self::Cancelled => formatter.write_str("operation cancelled"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Convenient result alias for BTPC core operations.
pub type Result<T> = std::result::Result<T, Error>;
