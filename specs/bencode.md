---
spec_id: BENC
title: "Bencode Parsing and Encoding"
status: Implemented
owners:
  - "core maintainers"
source_paths:
  - "crates/btpc-core/src/bencode.rs"
  - "crates/btpc-core/src/limits.rs"
test_paths:
  - "crates/btpc-core/tests/bencode_parser.rs"
  - "crates/btpc-core/tests/bencode_canonical.rs"
last_reviewed: "2026-07-01"
---

# Bencode Parsing and Encoding

## Requirements

### BENC-PARSE-001 — Parse one complete bencode value

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/bencode.rs`
- **Verification:** `crates/btpc-core/tests/bencode_parser.rs`
- **Depends on:** None

The syntax parser **MUST** parse integers, byte strings, lists, and dictionaries,
consume exactly one top-level value, and reject malformed or trailing input.
Integer syntax **MUST** preserve arbitrary-length signed decimal digits losslessly;
bounded protocol conversions are a separate validation step.

### BENC-BYTES-001 — Preserve bytes and exact spans

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/bencode.rs`
- **Verification:** `crates/btpc-core/tests/bencode_parser.rs`
- **Depends on:** `BENC-PARSE-001`

Parsed byte strings and dictionary keys **MUST** remain raw bytes. Every parsed
value **MUST** expose its exact half-open source span without UTF-8 conversion.

### BENC-LIMIT-001 — Enforce parser resource limits

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/bencode.rs`, `crates/btpc-core/src/limits.rs`
- **Verification:** `crates/btpc-core/tests/bencode_parser.rs`, `crates/btpc-core/tests/error.rs`
- **Depends on:** `BENC-PARSE-001`

Parsing untrusted input **MUST** enforce configured total-input, depth, item-count,
byte-string-length, integer-digit, parser-container, canonical-buffer, and owned-snapshot limits
using checked arithmetic. Metainfo path loading **MUST** check regular-file length
before allocation and cap the actual read against the same input limit.

### BENC-CANON-001 — Report canonicality independently

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/bencode.rs`
- **Verification:** `crates/btpc-core/tests/bencode_canonical.rs`
- **Depends on:** `BENC-PARSE-001`

Canonical validation **MUST** reject non-minimal integers and byte-string lengths,
unsorted raw-byte dictionary keys, and duplicate keys with a useful byte offset.
Syntax parsing **MUST** remain able to inspect parseable non-canonical input.

### BENC-ENC-001 — Emit deterministic canonical bencode

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/bencode.rs`
- **Verification:** `crates/btpc-core/tests/bencode_canonical.rs`
- **Depends on:** `BENC-CANON-001`

Owned values **MUST** encode canonically to a writer or byte vector. Dictionary
keys **MUST** use unsigned raw-byte lexical order, duplicate keys **MUST** be
rejected, and writer errors **MUST** retain their source.

## Design Rationale

A custom borrowed parser supplies exact `info` spans and byte safety. Syntax and
canonicality are separate because existing torrents may be valid to inspect even
when their original encoding is not canonical.
