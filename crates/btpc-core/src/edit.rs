//! Validated owned metainfo editing.

use std::collections::BTreeMap;

use crate::bencode::{OwnedValue, Value, ValueKind, parse};
use crate::{Error, Metainfo, Result};

const RESERVED_TOP_LEVEL: &[&[u8]] = &[
    b"announce",
    b"announce-list",
    b"url-list",
    b"nodes",
    b"comment",
    b"created by",
    b"creation date",
    b"info",
    b"piece layers",
];

/// Owned metainfo editor preserving untouched known and unknown fields.
#[derive(Clone, Debug)]
pub struct MetainfoEditor {
    top_level: BTreeMap<Vec<u8>, OwnedValue>,
    info: EditorInfo,
}

#[derive(Clone, Debug)]
enum EditorInfo {
    Raw(Vec<u8>),
    Owned(BTreeMap<Vec<u8>, OwnedValue>),
}

impl MetainfoEditor {
    /// Starts from a parsed validated metainfo object.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or owned top-level conversion fails.
    pub fn from_metainfo(metainfo: &Metainfo) -> Result<Self> {
        let parsed = parse(metainfo.original_bytes())?;
        let ValueKind::Dictionary(entries) = parsed.kind() else {
            unreachable!("validated metainfo roots are dictionaries");
        };
        let mut top_level = BTreeMap::new();
        let mut info = None;
        for (key, value) in entries {
            if key.bytes() == b"info" {
                info = Some(EditorInfo::Raw(
                    metainfo.original_bytes()[value.span().range()].to_vec(),
                ));
            } else {
                top_level.insert(key.bytes().to_vec(), owned(value)?);
            }
        }
        Ok(Self {
            top_level,
            info: info.ok_or_else(|| Error::metainfo_field("info", "missing required field"))?,
        })
    }

    /// Starts from an owned validated info dictionary.
    ///
    /// # Errors
    ///
    /// Returns an error when `info` is not a dictionary.
    pub fn from_info(info: OwnedValue) -> Result<Self> {
        let OwnedValue::Dictionary(info) = info else {
            return Err(Error::metainfo_field("info", "must be a dictionary"));
        };
        Ok(Self {
            top_level: BTreeMap::new(),
            info: EditorInfo::Owned(info),
        })
    }

    /// Replaces tracker tiers.
    #[must_use]
    pub fn trackers(mut self, tiers: impl IntoIterator<Item = Vec<Vec<u8>>>) -> Self {
        let tiers = tiers.into_iter().collect::<Vec<_>>();
        self.top_level.remove(b"announce".as_slice());
        self.top_level.remove(b"announce-list".as_slice());
        if let Some(announce) = tiers.first().and_then(|tier| tier.first()) {
            self.top_level
                .insert(b"announce".to_vec(), OwnedValue::bytes(announce.clone()));
        }
        if !tiers.is_empty() {
            self.top_level.insert(
                b"announce-list".to_vec(),
                OwnedValue::list(
                    tiers
                        .into_iter()
                        .map(|tier| OwnedValue::list(tier.into_iter().map(OwnedValue::bytes))),
                ),
            );
        }
        self
    }

    /// Replaces web seed URLs.
    #[must_use]
    pub fn web_seeds(mut self, seeds: impl IntoIterator<Item = Vec<u8>>) -> Self {
        let seeds = seeds.into_iter().collect::<Vec<_>>();
        set_optional(
            &mut self.top_level,
            b"url-list",
            (!seeds.is_empty()).then(|| OwnedValue::list(seeds.into_iter().map(OwnedValue::bytes))),
        );
        self
    }

    /// Replaces DHT bootstrap nodes.
    #[must_use]
    pub fn nodes(mut self, nodes: impl IntoIterator<Item = (Vec<u8>, u16)>) -> Self {
        let nodes = nodes.into_iter().collect::<Vec<_>>();
        set_optional(
            &mut self.top_level,
            b"nodes",
            (!nodes.is_empty()).then(|| {
                OwnedValue::list(nodes.into_iter().map(|(host, port)| {
                    OwnedValue::list([
                        OwnedValue::bytes(host),
                        OwnedValue::integer(i64::from(port)),
                    ])
                }))
            }),
        );
        self
    }

    /// Sets or removes the top-level comment.
    #[must_use]
    pub fn comment(mut self, value: Option<Vec<u8>>) -> Self {
        set_optional(
            &mut self.top_level,
            b"comment",
            value.map(OwnedValue::bytes),
        );
        self
    }

    /// Sets or removes the top-level creator.
    #[must_use]
    pub fn created_by(mut self, value: Option<Vec<u8>>) -> Self {
        set_optional(
            &mut self.top_level,
            b"created by",
            value.map(OwnedValue::bytes),
        );
        self
    }

    /// Sets or removes the top-level creation timestamp.
    #[must_use]
    pub fn creation_date(mut self, value: Option<i64>) -> Self {
        set_optional(
            &mut self.top_level,
            b"creation date",
            value.map(OwnedValue::integer),
        );
        self
    }

    /// Sets or removes the private flag inside `info`.
    #[must_use]
    pub fn private(mut self, value: Option<bool>) -> Self {
        if let Ok(info) = self.info_mut() {
            set_optional(
                info,
                b"private",
                value.map(|value| OwnedValue::integer(i64::from(value))),
            );
        }
        self
    }

    /// Sets or removes the source field inside `info`.
    #[must_use]
    pub fn source(mut self, value: Option<Vec<u8>>) -> Self {
        if let Ok(info) = self.info_mut() {
            set_optional(info, b"source", value.map(OwnedValue::bytes));
        }
        self
    }

    /// Adds or replaces an unknown top-level extension.
    ///
    /// # Errors
    ///
    /// Returns an error for reserved keys.
    pub fn raw_top_level(mut self, key: Vec<u8>, value: OwnedValue) -> Result<Self> {
        if RESERVED_TOP_LEVEL.contains(&key.as_slice()) {
            return Err(Error::metainfo_field(
                "top-level field",
                "reserved key must use its typed editor method",
            ));
        }
        self.top_level.insert(key, value);
        Ok(self)
    }

    /// Sets file attributes for a validated v1 or v2 path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is absent or has an unsupported shape.
    pub fn file_attributes(mut self, path: &[Vec<u8>], attributes: Vec<u8>) -> Result<Self> {
        let attributes = attributes.into_boxed_slice();
        update_file_attributes(self.info_mut()?, path, &attributes, false)?;
        Ok(self)
    }

    /// Serializes top-level fields canonically and validates the edited result.
    ///
    /// Untouched `info` bytes retain their exact source encoding. Any info-level
    /// edit canonicalizes the updated `info` dictionary. Calling
    /// [`Metainfo::to_bytes`] on the result explicitly canonicalizes the complete
    /// metainfo object.
    ///
    /// # Errors
    ///
    /// Returns serialization or protocol validation errors.
    pub fn to_metainfo(self) -> Result<Metainfo> {
        let bytes = match self.info {
            EditorInfo::Raw(info) => encode_with_raw_info(self.top_level, &info)?,
            EditorInfo::Owned(info) => {
                let mut root = self.top_level;
                root.insert(b"info".to_vec(), OwnedValue::Dictionary(info));
                OwnedValue::Dictionary(root).to_vec()?
            }
        };
        Metainfo::from_bytes(&bytes)
    }

    fn info_mut(&mut self) -> Result<&mut BTreeMap<Vec<u8>, OwnedValue>> {
        if let EditorInfo::Raw(bytes) = &self.info {
            let parsed = parse(bytes)?;
            let OwnedValue::Dictionary(info) = owned(&parsed)? else {
                return Err(Error::metainfo_field("info", "must be a dictionary"));
            };
            self.info = EditorInfo::Owned(info);
        }
        let EditorInfo::Owned(info) = &mut self.info else {
            unreachable!("raw info was converted to owned info");
        };
        Ok(info)
    }
}

fn encode_with_raw_info(top_level: BTreeMap<Vec<u8>, OwnedValue>, info: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![b'd'];
    let mut wrote_info = false;
    for (key, value) in top_level {
        if !wrote_info && key.as_slice() > b"info".as_slice() {
            write_bytes(&mut output, b"info");
            output.extend_from_slice(info);
            wrote_info = true;
        }
        write_bytes(&mut output, &key);
        value.write_to(&mut output)?;
    }
    if !wrote_info {
        write_bytes(&mut output, b"info");
        output.extend_from_slice(info);
    }
    output.push(b'e');
    Ok(output)
}

fn write_bytes(output: &mut Vec<u8>, bytes: &[u8]) {
    output.extend_from_slice(bytes.len().to_string().as_bytes());
    output.push(b':');
    output.extend_from_slice(bytes);
}

fn update_v1_attributes(
    info: &mut BTreeMap<Vec<u8>, OwnedValue>,
    path: &[Vec<u8>],
    attributes: &[u8],
) -> Result<Option<bool>> {
    if let Some(files) = info.get_mut(b"files".as_slice()) {
        let OwnedValue::List(files) = files else {
            return Err(Error::metainfo_field("info.files", "must be a list"));
        };
        for file in files {
            let OwnedValue::Dictionary(fields) = file else {
                return Err(Error::metainfo_field(
                    "info.files",
                    "entry must be a dictionary",
                ));
            };
            if path_matches(fields.get(b"path".as_slice()), path) {
                let padding = matches!(
                    fields.get(b"attr".as_slice()),
                    Some(OwnedValue::Bytes(current)) if current.contains(&b'p')
                );
                set_optional(
                    fields,
                    b"attr",
                    (!attributes.is_empty()).then(|| OwnedValue::bytes(attributes.to_vec())),
                );
                return Ok(Some(padding));
            }
        }
    } else if path.len() == 1
        && info
            .get(b"name".as_slice())
            .is_some_and(|name| matches!(name, OwnedValue::Bytes(bytes) if bytes == &path[0]))
        && info.contains_key(b"length".as_slice())
    {
        let padding = matches!(
            info.get(b"attr".as_slice()),
            Some(OwnedValue::Bytes(current)) if current.contains(&b'p')
        );
        set_optional(
            info,
            b"attr",
            (!attributes.is_empty()).then(|| OwnedValue::bytes(attributes.to_vec())),
        );
        return Ok(Some(padding));
    }
    Ok(None)
}

fn update_v2_attributes(
    info: &mut BTreeMap<Vec<u8>, OwnedValue>,
    path: &[Vec<u8>],
    attributes: &[u8],
) -> Result<bool> {
    let Some(mut node) = info.get_mut(b"file tree".as_slice()) else {
        return Ok(false);
    };
    for component in path {
        let OwnedValue::Dictionary(entries) = node else {
            return Err(Error::metainfo_field(
                "info.file tree",
                "invalid path shape",
            ));
        };
        let Some(next) = entries.get_mut(component.as_slice()) else {
            return Ok(false);
        };
        node = next;
    }
    let OwnedValue::Dictionary(entries) = node else {
        return Err(Error::metainfo_field("info.file tree", "invalid leaf"));
    };
    let Some(properties) = entries.get_mut(b"".as_slice()) else {
        return Err(Error::metainfo_field(
            "info.file tree",
            "leaf marker missing",
        ));
    };
    let OwnedValue::Dictionary(properties) = properties else {
        return Err(Error::metainfo_field(
            "info.file tree",
            "invalid properties",
        ));
    };
    set_optional(
        properties,
        b"attr",
        (!attributes.is_empty()).then(|| OwnedValue::bytes(attributes.to_vec())),
    );
    Ok(true)
}

fn update_file_attributes(
    info: &mut BTreeMap<Vec<u8>, OwnedValue>,
    path: &[Vec<u8>],
    attributes: &[u8],
    inject_failure_after_v1: bool,
) -> Result<()> {
    let mut candidate = info.clone();
    let has_v1 =
        candidate.contains_key(b"files".as_slice()) || candidate.contains_key(b"length".as_slice());
    let has_v2 = candidate.contains_key(b"file tree".as_slice());
    let v1_padding = has_v1
        .then(|| update_v1_attributes(&mut candidate, path, attributes))
        .transpose()?
        .flatten();
    if inject_failure_after_v1 && v1_padding.is_some() {
        return Err(Error::metainfo_field("file attributes", "injected failure"));
    }
    if v1_padding == Some(true) {
        *info = candidate;
        return Ok(());
    }
    let v2_updated = has_v2 && update_v2_attributes(&mut candidate, path, attributes)?;
    match (has_v1, has_v2, v1_padding, v2_updated) {
        (true, true, Some(false), true)
        | (true, false, Some(false), false)
        | (false, true, None, true) => {
            *info = candidate;
            Ok(())
        }
        (true, true, _, _) => Err(Error::metainfo_field(
            "file path",
            "hybrid v1 and v2 representations are inconsistent",
        )),
        _ => Err(Error::metainfo_field("file path", "path not found")),
    }
}

fn set_optional(
    dictionary: &mut BTreeMap<Vec<u8>, OwnedValue>,
    key: &[u8],
    value: Option<OwnedValue>,
) {
    if let Some(value) = value {
        dictionary.insert(key.to_vec(), value);
    } else {
        dictionary.remove(key);
    }
}

fn path_matches(value: Option<&OwnedValue>, expected: &[Vec<u8>]) -> bool {
    let Some(OwnedValue::List(values)) = value else {
        return false;
    };
    values.len() == expected.len()
        && values.iter().zip(expected).all(
            |(value, expected)| matches!(value, OwnedValue::Bytes(actual) if actual == expected),
        )
}

fn owned(value: &Value<'_>) -> Result<OwnedValue> {
    match value.kind() {
        ValueKind::Integer(value) => OwnedValue::integer_bytes(value.canonical_bytes()),
        ValueKind::Bytes(bytes) => Ok(OwnedValue::bytes(bytes.to_vec())),
        ValueKind::List(values) => values
            .iter()
            .map(owned)
            .collect::<Result<Vec<_>>>()
            .map(OwnedValue::list),
        ValueKind::Dictionary(entries) => OwnedValue::dictionary(
            entries
                .iter()
                .map(|(key, value)| Ok((key.bytes().to_vec(), owned(value)?)))
                .collect::<Result<Vec<_>>>()?,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{OwnedValue, update_file_attributes};

    #[test]
    fn injected_hybrid_attribute_failure_leaves_info_unchanged() {
        let leaf = OwnedValue::dictionary([(
            Vec::new(),
            OwnedValue::dictionary([(b"length".to_vec(), OwnedValue::integer(1))]).unwrap(),
        )])
        .unwrap();
        let mut info = std::collections::BTreeMap::from([
            (
                b"files".to_vec(),
                OwnedValue::list([OwnedValue::dictionary([
                    (b"length".to_vec(), OwnedValue::integer(1)),
                    (
                        b"path".to_vec(),
                        OwnedValue::list([OwnedValue::bytes(b"file".to_vec())]),
                    ),
                ])
                .unwrap()]),
            ),
            (
                b"file tree".to_vec(),
                OwnedValue::dictionary([(b"file".to_vec(), leaf)]).unwrap(),
            ),
        ]);
        let original = info.clone();
        assert!(update_file_attributes(&mut info, &[b"file".to_vec()], b"x", true).is_err());
        assert_eq!(info, original);
    }
}
