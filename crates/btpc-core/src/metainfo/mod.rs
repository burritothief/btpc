//! Borrowed top-level metainfo views.

mod path_types;

pub use path_types::{TorrentBytes, TorrentPath};

use sha1::{Digest as _, Sha1};
use sha2::Sha256;

use crate::bencode::{ByteString, Span, Value, ValueKind, parse_with_budget};
use crate::limits::AllocationBudget;
use crate::metadata::{
    DhtNode, OptionalMetadata, validate_creation_date, validate_nodes, validate_tracker_tiers,
    validate_web_seeds,
};
use crate::{Error, ErrorCategory, ParseOptions, Result};

const INFO: &[u8] = b"info";
const KNOWN_TOP_LEVEL_FIELDS: &[&[u8]] = &[
    b"announce",
    b"announce-list",
    b"url-list",
    b"nodes",
    b"comment",
    b"created by",
    b"creation date",
    INFO,
];

/// Borrowed, syntax-level view of a `BitTorrent` metainfo dictionary.
#[derive(Debug)]
pub struct RawMetainfo<'a> {
    original: &'a [u8],
    root: Value<'a>,
    info_index: usize,
}

impl<'a> RawMetainfo<'a> {
    /// Parses a top-level metainfo dictionary while preserving original bytes.
    ///
    /// # Errors
    ///
    /// Returns a syntax/resource error from bencode parsing, or a metainfo error
    /// when the root is not a dictionary or `info` is missing, duplicated, or not
    /// a dictionary.
    pub fn from_bytes(original: &'a [u8]) -> Result<Self> {
        Self::from_bytes_with_options(original, ParseOptions::default())
    }

    /// Parses a top-level metainfo dictionary using caller-provided limits.
    ///
    /// # Errors
    ///
    /// Returns a syntax/resource error from bencode parsing, or a metainfo error
    /// when the root is not a dictionary or `info` is invalid.
    pub fn from_bytes_with_options(original: &'a [u8], options: ParseOptions) -> Result<Self> {
        let mut budget = AllocationBudget::new(options.limits());
        let root = parse_with_budget(original, options.limits(), &mut budget)?;
        Self::from_parsed(original, root)
    }

    fn from_parsed(original: &'a [u8], root: Value<'a>) -> Result<Self> {
        let ValueKind::Dictionary(entries) = root.kind() else {
            return Err(Error::metainfo_field(
                "<root>",
                "top-level metainfo must be a dictionary",
            ));
        };

        let mut info_indices = entries
            .iter()
            .enumerate()
            .filter_map(|(index, (key, _))| (key.bytes() == INFO).then_some(index));
        let Some(info_index) = info_indices.next() else {
            return Err(Error::metainfo_field("info", "missing required dictionary"));
        };
        if info_indices.next().is_some() {
            return Err(Error::metainfo_field("info", "duplicate top-level key"));
        }
        if !matches!(entries[info_index].1.kind(), ValueKind::Dictionary(_)) {
            return Err(Error::metainfo_field("info", "value must be a dictionary"));
        }

        Ok(Self {
            original,
            root,
            info_index,
        })
    }

    /// Reports whether the original source uses canonical bencode.
    #[must_use]
    pub fn canonicality(&self) -> Canonicality {
        canonicality(self.original, &self.root)
    }

    /// Returns the complete original metainfo bytes.
    #[must_use]
    pub const fn original_bytes(&self) -> &'a [u8] {
        self.original
    }

    /// Returns the exact original encoded `info` bytes.
    #[must_use]
    pub fn info_bytes(&self) -> &'a [u8] {
        let span = self.info_span();
        &self.original[span.range()]
    }

    /// Returns the exact source span of the encoded `info` dictionary.
    #[must_use]
    pub fn info_span(&self) -> Span {
        self.info_value().span()
    }

    /// Returns the parsed `info` dictionary value.
    #[must_use]
    pub fn info_value(&self) -> &Value<'a> {
        &self.entries()[self.info_index].1
    }

    pub(crate) const fn root(&self) -> &Value<'a> {
        &self.root
    }

    /// Returns the first `announce` field, when present.
    #[must_use]
    pub fn announce(&self) -> Option<&Value<'a>> {
        self.field(b"announce")
    }

    /// Returns the first `announce-list` field, when present.
    #[must_use]
    pub fn announce_list(&self) -> Option<&Value<'a>> {
        self.field(b"announce-list")
    }

    /// Returns the first `url-list` field, when present.
    #[must_use]
    pub fn url_list(&self) -> Option<&Value<'a>> {
        self.field(b"url-list")
    }

    /// Returns the first `nodes` field, when present.
    #[must_use]
    pub fn nodes(&self) -> Option<&Value<'a>> {
        self.field(b"nodes")
    }

    /// Returns the first `comment` field, when present.
    #[must_use]
    pub fn comment(&self) -> Option<&Value<'a>> {
        self.field(b"comment")
    }

    /// Returns the first `created by` field, when present.
    #[must_use]
    pub fn created_by(&self) -> Option<&Value<'a>> {
        self.field(b"created by")
    }

    /// Returns the first `creation date` field, when present.
    #[must_use]
    pub fn creation_date(&self) -> Option<&Value<'a>> {
        self.field(b"creation date")
    }

    /// Returns top-level entries whose keys are not recognized by this view.
    #[must_use]
    pub fn unknown_fields(&self) -> Vec<(ByteString<'a>, &Value<'a>)> {
        self.entries()
            .iter()
            .filter(|(key, _)| !KNOWN_TOP_LEVEL_FIELDS.contains(&key.bytes()))
            .map(|(key, value)| (*key, value))
            .collect()
    }

    /// Computes SHA-1 over the exact original `info` bytes.
    #[must_use]
    pub fn info_hash_sha1(&self) -> [u8; 20] {
        Sha1::digest(self.info_bytes()).into()
    }

    /// Computes SHA-256 over the exact original `info` bytes.
    #[must_use]
    pub fn info_hash_sha256(&self) -> [u8; 32] {
        Sha256::digest(self.info_bytes()).into()
    }

    pub(crate) fn field(&self, name: &[u8]) -> Option<&Value<'a>> {
        self.entries()
            .iter()
            .find_map(|(key, value)| (key.bytes() == name).then_some(value))
    }

    pub(crate) fn entries(&self) -> &[(ByteString<'a>, Value<'a>)] {
        let ValueKind::Dictionary(entries) = self.root.kind() else {
            unreachable!("constructor requires a top-level dictionary")
        };
        entries
    }
}

/// Protocol representation identified for validated metainfo.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TorrentMode {
    /// BEP 3 / v1 metainfo only.
    V1,
    /// BEP 52 / v2 metainfo only.
    V2,
    /// Both compatible v1 and v2 representations.
    Hybrid,
}

/// A validated v1 payload file entry.
#[derive(Debug)]
pub struct V1File<'a> {
    length: u64,
    path: Vec<&'a [u8]>,
    attributes: &'a [u8],
}

impl<'a> V1File<'a> {
    /// Returns the file length in bytes.
    #[must_use]
    pub const fn length(&self) -> u64 {
        self.length
    }

    /// Returns raw torrent path components.
    #[must_use]
    pub fn path_components(&self) -> &[&'a [u8]] {
        &self.path
    }

    /// Returns UTF-8 path components when all raw components are valid UTF-8.
    #[must_use]
    pub fn path_utf8(&self) -> Option<Vec<&'a str>> {
        self.path
            .iter()
            .map(|component| std::str::from_utf8(component).ok())
            .collect()
    }

    /// Returns BEP 47 file attribute bytes.
    #[must_use]
    pub const fn attributes(&self) -> &'a [u8] {
        self.attributes
    }

    /// Returns whether this entry is a validated hybrid alignment padding file.
    #[must_use]
    pub fn is_padding(&self) -> bool {
        self.attributes.contains(&b'p') && is_padding_path(&self.path, self.length)
    }
}

/// Validated typed view of v1 metainfo.
#[derive(Debug)]
pub struct V1Metainfo<'a> {
    name: &'a [u8],
    piece_length: u64,
    pieces: &'a [u8],
    files: Vec<V1File<'a>>,
    total_length: u64,
    single_file: bool,
    warnings: Vec<String>,
}

impl<'a> V1Metainfo<'a> {
    /// Validates and constructs a typed v1 view from raw metainfo.
    ///
    /// # Errors
    ///
    /// Returns a metainfo error for missing or incorrectly typed fields, invalid
    /// paths or lengths, arithmetic overflow, or an inconsistent piece count.
    pub fn from_raw(raw: &'a RawMetainfo<'a>) -> Result<Self> {
        let info = dictionary_entries(raw.info_value(), "info")?;
        let name = required_bytes(info, b"name", "info.name")?;
        validate_path_component(name, "info.name")?;
        let piece_length = positive_integer(info, b"piece length", "info.piece length")?;
        let pieces = required_bytes(info, b"pieces", "info.pieces")?;
        if pieces.len() % 20 != 0 {
            return Err(Error::metainfo_field(
                "info.pieces",
                "length must be divisible by 20",
            ));
        }

        let (files, total_length, single_file) = parse_v1_files(info, name)?;

        let expected_piece_count = if total_length == 0 {
            0
        } else {
            1 + (total_length - 1) / piece_length
        };
        let actual_piece_count = u64::try_from(pieces.len() / 20).map_err(|_| {
            Error::metainfo_field("info.pieces", "piece count cannot be represented")
        })?;
        if actual_piece_count != expected_piece_count {
            return Err(Error::metainfo_field(
                "info.pieces",
                format!(
                    "piece count {actual_piece_count} does not match expected {expected_piece_count}"
                ),
            ));
        }

        Ok(Self {
            name,
            piece_length,
            pieces,
            files,
            total_length,
            single_file,
            warnings: Vec::new(),
        })
    }

    /// Returns the validated torrent mode.
    #[must_use]
    pub const fn mode(&self) -> TorrentMode {
        TorrentMode::V1
    }

    /// Returns the raw v1 name bytes.
    #[must_use]
    pub const fn name(&self) -> &'a [u8] {
        self.name
    }

    /// Returns the name as UTF-8 when valid.
    #[must_use]
    pub fn name_utf8(&self) -> Option<&'a str> {
        std::str::from_utf8(self.name).ok()
    }

    /// Returns the piece length in bytes.
    #[must_use]
    pub const fn piece_length(&self) -> u64 {
        self.piece_length
    }

    /// Returns the concatenated 20-byte SHA-1 piece hashes.
    #[must_use]
    pub const fn pieces(&self) -> &'a [u8] {
        self.pieces
    }

    /// Returns validated payload files in torrent order.
    #[must_use]
    pub fn files(&self) -> &[V1File<'a>] {
        &self.files
    }

    /// Returns total payload bytes.
    #[must_use]
    pub const fn total_length(&self) -> u64 {
        self.total_length
    }

    /// Returns whether this uses the v1 single-file representation.
    #[must_use]
    pub const fn is_single_file(&self) -> bool {
        self.single_file
    }

    /// Returns non-fatal compatibility warnings.
    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}

fn parse_v1_files<'a>(
    info: &'a [(ByteString<'a>, Value<'a>)],
    name: &'a [u8],
) -> Result<(Vec<V1File<'a>>, u64, bool)> {
    let length_value = unique_field(info, b"length", "info.length")?;
    let files_value = unique_field(info, b"files", "info.files")?;
    match (length_value, files_value) {
        (Some(length), None) => {
            let length = non_negative_value(length, "info.length")?;
            Ok((
                vec![V1File {
                    length,
                    path: vec![name],
                    attributes: unique_field(info, b"attr", "info.attr")?
                        .map_or(Ok(&[][..]), |value| value_bytes(value, "info.attr"))?,
                }],
                length,
                true,
            ))
        }
        (None, Some(files)) => parse_v1_file_list(files),
        (Some(_), Some(_)) | (None, None) => Err(Error::metainfo_field(
            "info",
            "exactly one of length or files is required",
        )),
    }
}

fn parse_v1_file_list<'a>(files: &'a Value<'a>) -> Result<(Vec<V1File<'a>>, u64, bool)> {
    let values = list_values(files, "info.files")?;
    if values.is_empty() {
        return Err(Error::metainfo_field(
            "info.files",
            "multi-file torrents require at least one file",
        ));
    }
    let mut parsed = Vec::with_capacity(values.len());
    let mut total = 0_u64;
    for value in values {
        let entry = dictionary_entries(value, "info.files")?;
        let length = non_negative_value(
            required_field(entry, b"length", "info.files.length")?,
            "info.files.length",
        )?;
        total = total.checked_add(length).ok_or_else(|| {
            Error::metainfo_field("info.files.length", "total payload length overflowed")
        })?;
        let components = list_values(
            required_field(entry, b"path", "info.files.path")?,
            "info.files.path",
        )?;
        if components.is_empty() {
            return Err(Error::metainfo_field(
                "info.files.path",
                "path must contain at least one component",
            ));
        }
        let mut path = Vec::with_capacity(components.len());
        for component in components {
            let component = value_bytes(component, "info.files.path")?;
            validate_path_component(component, "info.files.path")?;
            path.push(component);
        }
        let attributes = unique_field(entry, b"attr", "info.files.attr")?
            .map_or(Ok(&[][..]), |value| value_bytes(value, "info.files.attr"))?;
        parsed.push(V1File {
            length,
            path,
            attributes,
        });
    }
    validate_v1_path_graph(&parsed)?;
    Ok((parsed, total, false))
}

fn validate_v1_path_graph(files: &[V1File<'_>]) -> Result<()> {
    validate_torrent_path_graph(
        files
            .iter()
            .map(|file| file.path_components().to_vec())
            .collect(),
        "info.files.path",
    )
}

pub(crate) fn validate_torrent_path_graph(
    mut paths: Vec<Vec<&[u8]>>,
    field: &'static str,
) -> Result<()> {
    paths.sort_unstable();
    for pair in paths.windows(2) {
        if pair[0] == pair[1] || pair[1].starts_with(&pair[0]) {
            return Err(Error::metainfo_field(
                field,
                "duplicate or prefix-colliding file path",
            ));
        }
        #[cfg(windows)]
        if windows_path_key(&pair[0]) == windows_path_key(&pair[1]) {
            return Err(Error::metainfo_field(
                field,
                "paths collide after Windows filesystem mapping",
            ));
        }
    }
    Ok(())
}

#[cfg(windows)]
fn windows_path_key(path: &[&[u8]]) -> Vec<Vec<u8>> {
    path.iter()
        .map(|component| component.iter().map(u8::to_ascii_lowercase).collect())
        .collect()
}

fn is_padding_path(path: &[&[u8]], length: u64) -> bool {
    let [directory, name] = path else {
        return false;
    };
    if *directory != b".pad" {
        return false;
    }
    let Some((_, encoded_length)) = parse_padding_name(name) else {
        return false;
    };
    encoded_length == length
}

fn parse_padding_name(value: &[u8]) -> Option<(Option<u64>, u64)> {
    if let Some(separator) = value.iter().position(|byte| *byte == b'-') {
        let (left, right_with_separator) = value.split_at(separator);
        let right = right_with_separator.get(1..)?;
        if left.is_empty() || right.is_empty() {
            return None;
        }
        let offset = std::str::from_utf8(left).ok()?.parse().ok()?;
        let length = std::str::from_utf8(right).ok()?.parse().ok()?;
        return Some((Some(offset), length));
    }
    let length = std::str::from_utf8(value).ok()?.parse().ok()?;
    Some((None, length))
}

fn dictionary_entries<'a>(
    value: &'a Value<'a>,
    field: &'static str,
) -> Result<&'a [(ByteString<'a>, Value<'a>)]> {
    match value.kind() {
        ValueKind::Dictionary(entries) => Ok(entries),
        _ => Err(Error::metainfo_field(field, "must be a dictionary")),
    }
}

fn list_values<'a>(value: &'a Value<'a>, field: &'static str) -> Result<&'a [Value<'a>]> {
    match value.kind() {
        ValueKind::List(values) => Ok(values),
        _ => Err(Error::metainfo_field(field, "must be a list")),
    }
}

fn value_bytes<'a>(value: &'a Value<'a>, field: &'static str) -> Result<&'a [u8]> {
    value
        .as_bytes()
        .ok_or_else(|| Error::metainfo_field(field, "must be a byte string"))
}

fn unique_field<'a>(
    entries: &'a [(ByteString<'a>, Value<'a>)],
    key: &[u8],
    field: &'static str,
) -> Result<Option<&'a Value<'a>>> {
    let mut matching = entries
        .iter()
        .filter_map(|(candidate, value)| (candidate.bytes() == key).then_some(value));
    let value = matching.next();
    if matching.next().is_some() {
        return Err(Error::metainfo_field(field, "duplicate field"));
    }
    Ok(value)
}

fn required_field<'a>(
    entries: &'a [(ByteString<'a>, Value<'a>)],
    key: &[u8],
    field: &'static str,
) -> Result<&'a Value<'a>> {
    unique_field(entries, key, field)?
        .ok_or_else(|| Error::metainfo_field(field, "missing required field"))
}

fn required_bytes<'a>(
    entries: &'a [(ByteString<'a>, Value<'a>)],
    key: &[u8],
    field: &'static str,
) -> Result<&'a [u8]> {
    value_bytes(required_field(entries, key, field)?, field)
}

fn positive_integer(
    entries: &[(ByteString<'_>, Value<'_>)],
    key: &[u8],
    field: &'static str,
) -> Result<u64> {
    let value = required_field(entries, key, field)?;
    let integer = value
        .integer()
        .ok_or_else(|| Error::metainfo_field(field, "must be an integer"))?;
    let Some(integer) = integer.to_u64() else {
        return Err(Error::metainfo_field(field, "is out of range"));
    };
    if integer == 0 {
        return Err(Error::metainfo_field(field, "must be positive"));
    }
    Ok(integer)
}

fn non_negative_value(value: &Value<'_>, field: &'static str) -> Result<u64> {
    let integer = value
        .integer()
        .ok_or_else(|| Error::metainfo_field(field, "must be an integer"))?;
    integer
        .to_u64()
        .ok_or_else(|| Error::metainfo_field(field, "must be non-negative and fit in u64"))
}

fn validate_path_component(component: &[u8], field: &'static str) -> Result<()> {
    if component.is_empty()
        || component == b"."
        || component == b".."
        || component.contains(&b'/')
        || component.contains(&b'\\')
        || component.contains(&0)
    {
        return Err(Error::metainfo_field(
            field,
            "contains an unsafe path component",
        ));
    }
    Ok(())
}

/// A validated v2 payload file entry.
#[derive(Debug)]
pub struct V2File<'a> {
    length: u64,
    path: Vec<&'a [u8]>,
    pieces_root: Option<&'a [u8; 32]>,
    attributes: &'a [u8],
    piece_layer: Option<&'a [u8]>,
    properties: &'a Value<'a>,
}

impl<'a> V2File<'a> {
    /// Returns the file length in bytes.
    #[must_use]
    pub const fn length(&self) -> u64 {
        self.length
    }

    /// Returns raw torrent path components.
    #[must_use]
    pub fn path_components(&self) -> &[&'a [u8]] {
        &self.path
    }

    /// Returns UTF-8 path components when all components are valid UTF-8.
    #[must_use]
    pub fn path_utf8(&self) -> Option<Vec<&'a str>> {
        self.path
            .iter()
            .map(|component| std::str::from_utf8(component).ok())
            .collect()
    }

    /// Returns the 32-byte pieces root for a non-empty file.
    #[must_use]
    pub const fn pieces_root(&self) -> Option<&'a [u8; 32]> {
        self.pieces_root
    }

    /// Returns file attribute bytes, including unknown attributes.
    #[must_use]
    pub const fn attributes(&self) -> &'a [u8] {
        self.attributes
    }

    /// Returns the validated piece-layer bytes for files larger than one piece.
    #[must_use]
    pub const fn piece_layer(&self) -> Option<&'a [u8]> {
        self.piece_layer
    }

    /// Returns the raw file properties dictionary for extension access.
    #[must_use]
    pub const fn properties(&self) -> &'a Value<'a> {
        self.properties
    }
}

/// Validated typed view of v2 or hybrid metainfo.
#[derive(Debug)]
pub struct V2Metainfo<'a> {
    name: &'a [u8],
    piece_length: u64,
    files: Vec<V2File<'a>>,
    total_length: u64,
    piece_count: u64,
    mode: TorrentMode,
}

impl<'a> V2Metainfo<'a> {
    /// Validates and constructs a typed v2 or hybrid view.
    ///
    /// # Errors
    ///
    /// Returns a metainfo error for malformed trees, invalid roots or layers,
    /// unsupported meta versions, or inconsistent hybrid representations.
    pub fn from_raw(raw: &'a RawMetainfo<'a>) -> Result<Self> {
        let info = dictionary_entries(raw.info_value(), "info")?;
        let meta_version = required_field(info, b"meta version", "info.meta version")?
            .integer()
            .ok_or_else(|| Error::metainfo_field("info.meta version", "must be an integer"))?;
        let meta_version = meta_version.to_i64().ok_or_else(|| {
            Error::metainfo_field("info.meta version", "is outside the supported range")
        })?;
        if meta_version != 2 {
            return Err(Error::metainfo_field(
                "info.meta version",
                "unsupported meta version",
            ));
        }
        let name = required_bytes(info, b"name", "info.name")?;
        validate_path_component(name, "info.name")?;
        let piece_length = positive_integer(info, b"piece length", "info.piece length")?;
        if piece_length < 16_384 || !piece_length.is_power_of_two() {
            return Err(Error::metainfo_field(
                "info.piece length",
                "must be a power of two and at least 16384",
            ));
        }
        let tree = required_field(info, b"file tree", "info.file tree")?;
        let root_entries = dictionary_entries(tree, "info.file tree")?;
        if root_entries.iter().any(|(key, _)| key.bytes().is_empty()) {
            return Err(Error::metainfo_field(
                "info.file tree",
                "root must not be a file",
            ));
        }
        let mut pending = Vec::new();
        let mut path = Vec::new();
        walk_file_tree(root_entries, &mut path, &mut pending)?;
        if pending.is_empty() {
            return Err(Error::metainfo_field(
                "info.file tree",
                "must contain at least one file",
            ));
        }
        let layer_entries = raw
            .field(b"piece layers")
            .map(|layers| dictionary_entries(layers, "piece layers"))
            .transpose()?
            .unwrap_or_default();
        let files = validate_piece_layers(pending, layer_entries, piece_length)?;
        let total_length =
            checked_total_length(files.iter().map(V2File::length), "info.file tree.length")?;
        let piece_count = checked_piece_count(
            files.iter().map(V2File::length),
            piece_length,
            "info.file tree.piece count",
        )?;
        let has_v1 = unique_field(info, b"pieces", "info.pieces")?.is_some()
            || unique_field(info, b"length", "info.length")?.is_some()
            || unique_field(info, b"files", "info.files")?.is_some();
        let mode = if has_v1 {
            let v1 = V1Metainfo::from_raw(raw)?;
            validate_hybrid(&v1, &files, piece_length)?;
            TorrentMode::Hybrid
        } else {
            TorrentMode::V2
        };
        Ok(Self {
            name,
            piece_length,
            files,
            total_length,
            piece_count,
            mode,
        })
    }

    /// Returns the validated torrent mode.
    #[must_use]
    pub const fn mode(&self) -> TorrentMode {
        self.mode
    }

    /// Returns the raw display name.
    #[must_use]
    pub const fn name(&self) -> &'a [u8] {
        self.name
    }

    /// Returns the piece length in bytes.
    #[must_use]
    pub const fn piece_length(&self) -> u64 {
        self.piece_length
    }

    /// Returns v2 payload files in raw-byte file-tree order.
    #[must_use]
    pub fn files(&self) -> &[V2File<'a>] {
        &self.files
    }

    /// Returns the checked aggregate payload length.
    #[must_use]
    pub const fn total_length(&self) -> u64 {
        self.total_length
    }

    /// Returns the checked aggregate number of file pieces.
    #[must_use]
    pub const fn piece_count(&self) -> u64 {
        self.piece_count
    }
}

pub(crate) fn checked_total_length(
    lengths: impl IntoIterator<Item = u64>,
    field: &'static str,
) -> Result<u64> {
    lengths.into_iter().try_fold(0_u64, |total, length| {
        total
            .checked_add(length)
            .ok_or_else(|| Error::metainfo_field(field, "total payload length overflowed"))
    })
}

pub(crate) fn checked_piece_count(
    lengths: impl IntoIterator<Item = u64>,
    piece_length: u64,
    field: &'static str,
) -> Result<u64> {
    lengths.into_iter().try_fold(0_u64, |total, length| {
        total
            .checked_add(length.div_ceil(piece_length))
            .ok_or_else(|| Error::metainfo_field(field, "aggregate piece count overflowed"))
    })
}

struct PendingV2File<'a> {
    length: u64,
    path: Vec<&'a [u8]>,
    pieces_root: Option<&'a [u8; 32]>,
    attributes: &'a [u8],
    properties: &'a Value<'a>,
}

fn walk_file_tree<'a>(
    entries: &'a [(ByteString<'a>, Value<'a>)],
    path: &mut Vec<&'a [u8]>,
    files: &mut Vec<PendingV2File<'a>>,
) -> Result<()> {
    for (key, value) in entries {
        if key.bytes().is_empty() {
            return Err(Error::metainfo_field(
                "info.file tree",
                "file marker cannot be a sibling of child entries",
            ));
        }
        validate_path_component(key.bytes(), "info.file tree")?;
        path.push(key.bytes());
        let node = dictionary_entries(value, "info.file tree")?;
        if let Some(properties) = unique_field(node, b"", "info.file tree")? {
            if node.len() != 1 {
                return Err(Error::metainfo_field(
                    "info.file tree",
                    "file properties cannot have sibling child entries",
                ));
            }
            files.push(parse_v2_file(properties, path)?);
        } else {
            if node.is_empty() {
                return Err(Error::metainfo_field(
                    "info.file tree",
                    "directory node must not be empty",
                ));
            }
            walk_file_tree(node, path, files)?;
        }
        path.pop();
    }
    Ok(())
}

fn parse_v2_file<'a>(properties: &'a Value<'a>, path: &[&'a [u8]]) -> Result<PendingV2File<'a>> {
    let entries = dictionary_entries(properties, "info.file tree")?;
    let length = non_negative_value(
        required_field(entries, b"length", "info.file tree.length")?,
        "info.file tree.length",
    )?;
    let root = unique_field(entries, b"pieces root", "info.file tree.pieces root")?
        .map(|value| value_bytes(value, "info.file tree.pieces root"))
        .transpose()?;
    let pieces_root = match (length, root) {
        (0, None) => None,
        (0, Some(_)) => {
            return Err(Error::metainfo_field(
                "info.file tree.pieces root",
                "empty files must not have a pieces root",
            ));
        }
        (_, None) => {
            return Err(Error::metainfo_field(
                "info.file tree.pieces root",
                "non-empty files require a pieces root",
            ));
        }
        (_, Some(root)) => Some(<&[u8; 32]>::try_from(root).map_err(|_| {
            Error::metainfo_field("info.file tree.pieces root", "must be exactly 32 bytes")
        })?),
    };
    let attributes = unique_field(entries, b"attr", "info.file tree.attr")?
        .map_or(Ok(&[][..]), |value| {
            value_bytes(value, "info.file tree.attr")
        })?;
    Ok(PendingV2File {
        length,
        path: path.to_vec(),
        pieces_root,
        attributes,
        properties,
    })
}

fn validate_piece_layers<'a>(
    pending: Vec<PendingV2File<'a>>,
    layer_entries: &'a [(ByteString<'a>, Value<'a>)],
    piece_length: u64,
) -> Result<Vec<V2File<'a>>> {
    let mut used = vec![false; layer_entries.len()];
    let mut files = Vec::with_capacity(pending.len());
    for file in pending {
        let piece_count = file.length.div_ceil(piece_length);
        let piece_layer = if piece_count > 1 {
            let root = file.pieces_root.expect("non-empty files have roots");
            let mut matches =
                layer_entries
                    .iter()
                    .enumerate()
                    .filter_map(|(index, (key, value))| {
                        (key.bytes() == root).then_some((index, value))
                    });
            let Some((index, value)) = matches.next() else {
                return Err(Error::metainfo_field(
                    "piece layers",
                    "missing layer for a multi-piece file",
                ));
            };
            if matches.next().is_some() {
                return Err(Error::metainfo_field(
                    "piece layers",
                    "duplicate pieces root",
                ));
            }
            let bytes = value_bytes(value, "piece layers")?;
            let expected_len = usize::try_from(piece_count)
                .ok()
                .and_then(|count| count.checked_mul(32))
                .ok_or_else(|| Error::metainfo_field("piece layers", "length overflowed"))?;
            if bytes.len() != expected_len {
                return Err(Error::metainfo_field(
                    "piece layers",
                    "layer length does not match file piece count",
                ));
            }
            if merkle_root_from_piece_layer(bytes, piece_length)? != *root {
                return Err(Error::metainfo_field(
                    "piece layers",
                    "layer hashes do not match the pieces root",
                ));
            }
            used[index] = true;
            Some(bytes)
        } else {
            None
        };
        files.push(V2File {
            length: file.length,
            path: file.path,
            pieces_root: file.pieces_root,
            attributes: file.attributes,
            piece_layer,
            properties: file.properties,
        });
    }
    if used.iter().any(|used| !used) {
        return Err(Error::metainfo_field(
            "piece layers",
            "contains a layer not required by the file tree",
        ));
    }
    Ok(files)
}

fn merkle_root_from_piece_layer(bytes: &[u8], piece_length: u64) -> Result<[u8; 32]> {
    let mut hashes = bytes
        .chunks_exact(32)
        .map(|chunk| <[u8; 32]>::try_from(chunk).expect("exact chunks"))
        .collect::<Vec<_>>();
    let target = hashes.len().next_power_of_two();
    hashes.resize(target, zero_hash_for_piece_length(piece_length));
    while hashes.len() > 1 {
        hashes = hashes
            .chunks_exact(2)
            .map(|pair| hash_pair(pair[0], pair[1]))
            .collect();
    }
    hashes
        .pop()
        .ok_or_else(|| Error::metainfo_field("piece layers", "layer must not be empty"))
}

fn zero_hash_for_piece_length(piece_length: u64) -> [u8; 32] {
    let mut hash = [0; 32];
    let mut covered = 16_384;
    while covered < piece_length {
        hash = hash_pair(hash, hash);
        covered *= 2;
    }
    hash
}

fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    use sha2::Digest as _;
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

fn validate_hybrid(v1: &V1Metainfo<'_>, v2: &[V2File<'_>], piece_length: u64) -> Result<()> {
    if v1.piece_length() != piece_length {
        return Err(Error::metainfo_field(
            "info.piece length",
            "v1 and v2 piece lengths differ",
        ));
    }
    let mut real_files = Vec::new();
    let mut offset = 0_u64;
    let mut previous_was_padding = false;
    for (index, file) in v1.files().iter().enumerate() {
        if file.attributes().contains(&b'p') {
            if !file.is_padding() {
                return Err(Error::metainfo_field(
                    "info.files.path",
                    "padding file must use .pad/<length> or .pad/<offset>-<length>",
                ));
            }
            let required = (piece_length - (offset % piece_length)) % piece_length;
            let [_, name] = file.path_components() else {
                unreachable!("validated padding paths have two components");
            };
            let (encoded_offset, encoded_length) =
                parse_padding_name(name).expect("validated padding path has decimal length");
            if index == 0
                || index + 1 == v1.files().len()
                || previous_was_padding
                || required == 0
                || file.length() != required
                || encoded_offset.is_some_and(|encoded_offset| encoded_offset != offset)
                || encoded_length != file.length()
            {
                return Err(Error::metainfo_field(
                    "info.files.attr",
                    "padding file has invalid placement, path, or alignment length",
                ));
            }
            offset = offset.checked_add(file.length()).ok_or_else(|| {
                Error::metainfo_field("info.files.length", "hybrid offset overflowed")
            })?;
            previous_was_padding = true;
            continue;
        }
        if offset % piece_length != 0 {
            return Err(Error::metainfo_field(
                "info.files",
                "hybrid real file is not piece-aligned",
            ));
        }
        if file.path_components().first() == Some(&b".pad".as_slice()) {
            return Err(Error::metainfo_field(
                "info.files.path",
                "hybrid real files must not use the reserved .pad directory",
            ));
        }
        real_files.push(file);
        offset = offset.checked_add(file.length()).ok_or_else(|| {
            Error::metainfo_field("info.files.length", "hybrid offset overflowed")
        })?;
        previous_was_padding = false;
    }
    if real_files.len() != v2.len() {
        return Err(Error::metainfo_field(
            "info.files",
            "v1 and v2 file counts differ",
        ));
    }
    for (v1_file, v2_file) in real_files.into_iter().zip(v2) {
        if v1_file.length() != v2_file.length()
            || v1_file.path_components() != v2_file.path_components()
        {
            return Err(Error::metainfo_field(
                "info.files",
                "v1 and v2 paths or lengths differ",
            ));
        }
    }
    Ok(())
}

/// Owned hash value for a v1 SHA-1 info hash.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InfoHashV1([u8; 20]);

impl InfoHashV1 {
    pub(crate) const fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Returns the raw 20-byte digest.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Returns lowercase hexadecimal text.
    #[must_use]
    pub fn hex(&self) -> String {
        encode_hex(&self.0)
    }
}

impl std::fmt::Display for InfoHashV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.hex())
    }
}

/// Owned hash value for a v2 SHA-256 info hash.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InfoHashV2([u8; 32]);

impl InfoHashV2 {
    pub(crate) const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw 32-byte digest.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns lowercase hexadecimal text.
    #[must_use]
    pub fn hex(&self) -> String {
        encode_hex(&self.0)
    }
}

impl std::fmt::Display for InfoHashV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.hex())
    }
}

/// Owned payload file entry exposed by the inspection API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TorrentFile {
    length: u64,
    path: Vec<Vec<u8>>,
    attributes: Vec<u8>,
    pieces_root: Option<[u8; 32]>,
}

impl TorrentFile {
    /// Returns the payload file length.
    #[must_use]
    pub const fn length(&self) -> u64 {
        self.length
    }

    /// Returns raw torrent path components.
    #[must_use]
    pub fn path_components(&self) -> &[Vec<u8>] {
        &self.path
    }

    /// Returns an owned raw-identity torrent path value.
    #[must_use]
    pub fn torrent_path(&self) -> TorrentPath {
        TorrentPath::from_raw(&self.path)
    }

    /// Returns UTF-8 path components when every component is valid UTF-8.
    #[must_use]
    pub fn path_utf8(&self) -> Option<Vec<&str>> {
        self.path
            .iter()
            .map(|component| std::str::from_utf8(component).ok())
            .collect()
    }

    /// Returns file attribute bytes.
    #[must_use]
    pub fn attributes(&self) -> &[u8] {
        &self.attributes
    }

    /// Returns whether this owned entry represents validated hybrid padding.
    #[must_use]
    pub fn is_padding(&self) -> bool {
        let path = self.path.iter().map(Vec::as_slice).collect::<Vec<_>>();
        self.attributes.contains(&b'p') && is_padding_path(&path, self.length)
    }

    /// Returns the v2 pieces root when applicable.
    #[must_use]
    pub const fn pieces_root(&self) -> Option<&[u8; 32]> {
        self.pieces_root.as_ref()
    }
}

/// An owned unknown top-level metainfo field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownField {
    key: Vec<u8>,
    value: crate::bencode::OwnedValue,
}

impl UnknownField {
    /// Returns the raw field key.
    #[must_use]
    pub fn key(&self) -> &[u8] {
        &self.key
    }

    /// Returns the owned bencode value.
    #[must_use]
    pub const fn value(&self) -> &crate::bencode::OwnedValue {
        &self.value
    }
}

/// Result of metainfo-only validation.
/// Canonical encoding status for parseable source bytes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum Canonicality {
    /// The original bytes are canonical bencode.
    #[default]
    Canonical,
    /// The original bytes are parseable but not canonical.
    NonCanonical {
        /// Byte offset where canonical validation detected the issue.
        offset: usize,
        /// Human-readable canonicality explanation.
        message: String,
    },
}

impl Canonicality {
    /// Returns true when the original source is canonically encoded.
    #[must_use]
    pub const fn is_canonical(&self) -> bool {
        matches!(self, Self::Canonical)
    }

    /// Returns the canonicality issue offset, when non-canonical.
    #[must_use]
    pub const fn offset(&self) -> Option<usize> {
        match self {
            Self::Canonical => None,
            Self::NonCanonical { offset, .. } => Some(*offset),
        }
    }

    /// Returns the canonicality explanation, when non-canonical.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Canonical => None,
            Self::NonCanonical { message, .. } => Some(message),
        }
    }
}

fn canonicality(bytes: &[u8], root: &Value<'_>) -> Canonicality {
    match crate::bencode::validate_parsed_canonical(bytes, root) {
        Ok(()) => Canonicality::Canonical,
        Err(error) if error.category() == ErrorCategory::BencodeCanonical => {
            let offset = error.offset().unwrap_or_default();
            let Error::BencodeCanonical { message, .. } = error else {
                unreachable!("canonical validation returns canonical errors");
            };
            Canonicality::NonCanonical { offset, message }
        }
        Err(_) => unreachable!("already parsed bytes remain syntactically valid"),
    }
}

/// Structured non-fatal compatibility warning.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ValidationWarning {
    message: String,
    field: Option<String>,
    offset: Option<usize>,
}

impl ValidationWarning {
    /// Returns the warning message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the related field path, when applicable.
    #[must_use]
    pub fn field(&self) -> Option<&str> {
        self.field.as_deref()
    }

    /// Returns the related source offset, when applicable.
    #[must_use]
    pub const fn offset(&self) -> Option<usize> {
        self.offset
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValidationReport {
    warnings: Vec<String>,
    warning_details: Vec<ValidationWarning>,
    canonicality: Canonicality,
}

impl ValidationReport {
    /// Returns true when no structural error was found.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        true
    }

    /// Returns non-fatal compatibility warnings.
    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Returns structured non-fatal compatibility warnings.
    #[must_use]
    pub fn warning_details(&self) -> &[ValidationWarning] {
        &self.warning_details
    }

    /// Returns canonicality separately from protocol validity.
    #[must_use]
    pub const fn canonicality(&self) -> &Canonicality {
        &self.canonicality
    }
}

/// Owned, validated metainfo inspection object.
///
/// Input bytes are copied so this value does not borrow its source. The exact
/// original bytes remain available, while [`Metainfo::to_bytes`] and
/// [`Metainfo::write_canonical`] emit a canonical re-encoding.
///
/// # Examples
///
/// ```
/// use btpc_core::{Metainfo, TorrentMode};
///
/// let bytes = b"d4:infod6:lengthi0e4:name5:empty12:piece lengthi16e6:pieces0:ee";
/// let torrent = Metainfo::from_bytes(bytes)?;
/// assert_eq!(torrent.mode(), TorrentMode::V1);
/// assert_eq!(torrent.name_utf8(), Some("empty"));
/// assert_eq!(torrent.original_bytes(), bytes);
/// # Ok::<(), btpc_core::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Metainfo {
    original: Vec<u8>,
    canonical_root: crate::bencode::OwnedValue,
    canonical: std::sync::OnceLock<Vec<u8>>,
    mode: TorrentMode,
    name: Vec<u8>,
    piece_length: u64,
    total_length: u64,
    piece_count: u64,
    files: Vec<TorrentFile>,
    optional_metadata: OptionalMetadata,
    private: Option<bool>,
    info_hash_v1: Option<InfoHashV1>,
    info_hash_v2: Option<InfoHashV2>,
    unknown_fields: Vec<UnknownField>,
    validation: ValidationReport,
    parse_options: ParseOptions,
}

impl Metainfo {
    /// Generates a deterministic magnet URI.
    #[must_use]
    pub fn magnet(&self, options: &crate::magnet::MagnetOptions) -> String {
        crate::magnet::generate(self, options)
    }

    /// Parses and validates metainfo from bytes, copying the input.
    ///
    /// # Errors
    ///
    /// Returns a bencode, resource-limit, or protocol validation error.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_bytes_with_options(bytes, ParseOptions::default())
    }

    /// Parses and validates metainfo using caller-provided limits.
    ///
    /// # Errors
    ///
    /// Returns a bencode, resource-limit, or protocol validation error.
    pub fn from_bytes_with_options(bytes: &[u8], options: ParseOptions) -> Result<Self> {
        Self::from_owned_bytes_with_options(bytes.to_vec(), options)
    }

    #[doc(hidden)]
    pub fn from_owned_bytes_with_options(bytes: Vec<u8>, options: ParseOptions) -> Result<Self> {
        let limits = options.limits();
        limits.check_total_input(bytes.len())?;
        let mut budget = AllocationBudget::new(limits);
        let root = parse_with_budget(&bytes, limits, &mut budget)?;
        budget.charge(owned_load_cost(&root, bytes.len(), true)?)?;
        let raw = RawMetainfo::from_parsed(&bytes, root)?;
        let source_canonicality = raw.canonicality();
        let info_entries = dictionary_entries(raw.info_value(), "info")?;
        let mut snapshot = inspection_snapshot(&raw, info_entries)?;
        let canonical_root = value_to_owned(raw.root())?;
        let (optional_metadata, optional_warnings) = parse_optional_metadata(&raw, info_entries)?;
        snapshot.warnings.extend(optional_warnings);
        let private = unique_field(info_entries, b"private", "info.private")?
            .map(|value| {
                value
                    .integer()
                    .ok_or_else(|| Error::metainfo_field("info.private", "must be an integer"))
                    .and_then(|value| match value.to_i64() {
                        Some(0) => Ok(false),
                        Some(1) => Ok(true),
                        _ => Err(Error::metainfo_field("info.private", "must be 0 or 1")),
                    })
            })
            .transpose()?;
        let unknown_fields = raw
            .unknown_fields()
            .into_iter()
            .map(|(key, value)| {
                Ok(UnknownField {
                    key: key.bytes().to_vec(),
                    value: value_to_owned(value)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let info_hash_v1 = matches!(snapshot.mode, TorrentMode::V1 | TorrentMode::Hybrid)
            .then(|| InfoHashV1(raw.info_hash_sha1()));
        let info_hash_v2 = matches!(snapshot.mode, TorrentMode::V2 | TorrentMode::Hybrid)
            .then(|| InfoHashV2(raw.info_hash_sha256()));
        drop(raw);
        Ok(Self {
            original: bytes,
            canonical_root,
            canonical: std::sync::OnceLock::new(),
            mode: snapshot.mode,
            name: snapshot.name,
            piece_length: snapshot.piece_length,
            total_length: snapshot.total_length,
            piece_count: snapshot.piece_count,
            files: snapshot.files,
            optional_metadata,
            private,
            info_hash_v1,
            info_hash_v2,
            unknown_fields,
            validation: ValidationReport {
                warnings: snapshot
                    .warnings
                    .iter()
                    .map(|warning| warning.message.clone())
                    .collect(),
                warning_details: snapshot.warnings,
                canonicality: source_canonicality,
            },
            parse_options: options,
        })
    }

    /// Reads, parses, and validates metainfo from a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns a contextual I/O error or any error from [`Metainfo::from_bytes`].
    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self> {
        Self::from_path_with_options(path, ParseOptions::default())
    }

    /// Reads, parses, and validates metainfo using caller-provided limits.
    ///
    /// Regular files are checked before allocation. The actual read is capped at
    /// one byte beyond the configured maximum so growth races and streams are
    /// still rejected without unbounded allocation.
    ///
    /// # Errors
    ///
    /// Returns a contextual I/O, resource-limit, bencode, or validation error.
    pub fn from_path_with_options(
        path: impl AsRef<std::path::Path>,
        options: ParseOptions,
    ) -> Result<Self> {
        use std::io::Read as _;

        let path = path.as_ref();
        let limits = options.limits();
        let file = std::fs::File::open(path).map_err(|source| Error::io(path, source))?;
        let metadata = file.metadata().map_err(|source| Error::io(path, source))?;
        if metadata.is_file() {
            limits.check_total_input(usize::try_from(metadata.len()).unwrap_or(usize::MAX))?;
        }
        let maximum = limits.max_total_input();
        let initial = usize::try_from(metadata.len())
            .unwrap_or(maximum)
            .min(maximum);
        let mut bytes = Vec::with_capacity(initial);
        let read_limit = maximum.saturating_add(1);
        file.take(u64::try_from(read_limit).unwrap_or(u64::MAX))
            .read_to_end(&mut bytes)
            .map_err(|source| Error::io(path, source))?;
        limits.check_total_input(bytes.len())?;
        Self::from_owned_bytes_with_options(bytes, options)
    }

    /// Returns the resource limits used to load this object.
    #[must_use]
    pub const fn parse_options(&self) -> ParseOptions {
        self.parse_options
    }

    /// Returns the validated protocol mode.
    #[must_use]
    pub const fn mode(&self) -> TorrentMode {
        self.mode
    }

    /// Returns the raw torrent name bytes.
    #[must_use]
    pub fn name(&self) -> &[u8] {
        &self.name
    }

    /// Returns the torrent name as UTF-8 when valid.
    #[must_use]
    pub fn name_utf8(&self) -> Option<&str> {
        std::str::from_utf8(&self.name).ok()
    }

    /// Returns payload files.
    #[must_use]
    pub fn files(&self) -> &[TorrentFile] {
        &self.files
    }

    /// Returns the piece length in bytes.
    #[must_use]
    pub const fn piece_length(&self) -> u64 {
        self.piece_length
    }

    /// Returns total payload bytes.
    #[must_use]
    pub const fn total_length(&self) -> u64 {
        self.total_length
    }

    /// Returns the logical piece count for the active representation.
    #[must_use]
    pub const fn piece_count(&self) -> u64 {
        self.piece_count
    }

    /// Returns tracker tiers as raw byte strings.
    #[must_use]
    pub fn trackers(&self) -> &[Vec<Vec<u8>>] {
        self.optional_metadata.trackers()
    }

    /// Returns web seed URLs as raw byte strings.
    #[must_use]
    pub fn web_seeds(&self) -> &[Vec<u8>] {
        self.optional_metadata.web_seeds()
    }

    /// Returns all validated optional metadata in one lossless owned model.
    #[must_use]
    pub const fn optional_metadata(&self) -> &OptionalMetadata {
        &self.optional_metadata
    }

    /// Returns validated DHT bootstrap nodes.
    #[must_use]
    pub fn nodes(&self) -> &[DhtNode] {
        self.optional_metadata.nodes()
    }

    /// Returns the private flag when explicitly present.
    #[must_use]
    pub const fn private(&self) -> Option<bool> {
        self.private
    }

    /// Returns raw source bytes when explicitly present in the info dictionary.
    #[must_use]
    pub fn source(&self) -> Option<&[u8]> {
        self.optional_metadata.source()
    }

    /// Returns raw top-level comment bytes when present.
    #[must_use]
    pub fn comment(&self) -> Option<&[u8]> {
        self.optional_metadata.comment()
    }

    /// Returns raw top-level creator bytes when present.
    #[must_use]
    pub fn created_by(&self) -> Option<&[u8]> {
        self.optional_metadata.created_by()
    }

    /// Returns the top-level creation timestamp when representable as `i64`.
    #[must_use]
    pub const fn creation_date(&self) -> Option<i64> {
        self.optional_metadata.creation_date()
    }

    /// Returns the v1 info hash when applicable.
    #[must_use]
    pub const fn info_hash_v1(&self) -> Option<InfoHashV1> {
        self.info_hash_v1
    }

    /// Returns the v2 info hash when applicable.
    #[must_use]
    pub const fn info_hash_v2(&self) -> Option<InfoHashV2> {
        self.info_hash_v2
    }

    /// Returns exact input bytes.
    #[must_use]
    pub fn original_bytes(&self) -> &[u8] {
        &self.original
    }

    /// Returns unknown top-level extension fields.
    #[must_use]
    pub fn unknown_fields(&self) -> &[UnknownField] {
        &self.unknown_fields
    }

    /// Returns the successful validation report retained at construction.
    #[must_use]
    pub const fn validate(&self) -> &ValidationReport {
        &self.validation
    }

    /// Writes exact original input bytes.
    ///
    /// # Errors
    ///
    /// Returns an I/O error preserving the writer's error as its source.
    pub fn write_original(&self, writer: &mut impl std::io::Write) -> Result<()> {
        writer
            .write_all(&self.original)
            .map_err(|source| Error::io("<writer>", source))
    }

    /// Writes canonical bencode for this unchanged parsed object.
    ///
    /// # Errors
    ///
    /// Returns an I/O error preserving the writer's error as its source.
    pub fn write_canonical(&self, writer: &mut impl std::io::Write) -> Result<()> {
        writer
            .write_all(self.canonical_bytes()?)
            .map_err(|source| Error::io("<writer>", source))
    }

    /// Returns canonical bencode bytes.
    ///
    /// # Errors
    ///
    /// This unchanged owned snapshot is already validated, so this method does
    /// not currently fail; the result type preserves serialization API stability.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.canonical_bytes()?.clone())
    }

    /// Returns whether canonical bytes have been materialized.
    #[doc(hidden)]
    #[must_use]
    pub fn canonical_bytes_cached(&self) -> bool {
        self.canonical.get().is_some()
    }

    fn canonical_bytes(&self) -> Result<&Vec<u8>> {
        if let Some(bytes) = self.canonical.get() {
            return Ok(bytes);
        }
        let bytes = self.canonical_root.to_vec()?;
        let _ = self.canonical.set(bytes);
        Ok(self
            .canonical
            .get()
            .expect("canonical bytes were initialized"))
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn owned_load_cost(root: &Value<'_>, input_length: usize, copy_original: bool) -> Result<usize> {
    let tree = owned_tree_cost(root)?;
    let repeated_tree = tree.checked_mul(2).ok_or_else(owned_allocation_overflow)?;
    let original = if copy_original { input_length } else { 0 };
    original
        .checked_add(repeated_tree)
        .ok_or_else(owned_allocation_overflow)
}

fn owned_tree_cost(value: &Value<'_>) -> Result<usize> {
    let base = std::mem::size_of::<crate::bencode::OwnedValue>();
    match value.kind() {
        ValueKind::Integer(value) => base
            .checked_add(value.encoded().len())
            .ok_or_else(owned_allocation_overflow),
        ValueKind::Bytes(bytes) => base
            .checked_add(bytes.len())
            .ok_or_else(owned_allocation_overflow),
        ValueKind::List(values) => values.iter().try_fold(
            base.checked_add(
                values
                    .len()
                    .checked_mul(std::mem::size_of::<crate::bencode::OwnedValue>())
                    .ok_or_else(owned_allocation_overflow)?,
            )
            .ok_or_else(owned_allocation_overflow)?,
            |total, value| {
                total
                    .checked_add(owned_tree_cost(value)?)
                    .ok_or_else(owned_allocation_overflow)
            },
        ),
        ValueKind::Dictionary(entries) => entries.iter().try_fold(
            base.checked_add(
                entries
                    .len()
                    .checked_mul(
                        std::mem::size_of::<(Vec<u8>, crate::bencode::OwnedValue)>()
                            + 3 * std::mem::size_of::<usize>(),
                    )
                    .ok_or_else(owned_allocation_overflow)?,
            )
            .ok_or_else(owned_allocation_overflow)?,
            |total, (key, value)| {
                total
                    .checked_add(key.bytes().len())
                    .and_then(|total| total.checked_add(owned_tree_cost(value).ok()?))
                    .ok_or_else(owned_allocation_overflow)
            },
        ),
    }
}

fn owned_allocation_overflow() -> Error {
    Error::resource_limit("owned allocation", usize::MAX, usize::MAX)
}

fn value_to_owned(value: &Value<'_>) -> Result<crate::bencode::OwnedValue> {
    use crate::bencode::OwnedValue;
    match value.kind() {
        ValueKind::Integer(value) => OwnedValue::integer_bytes(value.canonical_bytes()),
        ValueKind::Bytes(bytes) => Ok(OwnedValue::bytes(bytes.to_vec())),
        ValueKind::List(values) => values
            .iter()
            .map(value_to_owned)
            .collect::<Result<Vec<_>>>()
            .map(OwnedValue::list),
        ValueKind::Dictionary(entries) => OwnedValue::dictionary(
            entries
                .iter()
                .map(|(key, value)| Ok((key.bytes().to_vec(), value_to_owned(value)?)))
                .collect::<Result<Vec<_>>>()?,
        ),
    }
}

fn parse_optional_metadata(
    raw: &RawMetainfo<'_>,
    info: &[(ByteString<'_>, Value<'_>)],
) -> Result<(OptionalMetadata, Vec<ValidationWarning>)> {
    let root = dictionary_entries(raw.root(), "<root>")?;
    let announce = unique_field(root, b"announce", "announce")?
        .map(|value| value_bytes(value, "announce").map(ToOwned::to_owned))
        .transpose()?;
    if announce.as_ref().is_some_and(Vec::is_empty) {
        return Err(Error::metainfo_field("announce", "tracker URL is empty"));
    }
    let announce_list = unique_field(root, b"announce-list", "announce-list")?;
    let mut warnings = Vec::new();
    let trackers = if let Some(value) = announce_list {
        let tiers = list_values(value, "announce-list")?;
        if tiers.is_empty() {
            warnings.push(ValidationWarning {
                message: "empty announce-list ignored in favor of announce".into(),
                field: Some("announce-list".into()),
                offset: Some(value.span().start()),
            });
            announce.into_iter().map(|url| vec![url]).collect()
        } else {
            tiers
                .iter()
                .map(|tier| {
                    list_values(tier, "announce-list")?
                        .iter()
                        .map(|tracker| Ok(value_bytes(tracker, "announce-list")?.to_vec()))
                        .collect()
                })
                .collect::<Result<Vec<_>>>()?
        }
    } else {
        announce.into_iter().map(|url| vec![url]).collect()
    };
    validate_tracker_tiers(&trackers)?;

    let web_seeds = match unique_field(root, b"url-list", "url-list")? {
        None => Vec::new(),
        Some(value) if value.as_bytes().is_some() => {
            vec![value_bytes(value, "url-list")?.to_vec()]
        }
        Some(value) => list_values(value, "url-list")?
            .iter()
            .map(|seed| Ok(value_bytes(seed, "url-list")?.to_vec()))
            .collect::<Result<Vec<_>>>()?,
    };
    validate_web_seeds(&web_seeds)?;

    let nodes = parse_nodes(unique_field(root, b"nodes", "nodes")?)?;
    validate_nodes(&nodes)?;

    let source = unique_field(info, b"source", "info.source")?
        .map(|value| value_bytes(value, "info.source").map(ToOwned::to_owned))
        .transpose()?;
    let comment = unique_field(root, b"comment", "comment")?
        .map(|value| value_bytes(value, "comment").map(ToOwned::to_owned))
        .transpose()?;
    let created_by = unique_field(root, b"created by", "created by")?
        .map(|value| value_bytes(value, "created by").map(ToOwned::to_owned))
        .transpose()?;
    let creation_date = unique_field(root, b"creation date", "creation date")?
        .map(|value| {
            value
                .integer()
                .and_then(crate::bencode::Integer::to_i64)
                .ok_or_else(|| Error::metainfo_field("creation date", "must be an i64 integer"))
        })
        .transpose()?;
    validate_creation_date(creation_date)?;
    Ok((
        OptionalMetadata::new(
            trackers,
            web_seeds,
            nodes
                .into_iter()
                .map(|(host, port)| DhtNode::new(host, port))
                .collect(),
            source,
            comment,
            created_by,
            creation_date,
        ),
        warnings,
    ))
}

fn parse_nodes(value: Option<&Value<'_>>) -> Result<Vec<(Vec<u8>, u16)>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    list_values(value, "nodes")?
        .iter()
        .map(|node| {
            let pair = list_values(node, "nodes")?;
            if pair.len() != 2 {
                return Err(Error::metainfo_field(
                    "nodes",
                    "each node must be [host, port]",
                ));
            }
            let host = value_bytes(&pair[0], "nodes.host")?.to_vec();
            let port = pair[1]
                .integer()
                .and_then(crate::bencode::Integer::to_i64)
                .and_then(|value| u16::try_from(value).ok())
                .ok_or_else(|| {
                    Error::metainfo_field("nodes.port", "must be an integer in 1..=65535")
                })?;
            Ok((host, port))
        })
        .collect()
}

struct InspectionSnapshot {
    mode: TorrentMode,
    name: Vec<u8>,
    piece_length: u64,
    total_length: u64,
    piece_count: u64,
    files: Vec<TorrentFile>,
    warnings: Vec<ValidationWarning>,
}

fn inspection_snapshot(
    raw: &RawMetainfo<'_>,
    info: &[(ByteString<'_>, Value<'_>)],
) -> Result<InspectionSnapshot> {
    if unique_field(info, b"meta version", "info.meta version")?.is_some() {
        let typed = V2Metainfo::from_raw(raw)?;
        let mut warnings = Vec::new();
        if raw
            .field(b"piece layers")
            .map(|layers| dictionary_entries(layers, "piece layers"))
            .transpose()?
            .is_some_and(<[_]>::is_empty)
        {
            warnings.push(ValidationWarning {
                message: "piece layers is present but empty; canonical output omits it".into(),
                field: Some("piece layers".into()),
                offset: raw.field(b"piece layers").map(|value| value.span().start()),
            });
        }
        Ok(InspectionSnapshot {
            mode: typed.mode(),
            name: typed.name().to_vec(),
            piece_length: typed.piece_length(),
            total_length: typed.total_length(),
            piece_count: typed.piece_count(),
            files: if typed.mode() == TorrentMode::Hybrid {
                V1Metainfo::from_raw(raw)?
                    .files()
                    .iter()
                    .map(|file| TorrentFile {
                        length: file.length(),
                        path: file
                            .path_components()
                            .iter()
                            .map(|part| part.to_vec())
                            .collect(),
                        attributes: file.attributes().to_vec(),
                        pieces_root: typed
                            .files()
                            .iter()
                            .find(|candidate| candidate.path_components() == file.path_components())
                            .and_then(V2File::pieces_root)
                            .copied(),
                    })
                    .collect()
            } else {
                typed
                    .files()
                    .iter()
                    .map(|file| TorrentFile {
                        length: file.length(),
                        path: file
                            .path_components()
                            .iter()
                            .map(|part| part.to_vec())
                            .collect(),
                        attributes: file.attributes().to_vec(),
                        pieces_root: file.pieces_root().copied(),
                    })
                    .collect()
            },
            warnings,
        })
    } else {
        let typed = V1Metainfo::from_raw(raw)?;
        Ok(InspectionSnapshot {
            mode: typed.mode(),
            name: typed.name().to_vec(),
            piece_length: typed.piece_length(),
            total_length: typed.total_length(),
            piece_count: u64::try_from(typed.pieces().len() / 20)
                .expect("usize values fit in u64 on supported targets"),
            files: typed
                .files()
                .iter()
                .map(|file| TorrentFile {
                    length: file.length(),
                    path: file
                        .path_components()
                        .iter()
                        .map(|part| part.to_vec())
                        .collect(),
                    attributes: file.attributes().to_vec(),
                    pieces_root: None,
                })
                .collect(),
            warnings: typed
                .warnings()
                .iter()
                .map(|message| ValidationWarning {
                    message: message.clone(),
                    field: None,
                    offset: None,
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod checked_arithmetic_tests {
    use super::{checked_piece_count, checked_total_length};
    use crate::ErrorCategory;

    #[test]
    fn aggregate_boundaries_return_structured_errors() {
        assert_eq!(
            checked_total_length([u64::MAX], "length").unwrap(),
            u64::MAX
        );
        let length_error = checked_total_length([u64::MAX, 1], "length").unwrap_err();
        assert_eq!(length_error.category(), ErrorCategory::Metainfo);
        assert_eq!(length_error.field(), Some("length"));

        assert_eq!(
            checked_piece_count([u64::MAX], 1, "pieces").unwrap(),
            u64::MAX
        );
        let piece_error = checked_piece_count([u64::MAX, 1], 1, "pieces").unwrap_err();
        assert_eq!(piece_error.category(), ErrorCategory::Metainfo);
        assert_eq!(piece_error.field(), Some("pieces"));
    }
}
