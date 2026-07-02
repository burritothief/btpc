use crate::{Error, Result};

/// Stable identifier for the automatic piece-length selection table.
pub const PIECE_LENGTH_POLICY_ID: &str = "btpc-piece-v1";

/// Protocol constraints applied to an explicit piece length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PieceLengthMode {
    /// v1-only creation permits powers of two from 1 KiB.
    V1,
    /// v2 creation requires the BEP 52 16 KiB minimum.
    V2,
    /// Hybrid creation follows the v2 minimum.
    Hybrid,
}

const PIECE_LENGTH_BANDS: &[(u64, u64)] = &[
    (16 * 1024 * 1024, 16 * 1024),
    (32 * 1024 * 1024, 32 * 1024),
    (64 * 1024 * 1024, 64 * 1024),
    (128 * 1024 * 1024, 128 * 1024),
    (256 * 1024 * 1024, 256 * 1024),
    (512 * 1024 * 1024, 512 * 1024),
    (1024 * 1024 * 1024, 1024 * 1024),
    (2 * 1024 * 1024 * 1024, 2 * 1024 * 1024),
    (4 * 1024 * 1024 * 1024, 4 * 1024 * 1024),
    (8 * 1024 * 1024 * 1024, 8 * 1024 * 1024),
    (16 * 1024 * 1024 * 1024, 16 * 1024 * 1024),
];

/// Selects a stable automatic piece length for total payload bytes.
///
/// The table targets at most roughly 1024 pieces in each band while keeping a
/// 16 KiB minimum for v2 compatibility and a 16 MiB maximum for interoperability.
#[must_use]
pub fn automatic_piece_length(total_length: u64) -> u64 {
    PIECE_LENGTH_BANDS
        .iter()
        .find_map(|(maximum, piece_length)| (total_length <= *maximum).then_some(*piece_length))
        .unwrap_or(16 * 1024 * 1024)
}

/// Validates a caller-selected piece length.
///
/// # Errors
///
/// Returns a metainfo error when the value is not a power of two or is outside
/// the protocol-specific minimum and 16 MiB maximum.
pub fn validate_piece_length(piece_length: u64, mode: PieceLengthMode) -> Result<u64> {
    let minimum = match mode {
        PieceLengthMode::V1 => 1024,
        PieceLengthMode::V2 | PieceLengthMode::Hybrid => 16 * 1024,
    };
    if !piece_length.is_power_of_two() || !(minimum..=16 * 1024 * 1024).contains(&piece_length) {
        return Err(Error::metainfo_field(
            "piece length",
            format!(
                "must be a power of two between {minimum} and {} bytes",
                16 * 1024 * 1024
            ),
        ));
    }
    Ok(piece_length)
}
