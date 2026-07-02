use btpc_core::bencode::{Value, ValueKind, parse, parse_with_limits};
use btpc_core::{ErrorCategory, ParseLimits};
use proptest::prelude::*;

// Spec: BENC-PARSE-001
// Spec: BENC-BYTES-001
#[test]
fn parses_scalars_and_containers_with_exact_spans() {
    let integer = parse(b"i-42e").unwrap();
    assert_eq!(integer.span().range(), 0..5);
    assert_eq!(integer.as_integer(), Some(-42));

    let bytes = parse(b"4:spam").unwrap();
    assert_eq!(bytes.span().range(), 0..6);
    assert_eq!(bytes.as_bytes(), Some(&b"spam"[..]));

    let list = parse(b"li1e3:abce").unwrap();
    let ValueKind::List(items) = list.kind() else {
        panic!("expected list");
    };
    assert_eq!(list.span().range(), 0..10);
    assert_eq!(items[0].span().range(), 1..4);
    assert_eq!(items[1].span().range(), 4..9);

    let dictionary = parse(b"d3:foo3:bar4:listli2eee").unwrap();
    let ValueKind::Dictionary(entries) = dictionary.kind() else {
        panic!("expected dictionary");
    };
    assert_eq!(dictionary.span().range(), 0..23);
    assert_eq!(entries[0].0.span().range(), 1..6);
    assert_eq!(entries[0].1.span().range(), 6..11);
    assert_eq!(
        dictionary.get(b"foo").and_then(Value::as_bytes),
        Some(&b"bar"[..])
    );
    assert!(dictionary.get(b"missing").is_none());
}

// Spec: BENC-BYTES-001
#[test]
fn preserves_arbitrary_bytes_in_keys_and_values() {
    let value = parse(b"d1:\xff2:\x00\xfee").unwrap();
    assert_eq!(
        value.get(&[0xff]).and_then(Value::as_bytes),
        Some(&[0x00, 0xfe][..])
    );
}

// Spec: BENC-CANON-001
#[test]
fn accepts_syntax_that_canonical_validation_will_reject_later() {
    assert_eq!(parse(b"i03e").unwrap().as_integer(), Some(3));
    assert_eq!(parse(b"i-0e").unwrap().as_integer(), Some(0));
    assert_eq!(parse(b"03:abc").unwrap().as_bytes(), Some(&b"abc"[..]));
}

// Spec: BENC-PARSE-001
#[test]
fn rejects_every_malformed_form_with_precise_category() {
    for input in [
        &b""[..],
        b"e",
        b"i",
        b"ie",
        b"i-e",
        b"i+1e",
        b"i1",
        b"i1xe",
        b"1",
        b":",
        b"x:value",
        b"4:abc",
        b"l",
        b"li1e",
        b"d",
        b"d1:ae",
        b"di1ei2ee",
    ] {
        let error = parse(input).unwrap_err();
        assert_eq!(error.category(), ErrorCategory::BencodeSyntax, "{input:?}");
        assert!(error.offset().is_some());
    }
}

// Spec: BENC-PARSE-001
#[test]
fn rejects_trailing_input_and_integer_or_length_overflow() {
    for input in [
        &b"i1ei2e"[..],
        b"9999999999999999999999999999999999999999:x",
    ] {
        assert_eq!(
            parse(input).unwrap_err().category(),
            ErrorCategory::BencodeSyntax
        );
    }
}

#[test]
fn preserves_integer_digits_beyond_machine_ranges() {
    for (input, i64_value, u64_value) in [
        (
            &b"i9223372036854775807e"[..],
            Some(i64::MAX),
            Some(i64::MAX as u64),
        ),
        (
            &b"i9223372036854775808e"[..],
            None,
            Some(9_223_372_036_854_775_808),
        ),
        (&b"i18446744073709551615e"[..], None, Some(u64::MAX)),
        (&b"i18446744073709551616e"[..], None, None),
        (&b"i-9223372036854775808e"[..], Some(i64::MIN), None),
        (&b"i-9223372036854775809e"[..], None, None),
        (
            &b"i1234567890123456789012345678901234567890e"[..],
            None,
            None,
        ),
    ] {
        let parsed = parse(input).unwrap();
        let integer = parsed.integer().unwrap();
        assert_eq!(integer.encoded(), &input[1..input.len() - 1]);
        assert_eq!(integer.to_i64(), i64_value);
        assert_eq!(integer.to_u64(), u64_value);
    }
}

#[test]
fn integer_digit_limit_is_enforced_before_conversion() {
    let limits = ParseLimits::default().with_max_integer_digits(3);
    assert!(parse_with_limits(b"i999e", limits).is_ok());
    let error = parse_with_limits(b"i1000e", limits).unwrap_err();
    assert_eq!(error.limit(), Some("integer digits"));
}

// Spec: BENC-LIMIT-001
// Spec: SEC-PARSE-001
#[test]
fn enforces_input_depth_item_and_byte_string_limits_at_boundaries() {
    let depth_limits = ParseLimits::new(2, 100, 100, 100, usize::MAX);
    assert!(parse_with_limits(b"lli1eee", depth_limits).is_ok());
    assert_eq!(
        parse_with_limits(b"llli1eeee", depth_limits)
            .unwrap_err()
            .limit(),
        Some("depth")
    );

    let item_limits = ParseLimits::new(100, 3, 100, 100, usize::MAX);
    assert!(parse_with_limits(b"li1ei2ee", item_limits).is_ok());
    assert_eq!(
        parse_with_limits(b"li1ei2ei3ee", item_limits)
            .unwrap_err()
            .limit(),
        Some("item count")
    );

    let byte_limits = ParseLimits::new(100, 100, 3, 100, usize::MAX);
    assert!(parse_with_limits(b"3:abc", byte_limits).is_ok());
    assert_eq!(
        parse_with_limits(b"4:abcd", byte_limits)
            .unwrap_err()
            .limit(),
        Some("byte-string length")
    );

    let input_limits = ParseLimits::new(100, 100, 100, 8, usize::MAX);
    assert!(parse_with_limits(b"li1234ee", input_limits).is_ok());
    assert_eq!(
        parse_with_limits(b"li12345ee", input_limits)
            .unwrap_err()
            .limit(),
        Some("total input")
    );
}

// Spec: BENC-LIMIT-001
// Spec: SEC-PARSE-001
#[test]
fn resource_limits_cover_zero_exact_and_one_over_edges() {
    let scalar_only = ParseLimits::new(0, 1, 0, 3, 0);
    assert!(parse_with_limits(b"i0e", scalar_only).is_ok());
    assert_eq!(
        parse_with_limits(b"le", scalar_only).unwrap_err().limit(),
        Some("depth")
    );

    let items = ParseLimits::new(1, 2, 8, 8, usize::MAX);
    assert!(parse_with_limits(b"li0ee", items).is_ok());
    assert_eq!(
        parse_with_limits(b"li0ei1ee", ParseLimits::new(1, 2, 8, 9, usize::MAX))
            .unwrap_err()
            .limit(),
        Some("item count")
    );

    let bytes = ParseLimits::new(1, 2, 0, 2, usize::MAX);
    assert!(parse_with_limits(b"0:", bytes).is_ok());
    assert_eq!(
        parse_with_limits(b"1:x", ParseLimits::new(1, 2, 0, 3, usize::MAX))
            .unwrap_err()
            .limit(),
        Some("byte-string length")
    );

    assert!(parse_with_limits(b"i0e", ParseLimits::new(1, 1, 8, 3, usize::MAX)).is_ok());
    assert_eq!(
        parse_with_limits(b"i0e", ParseLimits::new(1, 1, 8, 2, usize::MAX))
            .unwrap_err()
            .limit(),
        Some("total input")
    );

    let allocation = ParseLimits::new(0, 0, 0, 0, 3);
    assert_eq!(allocation.checked_owned_allocation(1, 2).unwrap(), 3);
    assert_eq!(
        allocation
            .checked_owned_allocation(2, 2)
            .unwrap_err()
            .limit(),
        Some("owned allocation")
    );
}

#[test]
fn parser_container_storage_obeys_owned_allocation_budget() {
    let element_size = std::mem::size_of::<Value<'static>>();
    assert!(parse_with_limits(b"li0ee", ParseLimits::new(1, 2, 8, 8, element_size)).is_ok());
    let error =
        parse_with_limits(b"li0ee", ParseLimits::new(1, 2, 8, 8, element_size - 1)).unwrap_err();
    assert_eq!(error.limit(), Some("owned allocation"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn long_integer_digits_round_trip_without_numeric_conversion(
        negative in any::<bool>(),
        first in b'1'..=b'9',
        tail in proptest::collection::vec(b'0'..=b'9', 64..512),
    ) {
        let mut encoded = Vec::with_capacity(tail.len() + 3);
        encoded.push(b'i');
        if negative {
            encoded.push(b'-');
        }
        encoded.push(first);
        encoded.extend(tail);
        encoded.push(b'e');
        let parsed = parse(&encoded).unwrap();
        let integer = parsed.integer().unwrap();
        prop_assert_eq!(integer.encoded(), &encoded[1..encoded.len() - 1]);
        prop_assert_eq!(integer.to_i64(), None);
        if negative {
            prop_assert_eq!(integer.to_u64(), None);
        }
    }
}
