---
spec_id: RUSTAPI
title: "Rust Public API"
status: Draft
owners:
  - "API maintainers"
source_paths:
  - "crates/btpc-core/src/lib.rs"
  - "docs/rust/index.md"
  - "crates/btpc-core/src/error.rs"
test_paths:
  - "crates/btpc-core/tests"
last_reviewed: "2026-07-01"
---

# Rust Public API

## Requirements

### RUSTAPI-FACADE-001 — Expose a deliberate stable facade

- **Status:** Draft
- **Sources:** `crates/btpc-core/src/lib.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `ARCH-BOUND-001`

The crate root **MUST** expose only deliberately supported types and operations.
Low-level parser internals **MUST NOT** become semver commitments accidentally.

### RUSTAPI-BYTES-001 — Preserve byte identity in public types

- **Status:** Draft
- **Sources:** `crates/btpc-core/src/lib.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `BENC-BYTES-001`, `META-FIELD-001`

Public metainfo path and text types **MUST** preserve raw bytes as identity and
**MAY** expose validated UTF-8 views without lossy replacement. `TorrentBytes`
and `TorrentPath` equality, hashing, and ordering use raw bytes. Platform-path
conversion **MUST** be lossless: Unix preserves component bytes; Windows and other
Unicode-path platforms reject non-UTF-8 torrent components rather than replacing
them.

### RUSTAPI-COMPAT-001 — Verify feature and semver compatibility

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/lib.rs`
- **Verification:** `Cargo.toml`
- **Depends on:** `RUSTAPI-FACADE-001`

Public items **MUST** be documented. Supported feature combinations, MSRV, and
semver changes **MUST** be tested from an external consumer's perspective. The
explicit feature set is currently empty; default, no-default-feature, and
all-feature builds **MUST** remain equivalent until an additive feature has a
documented dependency or platform benefit. Pull requests **MUST** run public API
diffing against their base revision, and release candidates **MUST** compare
against the latest accepted release baseline.

### RUSTAPI-DOC-001 — Keep guide examples executable

- **Status:** Implemented
- **Sources:** `README.md`, `docs/rust/index.md`
- **Verification:** `crates/btpc-core/tests/documentation.rs`, Rust doctests
- **Depends on:** `RUSTAPI-FACADE-001`

The documented parse, create, magnet, progress/cancellation, error, and verify
surfaces **MUST** match the public API. The end-to-end create/parse/magnet/verify
example **MUST** execute against the sequential hybrid correctness path.

### RUSTAPI-METADATA-TYPES-001 — Use semantic byte-oriented metadata types internally

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/create/mod.rs`, `crates/btpc-core/src/metainfo/mod.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `RUSTAPI-BYTES-001`, `BENC-BYTES-001`

Core implementation boundaries **SHOULD** use lightweight semantic types for
tracker URLs and tiers, web seeds, DHT node hosts, and other easily confused
byte-oriented metadata instead of passing unrelated nested `Vec<u8>` shapes
through multiple subsystems. These types **MUST** preserve arbitrary bytes,
deterministic ordering, and canonical serialization, and **MUST NOT** force UTF-8
onto the Rust protocol API. Public exposure requires a separate compatibility
decision; internal newtypes **SHOULD** remain crate-private until then.

## Design Rationale

The facade remains Draft until ownership, mutation states, feature flags, and
byte-safe public path types are resolved. This prevents premature API lock-in.
