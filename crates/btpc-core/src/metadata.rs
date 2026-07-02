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
