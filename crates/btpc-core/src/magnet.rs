//! Deterministic magnet URI generation.

use std::fmt::Write as _;

use crate::Metainfo;

/// Optional magnet URI fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MagnetOptions {
    display_name: bool,
    trackers: bool,
    web_seeds: bool,
}

impl Default for MagnetOptions {
    fn default() -> Self {
        Self {
            display_name: true,
            trackers: true,
            web_seeds: true,
        }
    }
}

impl MagnetOptions {
    /// Starts an options builder.
    #[must_use]
    pub const fn builder() -> MagnetOptionsBuilder {
        MagnetOptionsBuilder(Self {
            display_name: true,
            trackers: true,
            web_seeds: true,
        })
    }
}

/// Builder for [`MagnetOptions`].
#[derive(Clone, Copy, Debug)]
pub struct MagnetOptionsBuilder(MagnetOptions);

impl MagnetOptionsBuilder {
    /// Includes or omits the display name.
    #[must_use]
    pub const fn display_name(mut self, include: bool) -> Self {
        self.0.display_name = include;
        self
    }

    /// Includes or omits tracker parameters.
    #[must_use]
    pub const fn trackers(mut self, include: bool) -> Self {
        self.0.trackers = include;
        self
    }

    /// Includes or omits web seed parameters.
    #[must_use]
    pub const fn web_seeds(mut self, include: bool) -> Self {
        self.0.web_seeds = include;
        self
    }

    /// Builds magnet options.
    #[must_use]
    pub const fn build(self) -> MagnetOptions {
        self.0
    }
}

/// Generates a deterministic magnet URI from validated metainfo.
#[must_use]
pub fn generate(metainfo: &Metainfo, options: &MagnetOptions) -> String {
    let mut parameters = Vec::new();
    if let Some(hash) = metainfo.info_hash_v1() {
        parameters.push(format!("xt=urn:btih:{}", hash.hex()));
    }
    if let Some(hash) = metainfo.info_hash_v2() {
        parameters.push(format!("xt=urn:btmh:1220{}", hash.hex()));
    }
    if options.display_name {
        parameters.push(format!("dn={}", percent_encode(metainfo.name())));
    }
    if options.trackers {
        for tracker in metainfo.trackers().iter().flatten() {
            parameters.push(format!("tr={}", percent_encode(tracker)));
        }
    }
    if options.web_seeds {
        for seed in metainfo.web_seeds() {
            parameters.push(format!("ws={}", percent_encode(seed)));
        }
    }
    format!("magnet:?{}", parameters.join("&"))
}

fn percent_encode(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut output, byte| {
        if byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'.' | b'_' | b'~') {
            output.push(char::from(*byte));
        } else {
            write!(output, "%{byte:02X}").expect("writing to String cannot fail");
        }
        output
    })
}
