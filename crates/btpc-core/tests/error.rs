use std::error::Error as _;
use std::io;
use std::path::Path;

use btpc_core::{Error, ErrorCategory, ParseLimits};

// Spec: ERR-CORE-001
#[test]
fn error_category_and_context_accessors_are_stable() {
    let syntax = Error::bencode_syntax(17, "unexpected delimiter");
    assert_eq!(syntax.category(), ErrorCategory::BencodeSyntax);
    assert_eq!(syntax.offset(), Some(17));
    assert_eq!(syntax.field(), None);
    assert_eq!(syntax.path(), None);

    let canonical = Error::bencode_canonical(4, "non-minimal integer");
    assert_eq!(canonical.category(), ErrorCategory::BencodeCanonical);
    assert_eq!(canonical.offset(), Some(4));

    let metainfo = Error::metainfo_field("info.piece length", "must be positive");
    assert_eq!(metainfo.category(), ErrorCategory::Metainfo);
    assert_eq!(metainfo.field(), Some("info.piece length"));

    let unsupported = Error::unsupported("mutable torrents");
    assert_eq!(unsupported.category(), ErrorCategory::Unsupported);

    let mismatch = Error::verification_mismatch(
        Some(Path::new("payload/file.bin").to_path_buf()),
        "piece 3 differs",
    );
    assert_eq!(mismatch.category(), ErrorCategory::Verification);
    assert_eq!(mismatch.path(), Some(Path::new("payload/file.bin")));

    assert_eq!(Error::cancelled().category(), ErrorCategory::Cancelled);
}

// Spec: ERR-IO-001
#[test]
fn io_error_preserves_path_and_source_chain() {
    let error = Error::io(
        "payload/file.bin",
        io::Error::new(io::ErrorKind::NotFound, "missing fixture"),
    );

    assert_eq!(error.category(), ErrorCategory::Io);
    assert_eq!(error.path(), Some(Path::new("payload/file.bin")));
    assert_eq!(
        error.source().map(ToString::to_string).as_deref(),
        Some("missing fixture")
    );
    assert!(error.to_string().contains("payload/file.bin"));
}

// Spec: BENC-LIMIT-001
#[test]
fn error_parse_limit_defaults_are_conservative_and_accessible() {
    let limits = ParseLimits::default();

    assert_eq!(limits.max_depth(), 128);
    assert_eq!(limits.max_items(), 1_000_000);
    assert_eq!(limits.max_byte_string_length(), 128 * 1024 * 1024);
    assert_eq!(limits.max_integer_digits(), 4_096);
    assert_eq!(limits.max_total_input(), 256 * 1024 * 1024);
    assert_eq!(limits.max_owned_allocation(), 256 * 1024 * 1024);
}

// Spec: BENC-LIMIT-001
// Spec: SEC-PARSE-001
#[test]
fn error_parse_limit_checks_report_boundary_and_overflow_safely() {
    let limits = ParseLimits::new(2, 3, 5, 8, 13);

    assert!(limits.check_depth(2).is_ok());
    assert!(limits.check_items(3).is_ok());
    assert!(limits.check_byte_string_length(5).is_ok());
    assert!(limits.check_total_input(8).is_ok());
    assert_eq!(limits.checked_owned_allocation(8, 5).unwrap(), 13);

    let exceeded = limits.check_depth(3).unwrap_err();
    assert_eq!(exceeded.category(), ErrorCategory::ResourceLimit);
    assert_eq!(exceeded.limit(), Some("depth"));
    assert_eq!(exceeded.actual_and_maximum(), Some((3, 2)));

    let overflow = limits.checked_owned_allocation(usize::MAX, 1).unwrap_err();
    assert_eq!(overflow.category(), ErrorCategory::ResourceLimit);
    assert_eq!(overflow.limit(), Some("owned allocation"));
    assert_eq!(overflow.actual_and_maximum(), Some((usize::MAX, 13)));
}
