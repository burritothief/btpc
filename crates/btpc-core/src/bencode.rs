//! Borrowed bencode syntax tree with exact source spans.

use std::ops::Range;

use crate::limits::AllocationBudget;
use crate::{Error, ParseLimits, Result};

/// Half-open byte offsets into the original bencoded input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Returns the start byte offset.
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the exclusive end byte offset.
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns the span as a standard half-open range.
    #[must_use]
    pub const fn range(self) -> Range<usize> {
        self.start..self.end
    }
}

/// A borrowed bencode byte string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteString<'a> {
    bytes: &'a [u8],
    span: Span,
}

impl<'a> ByteString<'a> {
    /// Returns the raw byte contents without UTF-8 decoding.
    #[must_use]
    pub const fn bytes(self) -> &'a [u8] {
        self.bytes
    }

    /// Returns the exact encoded source span, including length prefix.
    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }
}

/// The payload of a borrowed bencode value.
#[derive(Debug)]
pub enum ValueKind<'a> {
    /// Lossless signed bencode integer text.
    Integer(Integer<'a>),
    /// Raw byte string.
    Bytes(&'a [u8]),
    /// Ordered list of values.
    List(Vec<Value<'a>>),
    /// Ordered dictionary entries with raw byte-string keys.
    Dictionary(Vec<(ByteString<'a>, Value<'a>)>),
}

/// Borrowed, lossless bencode integer representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Integer<'a> {
    encoded: &'a [u8],
}

impl<'a> Integer<'a> {
    /// Returns the signed decimal bytes between the `i` and `e` delimiters.
    #[must_use]
    pub const fn encoded(self) -> &'a [u8] {
        self.encoded
    }

    /// Converts to `i64` when representable.
    #[must_use]
    pub fn to_i64(self) -> Option<i64> {
        std::str::from_utf8(self.encoded).ok()?.parse().ok()
    }

    /// Converts to `u64` when non-negative and representable.
    #[must_use]
    pub fn to_u64(self) -> Option<u64> {
        std::str::from_utf8(self.encoded).ok()?.parse().ok()
    }

    pub(crate) fn canonical_bytes(self) -> Vec<u8> {
        let (negative, digits) = self
            .encoded
            .strip_prefix(b"-")
            .map_or((false, self.encoded), |digits| (true, digits));
        let significant = digits
            .iter()
            .position(|digit| *digit != b'0')
            .map_or(&digits[digits.len()..], |start| &digits[start..]);
        if significant.is_empty() {
            return b"0".to_vec();
        }
        let mut output = Vec::with_capacity(significant.len() + usize::from(negative));
        if negative {
            output.push(b'-');
        }
        output.extend_from_slice(significant);
        output
    }
}

/// A parsed bencode value borrowing byte strings from its input.
#[derive(Debug)]
pub struct Value<'a> {
    kind: ValueKind<'a>,
    span: Span,
}

impl<'a> Value<'a> {
    /// Returns this value's payload.
    #[must_use]
    pub const fn kind(&self) -> &ValueKind<'a> {
        &self.kind
    }

    /// Returns the exact encoded source span.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the integer payload when this is an integer.
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self.kind {
            ValueKind::Integer(value) => value.to_i64(),
            _ => None,
        }
    }

    /// Returns the lossless integer payload when this is an integer.
    #[must_use]
    pub const fn integer(&self) -> Option<Integer<'a>> {
        match self.kind {
            ValueKind::Integer(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the raw bytes when this is a byte string.
    #[must_use]
    pub const fn as_bytes(&self) -> Option<&'a [u8]> {
        match self.kind {
            ValueKind::Bytes(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Looks up a raw byte key without allocation.
    #[must_use]
    pub fn get(&self, key: &[u8]) -> Option<&Value<'a>> {
        let ValueKind::Dictionary(entries) = &self.kind else {
            return None;
        };
        entries
            .iter()
            .find_map(|(candidate, value)| (candidate.bytes == key).then_some(value))
    }
}

/// Parses exactly one bencode value using default resource limits.
///
/// # Errors
///
/// Returns a syntax or resource-limit error for malformed, trailing, or
/// over-limit input.
pub fn parse(input: &[u8]) -> Result<Value<'_>> {
    parse_with_limits(input, ParseLimits::default())
}

/// Parses exactly one bencode value using caller-provided resource limits.
///
/// # Errors
///
/// Returns a syntax or resource-limit error for malformed, trailing, or
/// over-limit input.
pub fn parse_with_limits(input: &[u8], limits: ParseLimits) -> Result<Value<'_>> {
    let mut budget = AllocationBudget::new(limits);
    parse_with_budget(input, limits, &mut budget)
}

pub(crate) fn parse_with_budget<'input>(
    input: &'input [u8],
    limits: ParseLimits,
    budget: &mut AllocationBudget,
) -> Result<Value<'input>> {
    limits.check_total_input(input.len())?;
    let mut parser = Parser {
        input,
        position: 0,
        items: 0,
        limits,
        budget,
    };
    let value = parser.parse_value(0)?;
    if parser.position != input.len() {
        return Err(Error::bencode_syntax(
            parser.position,
            "trailing bytes after top-level value",
        ));
    }
    Ok(value)
}

struct Parser<'input, 'budget> {
    input: &'input [u8],
    position: usize,
    items: usize,
    limits: ParseLimits,
    budget: &'budget mut AllocationBudget,
}

impl<'input> Parser<'input, '_> {
    fn parse_value(&mut self, depth: usize) -> Result<Value<'input>> {
        self.count_item()?;
        match self.peek()? {
            b'i' => self.parse_integer(),
            b'l' => self.parse_list(depth),
            b'd' => self.parse_dictionary(depth),
            b'0'..=b'9' => self.parse_byte_value(),
            byte => Err(Error::bencode_syntax(
                self.position,
                format!("unexpected byte 0x{byte:02x}"),
            )),
        }
    }

    fn parse_integer(&mut self) -> Result<Value<'input>> {
        let start = self.position;
        self.position += 1;
        let number_start = self.position;
        while let Some(byte) = self.input.get(self.position) {
            if *byte == b'e' {
                break;
            }
            self.position += 1;
        }
        if self.input.get(self.position) != Some(&b'e') {
            return Err(Error::bencode_syntax(start, "unterminated integer"));
        }
        let encoded = &self.input[number_start..self.position];
        if encoded.is_empty() {
            return Err(Error::bencode_syntax(number_start, "empty integer"));
        }
        if encoded[0] == b'+' {
            return Err(Error::bencode_syntax(
                number_start,
                "plus sign is not allowed",
            ));
        }
        let digits = encoded.strip_prefix(b"-").unwrap_or(encoded);
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
            return Err(Error::bencode_syntax(
                number_start,
                "invalid integer digits",
            ));
        }
        self.limits.check_integer_digits(digits.len())?;
        self.position += 1;
        Ok(Value {
            kind: ValueKind::Integer(Integer { encoded }),
            span: Span::new(start, self.position),
        })
    }

    fn parse_byte_value(&mut self) -> Result<Value<'input>> {
        let byte_string = self.parse_byte_string(false)?;
        Ok(Value {
            kind: ValueKind::Bytes(byte_string.bytes),
            span: byte_string.span,
        })
    }

    fn parse_byte_string(&mut self, count_item: bool) -> Result<ByteString<'input>> {
        if count_item {
            self.count_item()?;
        }
        let start = self.position;
        let length_start = self.position;
        while matches!(self.input.get(self.position), Some(b'0'..=b'9')) {
            self.position += 1;
        }
        if self.position == length_start || self.input.get(self.position) != Some(&b':') {
            return Err(Error::bencode_syntax(start, "invalid byte-string length"));
        }
        let length_text = std::str::from_utf8(&self.input[length_start..self.position])
            .map_err(|_| Error::bencode_syntax(start, "invalid byte-string length"))?;
        let length = length_text
            .parse::<usize>()
            .map_err(|_| Error::bencode_syntax(start, "byte-string length is out of range"))?;
        self.limits.check_byte_string_length(length)?;
        self.position += 1;
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| Error::bencode_syntax(start, "byte-string end offset overflowed"))?;
        let Some(bytes) = self.input.get(self.position..end) else {
            return Err(Error::bencode_syntax(start, "truncated byte string"));
        };
        self.position = end;
        Ok(ByteString {
            bytes,
            span: Span::new(start, end),
        })
    }

    fn parse_list(&mut self, depth: usize) -> Result<Value<'input>> {
        let container_depth = depth
            .checked_add(1)
            .ok_or_else(|| Error::resource_limit("depth", usize::MAX, self.limits.max_depth()))?;
        self.limits.check_depth(container_depth)?;
        let start = self.position;
        self.position += 1;
        let mut items = Vec::new();
        loop {
            match self.input.get(self.position) {
                Some(b'e') => {
                    self.position += 1;
                    break;
                }
                Some(_) => {
                    self.budget.charge(std::mem::size_of::<Value<'input>>())?;
                    items.try_reserve_exact(1).map_err(|_| {
                        Error::resource_limit(
                            "owned allocation",
                            usize::MAX,
                            self.limits.max_owned_allocation(),
                        )
                    })?;
                    items.push(self.parse_value(container_depth)?);
                }
                None => return Err(Error::bencode_syntax(start, "unterminated list")),
            }
        }
        Ok(Value {
            kind: ValueKind::List(items),
            span: Span::new(start, self.position),
        })
    }

    fn parse_dictionary(&mut self, depth: usize) -> Result<Value<'input>> {
        let container_depth = depth
            .checked_add(1)
            .ok_or_else(|| Error::resource_limit("depth", usize::MAX, self.limits.max_depth()))?;
        self.limits.check_depth(container_depth)?;
        let start = self.position;
        self.position += 1;
        let mut entries = Vec::new();
        loop {
            match self.input.get(self.position) {
                Some(b'e') => {
                    self.position += 1;
                    break;
                }
                Some(b'0'..=b'9') => {
                    self.budget
                        .charge(std::mem::size_of::<(ByteString<'input>, Value<'input>)>())?;
                    entries.try_reserve_exact(1).map_err(|_| {
                        Error::resource_limit(
                            "owned allocation",
                            usize::MAX,
                            self.limits.max_owned_allocation(),
                        )
                    })?;
                    let key = self.parse_byte_string(true)?;
                    if self.input.get(self.position).is_none() {
                        return Err(Error::bencode_syntax(start, "dictionary key has no value"));
                    }
                    let value = self.parse_value(container_depth)?;
                    entries.push((key, value));
                }
                Some(_) => {
                    return Err(Error::bencode_syntax(
                        self.position,
                        "dictionary key must be a byte string",
                    ));
                }
                None => return Err(Error::bencode_syntax(start, "unterminated dictionary")),
            }
        }
        Ok(Value {
            kind: ValueKind::Dictionary(entries),
            span: Span::new(start, self.position),
        })
    }

    fn count_item(&mut self) -> Result<()> {
        self.items = self.items.checked_add(1).ok_or_else(|| {
            Error::resource_limit("item count", usize::MAX, self.limits.max_items())
        })?;
        self.limits.check_items(self.items)
    }

    fn peek(&self) -> Result<u8> {
        self.input
            .get(self.position)
            .copied()
            .ok_or_else(|| Error::bencode_syntax(self.position, "expected a value"))
    }
}

/// An owned bencode value that serializes canonically.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnedValue {
    /// Signed bencode integer.
    Integer(i64),
    /// Arbitrary-precision canonical signed decimal bytes.
    IntegerBytes(Vec<u8>),
    /// Raw byte string.
    Bytes(Vec<u8>),
    /// Ordered list of values.
    List(Vec<Self>),
    /// Dictionary stored in canonical raw-byte key order.
    Dictionary(std::collections::BTreeMap<Vec<u8>, Self>),
}

impl OwnedValue {
    /// Creates an integer value.
    #[must_use]
    pub const fn integer(value: i64) -> Self {
        Self::Integer(value)
    }

    /// Creates an arbitrary-precision canonical integer.
    ///
    /// # Errors
    ///
    /// Returns a canonical error for empty digits, non-digits, a plus sign,
    /// leading zeroes, or negative zero.
    pub fn integer_bytes(value: impl AsRef<[u8]>) -> Result<Self> {
        let value = value.as_ref();
        let digits = value.strip_prefix(b"-").unwrap_or(value);
        let valid = !digits.is_empty() && digits.iter().all(u8::is_ascii_digit);
        let canonical_zero = digits == b"0" && !value.starts_with(b"-");
        let canonical_nonzero = digits.first().is_some_and(|digit| *digit != b'0');
        if value.starts_with(b"+") || !valid || !(canonical_zero || canonical_nonzero) {
            return Err(Error::bencode_canonical(0, "invalid canonical integer"));
        }
        Ok(Self::IntegerBytes(value.to_vec()))
    }

    /// Creates a byte-string value.
    #[must_use]
    pub fn bytes(value: impl Into<Vec<u8>>) -> Self {
        Self::Bytes(value.into())
    }

    /// Creates a list value.
    #[must_use]
    pub fn list(values: impl IntoIterator<Item = Self>) -> Self {
        Self::List(values.into_iter().collect())
    }

    /// Creates a dictionary and rejects duplicate raw byte keys.
    ///
    /// # Errors
    ///
    /// Returns a canonical error if the input contains duplicate keys.
    pub fn dictionary(entries: impl IntoIterator<Item = (Vec<u8>, Self)>) -> Result<Self> {
        let mut dictionary = std::collections::BTreeMap::new();
        for (key, value) in entries {
            if dictionary.insert(key, value).is_some() {
                return Err(Error::bencode_canonical(0, "duplicate dictionary key"));
            }
        }
        Ok(Self::Dictionary(dictionary))
    }

    /// Returns the exact canonical encoded length.
    #[must_use]
    pub fn encoded_len(&self) -> usize {
        match self {
            Self::Integer(value) => 2 + decimal_len_i64(*value),
            Self::IntegerBytes(value) => 2 + value.len(),
            Self::Bytes(bytes) => decimal_len_usize(bytes.len()) + 1 + bytes.len(),
            Self::List(values) => 2 + values.iter().map(Self::encoded_len).sum::<usize>(),
            Self::Dictionary(entries) => {
                2 + entries
                    .iter()
                    .map(|(key, value)| {
                        decimal_len_usize(key.len()) + 1 + key.len() + value.encoded_len()
                    })
                    .sum::<usize>()
            }
        }
    }

    /// Writes canonical bencode to a byte writer.
    ///
    /// # Errors
    ///
    /// Returns an I/O error preserving the writer's original error as its source.
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<()> {
        self.validate_integer_bytes()?;
        self.write_canonical(writer)
            .map_err(|source| Error::io("<writer>", source))
    }

    /// Encodes this value into a pre-sized byte vector.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if writing to the vector unexpectedly fails.
    pub fn to_vec(&self) -> Result<Vec<u8>> {
        self.validate_integer_bytes()?;
        let mut encoded = Vec::with_capacity(self.encoded_len());
        self.write_canonical(&mut encoded)
            .map_err(|source| Error::io("<writer>", source))?;
        Ok(encoded)
    }

    fn write_canonical(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::Integer(value) => write!(writer, "i{value}e"),
            Self::IntegerBytes(value) => {
                writer.write_all(b"i")?;
                writer.write_all(value)?;
                writer.write_all(b"e")
            }
            Self::Bytes(bytes) => {
                write!(writer, "{}:", bytes.len())?;
                writer.write_all(bytes)
            }
            Self::List(values) => {
                writer.write_all(b"l")?;
                for value in values {
                    value.write_canonical(writer)?;
                }
                writer.write_all(b"e")
            }
            Self::Dictionary(entries) => {
                writer.write_all(b"d")?;
                for (key, value) in entries {
                    write!(writer, "{}:", key.len())?;
                    writer.write_all(key)?;
                    value.write_canonical(writer)?;
                }
                writer.write_all(b"e")
            }
        }
    }

    fn validate_integer_bytes(&self) -> Result<()> {
        match self {
            Self::IntegerBytes(value) => Self::integer_bytes(value).map(|_| ()),
            Self::List(values) => values.iter().try_for_each(Self::validate_integer_bytes),
            Self::Dictionary(entries) => {
                entries.values().try_for_each(Self::validate_integer_bytes)
            }
            Self::Integer(_) | Self::Bytes(_) => Ok(()),
        }
    }
}

/// Validates that an input is syntactically valid and canonically encoded.
///
/// # Errors
///
/// Returns syntax/resource errors from parsing or canonical errors with precise
/// source offsets for non-minimal integers and lengths, unsorted keys, or
/// duplicate dictionary keys.
pub fn validate_canonical(input: &[u8]) -> Result<()> {
    let value = parse(input)?;
    validate_value_canonical(input, &value)
}

pub(crate) fn validate_parsed_canonical(input: &[u8], value: &Value<'_>) -> Result<()> {
    validate_value_canonical(input, value)
}

fn validate_value_canonical(input: &[u8], value: &Value<'_>) -> Result<()> {
    match value.kind() {
        ValueKind::Integer(_) => validate_integer_encoding(input, value.span()),
        ValueKind::Bytes(_) => validate_byte_string_encoding(input, value.span()),
        ValueKind::List(values) => {
            for value in values {
                validate_value_canonical(input, value)?;
            }
            Ok(())
        }
        ValueKind::Dictionary(entries) => {
            let mut previous: Option<&[u8]> = None;
            for (key, value) in entries {
                validate_byte_string_encoding(input, key.span())?;
                if let Some(previous) = previous {
                    match key.bytes().cmp(previous) {
                        std::cmp::Ordering::Less => {
                            return Err(Error::bencode_canonical(
                                key.span().start(),
                                "dictionary keys are not sorted",
                            ));
                        }
                        std::cmp::Ordering::Equal => {
                            return Err(Error::bencode_canonical(
                                key.span().start(),
                                "duplicate dictionary key",
                            ));
                        }
                        std::cmp::Ordering::Greater => {}
                    }
                }
                previous = Some(key.bytes());
                validate_value_canonical(input, value)?;
            }
            Ok(())
        }
    }
}

fn validate_integer_encoding(input: &[u8], span: Span) -> Result<()> {
    let encoded = &input[span.start() + 1..span.end() - 1];
    if encoded == b"-0" {
        return Err(Error::bencode_canonical(
            span.start() + 1,
            "negative zero is not canonical",
        ));
    }
    let digits = encoded.strip_prefix(b"-").unwrap_or(encoded);
    if digits.len() > 1 && digits[0] == b'0' {
        return Err(Error::bencode_canonical(
            span.start() + 1,
            "integer has a leading zero",
        ));
    }
    Ok(())
}

fn validate_byte_string_encoding(input: &[u8], span: Span) -> Result<()> {
    let encoded = &input[span.start()..span.end()];
    let colon = encoded
        .iter()
        .position(|byte| *byte == b':')
        .expect("parsed byte strings always contain a colon");
    if colon > 1 && encoded[0] == b'0' {
        return Err(Error::bencode_canonical(
            span.start(),
            "byte-string length has a leading zero",
        ));
    }
    Ok(())
}

fn decimal_len_usize(value: usize) -> usize {
    value.checked_ilog10().map_or(1, |log| log as usize + 1)
}

fn decimal_len_i64(value: i64) -> usize {
    if value < 0 {
        1 + decimal_len_u64(value.unsigned_abs())
    } else {
        decimal_len_u64(value.unsigned_abs())
    }
}

fn decimal_len_u64(value: u64) -> usize {
    value.checked_ilog10().map_or(1, |log| log as usize + 1)
}
