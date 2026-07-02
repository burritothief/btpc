use crate::{Error, Result};

/// Cooperative cancellation handle shared with long-running core operations.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    /// Creates an active token.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests cancellation.
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
    }

    /// Returns whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Acquire)
    }

    pub(crate) fn check(&self) -> Result<()> {
        if self.is_cancelled() {
            Err(Error::cancelled())
        } else {
            Ok(())
        }
    }
}

/// Monotonic progress snapshot for sequential hashing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HashProgress {
    pub(super) bytes_hashed: u64,
    pub(super) total_bytes: u64,
    pub(super) pieces_hashed: u64,
}

impl HashProgress {
    /// Creates a progress snapshot for shared core operations.
    #[must_use]
    pub const fn new(bytes_hashed: u64, total_bytes: u64, pieces_hashed: u64) -> Self {
        Self {
            bytes_hashed,
            total_bytes,
            pieces_hashed,
        }
    }

    /// Returns bytes consumed from payload files.
    #[must_use]
    pub const fn bytes_hashed(self) -> u64 {
        self.bytes_hashed
    }

    /// Returns the total expected payload bytes.
    #[must_use]
    pub const fn total_bytes(self) -> u64 {
        self.total_bytes
    }

    /// Returns completed piece hashes.
    #[must_use]
    pub const fn pieces_hashed(self) -> u64 {
        self.pieces_hashed
    }
}

/// Presentation-free observer for hashing progress.
pub trait ProgressSink: Sync {
    /// Receives a monotonic progress snapshot.
    fn on_progress(&self, progress: HashProgress);
}

/// Zero-overhead progress sink for callers that do not need updates.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoProgress;

impl ProgressSink for NoProgress {
    fn on_progress(&self, _progress: HashProgress) {}
}
