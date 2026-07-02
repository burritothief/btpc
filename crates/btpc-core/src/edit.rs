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
    root: BTreeMap<Vec<u8>, OwnedValue>,
}

impl MetainfoEditor {
    /// Starts from a parsed validated metainfo object.
    ///
    /// # Errors
    ///
    /// Returns an error if canonical owned conversion unexpectedly fails.
    pub fn from_metainfo(metainfo: &Metainfo) -> Result<Self> {
        let parsed = parse(metainfo.original_bytes())?;
        let root = owned(&parsed)?;
        let OwnedValue::Dictionary(root) = root else {
            unreachable!("validated metainfo roots are dictionaries");
        };
        Ok(Self { root })
    }

    /// Starts from an owned validated info dictionary.
    ///
    /// # Errors
    ///
    /// Returns an error when `info` is not a dictionary.
    pub fn from_info(info: OwnedValue) -> Result<Self> {
        if !matches!(info, OwnedValue::Dictionary(_)) {
            return Err(Error::metainfo_field("info", "must be a dictionary"));
        }
        Ok(Self {
            root: BTreeMap::from([(b"info".to_vec(), info)]),
        })
    }

    /// Replaces tracker tiers.
    #[must_use]
    pub fn trackers(mut self, tiers: impl IntoIterator<Item = Vec<Vec<u8>>>) -> Self {
        let tiers = tiers.into_iter().collect::<Vec<_>>();
        self.root.remove(b"announce".as_slice());
        self.root.remove(b"announce-list".as_slice());
        if let Some(announce) = tiers.first().and_then(|tier| tier.first()) {
            self.root
                .insert(b"announce".to_vec(), OwnedValue::bytes(announce.clone()));
        }
        if !tiers.is_empty() {
            self.root.insert(
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
            &mut self.root,
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
            &mut self.root,
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
        set_optional(&mut self.root, b"comment", value.map(OwnedValue::bytes));
        self
    }

    /// Sets or removes the top-level creator.
    #[must_use]
    pub fn created_by(mut self, value: Option<Vec<u8>>) -> Self {
        set_optional(&mut self.root, b"created by", value.map(OwnedValue::bytes));
        self
    }

    /// Sets or removes the top-level creation timestamp.
    #[must_use]
    pub fn creation_date(mut self, value: Option<i64>) -> Self {
        set_optional(
            &mut self.root,
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
        self.root.insert(key, value);
        Ok(self)
    }

    /// Sets file attributes for a validated v1 or v2 path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is absent or has an unsupported shape.
    pub fn file_attributes(mut self, path: &[Vec<u8>], attributes: Vec<u8>) -> Result<Self> {
        let info = self.info_mut()?;
        if let Some(files) = info.get_mut(b"files".as_slice()) {
            let OwnedValue::List(files) = files else {
                return Err(Error::metainfo_field("info.files", "must be a list"));
            };
            for file in files {
                let OwnedValue::Dictionary(fields) = file else {
                    continue;
                };
                if path_matches(fields.get(b"path".as_slice()), path) {
                    set_optional(
                        fields,
                        b"attr",
                        (!attributes.is_empty()).then(|| OwnedValue::bytes(attributes)),
                    );
                    return Ok(self);
                }
            }
        } else if path.len() == 1
            && info
                .get(b"name".as_slice())
                .is_some_and(|name| matches!(name, OwnedValue::Bytes(bytes) if bytes == &path[0]))
            && info.contains_key(b"length".as_slice())
        {
            set_optional(
                info,
                b"attr",
                (!attributes.is_empty()).then(|| OwnedValue::bytes(attributes)),
            );
            return Ok(self);
        }
        if let Some(tree) = info.get_mut(b"file tree".as_slice()) {
            let mut node = tree;
            for component in path {
                let OwnedValue::Dictionary(entries) = node else {
                    return Err(Error::metainfo_field(
                        "info.file tree",
                        "invalid path shape",
                    ));
                };
                node = entries
                    .get_mut(component.as_slice())
                    .ok_or_else(|| Error::metainfo_field("info.file tree", "path not found"))?;
            }
            let OwnedValue::Dictionary(entries) = node else {
                return Err(Error::metainfo_field("info.file tree", "invalid leaf"));
            };
            let properties = entries
                .get_mut(b"".as_slice())
                .ok_or_else(|| Error::metainfo_field("info.file tree", "leaf marker missing"))?;
            let OwnedValue::Dictionary(properties) = properties else {
                return Err(Error::metainfo_field(
                    "info.file tree",
                    "invalid properties",
                ));
            };
            set_optional(
                properties,
                b"attr",
                (!attributes.is_empty()).then(|| OwnedValue::bytes(attributes)),
            );
            return Ok(self);
        }
        Err(Error::metainfo_field("file path", "path not found"))
    }

    /// Serializes canonically and validates the edited result.
    ///
    /// # Errors
    ///
    /// Returns serialization or protocol validation errors.
    pub fn to_metainfo(self) -> Result<Metainfo> {
        let bytes = OwnedValue::Dictionary(self.root).to_vec()?;
        Metainfo::from_bytes(&bytes)
    }

    fn info_mut(&mut self) -> Result<&mut BTreeMap<Vec<u8>, OwnedValue>> {
        let info = self
            .root
            .get_mut(b"info".as_slice())
            .ok_or_else(|| Error::metainfo_field("info", "missing required field"))?;
        let OwnedValue::Dictionary(info) = info else {
            return Err(Error::metainfo_field("info", "must be a dictionary"));
        };
        Ok(info)
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
