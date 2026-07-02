use std::io::{self, Write};

use btpc_core::ErrorCategory;
use btpc_core::bencode::{OwnedValue, parse, validate_canonical};
use proptest::prelude::*;

// Spec: BENC-CANON-001
#[test]
fn distinguishes_parseable_from_canonical_forms_with_offsets() {
    for (input, offset) in [
        (&b"i03e"[..], 1),
        (b"i-0e", 1),
        (b"i00e", 1),
        (b"03:abc", 0),
        (b"d1:bi1e1:ai2ee", 7),
        (b"d1:ai1e1:ai2ee", 7),
    ] {
        assert!(parse(input).is_ok());
        let error = validate_canonical(input).unwrap_err();
        assert_eq!(error.category(), ErrorCategory::BencodeCanonical);
        assert_eq!(error.offset(), Some(offset));
    }

    for input in [&b"i0e"[..], b"i-1e", b"0:", b"d1:ai1e1:bi2ee"] {
        validate_canonical(input).unwrap();
    }
}

// Spec: BENC-CANON-001
// Spec: BENC-ENC-001
#[test]
fn dictionary_order_uses_unsigned_raw_bytes() {
    validate_canonical(b"d1:\x7fi1e1:\x80i2ee").unwrap();
    assert_eq!(
        validate_canonical(b"d1:\x80i2e1:\x7fi1ee")
            .unwrap_err()
            .offset(),
        Some(7)
    );
}

// Spec: BENC-ENC-001
#[test]
fn owned_model_rejects_duplicate_keys_and_encodes_golden_bytes() {
    let duplicate = OwnedValue::dictionary([
        (b"key".to_vec(), OwnedValue::integer(1)),
        (b"key".to_vec(), OwnedValue::integer(2)),
    ])
    .unwrap_err();
    assert_eq!(duplicate.category(), ErrorCategory::BencodeCanonical);

    let value = OwnedValue::dictionary([
        (b"z".to_vec(), OwnedValue::bytes(b"last".to_vec())),
        (
            b"a".to_vec(),
            OwnedValue::list([OwnedValue::integer(-2), OwnedValue::bytes(Vec::new())]),
        ),
    ])
    .unwrap();
    let expected = b"d1:ali-2e0:e1:z4:laste";
    assert_eq!(value.encoded_len(), expected.len());
    assert_eq!(value.to_vec().unwrap(), expected);
    validate_canonical(&value.to_vec().unwrap()).unwrap();
}

#[test]
fn arbitrary_precision_owned_integers_round_trip_canonically() {
    let positive = OwnedValue::integer_bytes(b"123456789012345678901234567890").unwrap();
    let negative = OwnedValue::integer_bytes(b"-123456789012345678901234567890").unwrap();
    assert_eq!(
        positive.to_vec().unwrap(),
        b"i123456789012345678901234567890e"
    );
    assert_eq!(
        negative.to_vec().unwrap(),
        b"i-123456789012345678901234567890e"
    );
    assert!(OwnedValue::integer_bytes(b"01").is_err());
    assert!(OwnedValue::integer_bytes(b"-0").is_err());
    assert!(OwnedValue::integer_bytes(b"-").is_err());
    assert!(OwnedValue::IntegerBytes(b"01".to_vec()).to_vec().is_err());
}

#[derive(Default)]
struct FailingWriter {
    remaining: usize,
}

struct InterruptedWriter(bool);

impl Write for InterruptedWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        if !self.0 {
            self.0 = true;
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "retryable interruption",
            ));
        }
        Err(io::Error::other("failure after retry"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct ZeroWriter;

impl Write for ZeroWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Write for FailingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::other("intentional writer failure"));
        }
        let written = buffer.len().min(self.remaining);
        self.remaining -= written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// Spec: BENC-ENC-001
#[test]
fn writer_failures_are_preserved_as_sources() {
    let value = OwnedValue::bytes(vec![0; 16]);
    let error = value
        .write_to(&mut FailingWriter { remaining: 2 })
        .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Io);
    assert_eq!(
        std::error::Error::source(&error).map(ToString::to_string),
        Some("intentional writer failure".to_owned())
    );
}

#[test]
fn interrupted_and_zero_writes_never_report_success() {
    let value = OwnedValue::bytes(vec![7; 16]);
    for (mut writer, expected) in [
        (
            Box::new(InterruptedWriter(false)) as Box<dyn Write>,
            io::ErrorKind::Other,
        ),
        (
            Box::new(ZeroWriter) as Box<dyn Write>,
            io::ErrorKind::WriteZero,
        ),
    ] {
        let error = value.write_to(&mut writer).unwrap_err();
        assert_eq!(error.category(), ErrorCategory::Io);
        assert_eq!(
            std::error::Error::source(&error)
                .and_then(|source| source.downcast_ref::<io::Error>())
                .map(io::Error::kind),
            Some(expected)
        );
    }
}

fn arb_value() -> impl Strategy<Value = OwnedValue> {
    let leaf = prop_oneof![
        any::<i64>().prop_map(OwnedValue::integer),
        proptest::collection::vec(any::<u8>(), 0..32).prop_map(OwnedValue::bytes),
    ];
    leaf.prop_recursive(3, 64, 8, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 0..8).prop_map(OwnedValue::list),
            proptest::collection::btree_map(
                proptest::collection::vec(any::<u8>(), 0..12),
                inner,
                0..8,
            )
            .prop_map(|entries| OwnedValue::dictionary(entries).unwrap()),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    // Spec: BENC-ENC-001
    #[test]
    fn owned_encode_parse_round_trip_is_canonical(value in arb_value()) {
        let encoded = value.to_vec().unwrap();
        prop_assert_eq!(encoded.len(), value.encoded_len());
        validate_canonical(&encoded).unwrap();
        prop_assert_eq!(parse(&encoded).unwrap().span().range(), 0..encoded.len());
    }
}
