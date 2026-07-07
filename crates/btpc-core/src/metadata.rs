#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TrackerUrl(Vec<u8>);

impl TrackerUrl {
    pub(crate) fn new(value: Vec<u8>) -> Self {
        Self(value)
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TrackerTier(Vec<TrackerUrl>);

impl TrackerTier {
    pub(crate) fn new(values: Vec<Vec<u8>>) -> Self {
        Self(values.into_iter().map(TrackerUrl::new).collect())
    }

    pub(crate) fn urls(&self) -> &[TrackerUrl] {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WebSeed(Vec<u8>);

impl WebSeed {
    pub(crate) fn new(value: Vec<u8>) -> Self {
        Self(value)
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NodeHost(Vec<u8>);

impl NodeHost {
    pub(crate) fn new(value: Vec<u8>) -> Self {
        Self(value)
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MetadataText(Vec<u8>);

impl MetadataText {
    pub(crate) fn new(value: Vec<u8>) -> Self {
        Self(value)
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{MetadataText, NodeHost, TrackerTier, WebSeed};

    #[test]
    fn field_types_preserve_non_utf8_bytes_without_interchange() {
        let tier = TrackerTier::new(vec![vec![0xff, b't']]);
        let seed = WebSeed::new(vec![0xfe, b'w']);
        let host = NodeHost::new(vec![0xfd, b'n']);
        let text = MetadataText::new(vec![0xfc, b'x']);
        assert_eq!(tier.urls()[0].as_bytes(), &[0xff, b't']);
        assert_eq!(seed.into_bytes(), vec![0xfe, b'w']);
        assert_eq!(host.as_bytes(), &[0xfd, b'n']);
        assert_eq!(text.as_bytes(), &[0xfc, b'x']);
    }
}
use crate::{Error, Result};

/// One validated DHT bootstrap node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DhtNode {
    host: Vec<u8>,
    port: u16,
}

impl DhtNode {
    pub(crate) fn new(host: Vec<u8>, port: u16) -> Self {
        Self { host, port }
    }

    /// Returns the raw host bytes.
    #[must_use]
    pub fn host(&self) -> &[u8] {
        &self.host
    }

    /// Returns the non-zero port.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }
}

/// Validated optional metainfo fields shared by parsing and inspection surfaces.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OptionalMetadata {
    trackers: Vec<Vec<Vec<u8>>>,
    web_seeds: Vec<Vec<u8>>,
    nodes: Vec<DhtNode>,
    source: Option<Vec<u8>>,
    comment: Option<Vec<u8>>,
    created_by: Option<Vec<u8>>,
    creation_date: Option<i64>,
}

impl OptionalMetadata {
    pub(crate) fn new(
        trackers: Vec<Vec<Vec<u8>>>,
        web_seeds: Vec<Vec<u8>>,
        nodes: Vec<DhtNode>,
        source: Option<Vec<u8>>,
        comment: Option<Vec<u8>>,
        created_by: Option<Vec<u8>>,
        creation_date: Option<i64>,
    ) -> Self {
        Self {
            trackers,
            web_seeds,
            nodes,
            source,
            comment,
            created_by,
            creation_date,
        }
    }

    /// Returns tracker tiers as raw byte strings.
    #[must_use]
    pub fn trackers(&self) -> &[Vec<Vec<u8>>] {
        &self.trackers
    }

    /// Returns web seed URLs as raw byte strings.
    #[must_use]
    pub fn web_seeds(&self) -> &[Vec<u8>] {
        &self.web_seeds
    }

    /// Returns validated DHT bootstrap nodes.
    #[must_use]
    pub fn nodes(&self) -> &[DhtNode] {
        &self.nodes
    }

    /// Returns raw source bytes.
    #[must_use]
    pub fn source(&self) -> Option<&[u8]> {
        self.source.as_deref()
    }

    /// Returns raw comment bytes.
    #[must_use]
    pub fn comment(&self) -> Option<&[u8]> {
        self.comment.as_deref()
    }

    /// Returns raw creator bytes.
    #[must_use]
    pub fn created_by(&self) -> Option<&[u8]> {
        self.created_by.as_deref()
    }

    /// Returns the non-negative Unix timestamp.
    #[must_use]
    pub const fn creation_date(&self) -> Option<i64> {
        self.creation_date
    }
}

pub(crate) fn validate_tracker_tiers(trackers: &[Vec<Vec<u8>>]) -> Result<()> {
    if trackers.iter().any(Vec::is_empty) {
        return Err(Error::metainfo_field(
            "announce-list",
            "tracker tier is empty",
        ));
    }
    if trackers.iter().flatten().any(Vec::is_empty) {
        return Err(Error::metainfo_field("announce", "tracker URL is empty"));
    }
    Ok(())
}

pub(crate) fn validate_web_seeds(web_seeds: &[Vec<u8>]) -> Result<()> {
    if web_seeds.iter().any(Vec::is_empty) {
        return Err(Error::metainfo_field("url-list", "web seed URL is empty"));
    }
    Ok(())
}

pub(crate) fn validate_nodes(nodes: &[(Vec<u8>, u16)]) -> Result<()> {
    if nodes.iter().any(|(host, _)| host.is_empty()) {
        return Err(Error::metainfo_field("nodes", "node host is empty"));
    }
    if nodes.iter().any(|(_, port)| *port == 0) {
        return Err(Error::metainfo_field(
            "nodes",
            "node port must be 1..=65535",
        ));
    }
    Ok(())
}

pub(crate) fn validate_creation_date(creation_date: Option<i64>) -> Result<()> {
    if creation_date.is_some_and(|value| value < 0) {
        return Err(Error::metainfo_field(
            "creation date",
            "must be a non-negative Unix timestamp",
        ));
    }
    Ok(())
}
