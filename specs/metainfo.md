---
spec_id: META
title: "BitTorrent Metainfo"
status: Accepted
owners:
  - "protocol maintainers"
source_paths:
  - "crates/btpc-core/src/metainfo/mod.rs"
test_paths:
  - "crates/btpc-core/tests/raw_metainfo.rs"
  - "crates/btpc-core/tests/v1_metainfo.rs"
last_reviewed: "2026-07-01"
---

# BitTorrent Metainfo

## Requirements

### META-RAW-001 — Preserve original metainfo and info bytes

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/metainfo/mod.rs`
- **Verification:** `crates/btpc-core/tests/raw_metainfo.rs`
- **Depends on:** `BENC-BYTES-001`

Raw metainfo parsing **MUST** require a top-level dictionary with exactly one
dictionary-valued `info` key and expose both the original bytes and exact original
`info` bytes/span.

### META-FIELD-001 — Preserve byte-safe top-level fields

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/metainfo/mod.rs`
- **Verification:** `crates/btpc-core/tests/raw_metainfo.rs`
- **Depends on:** `META-RAW-001`

Common top-level fields **MUST** remain byte-safe, and unknown top-level fields
**MUST** remain accessible rather than being silently discarded.

### META-HASH-001 — Hash the exact original info encoding

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/metainfo/mod.rs`
- **Verification:** `crates/btpc-core/tests/raw_metainfo.rs`
- **Depends on:** `META-RAW-001`

Parsed metainfo SHA-1 and SHA-256 info digests **MUST** be calculated over the exact
original bencoded `info` slice, not a re-encoded representation.

### META-V1-001 — Validate v1 file and piece invariants

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/metainfo/mod.rs`
- **Verification:** `crates/btpc-core/tests/v1_metainfo.rs`
- **Depends on:** `META-RAW-001`

V1 validation **MUST** enforce the single-file versus multi-file shape, non-negative
checked lengths, safe non-empty path components, positive piece length, 20-byte
piece digest width, and the expected piece count including zero-length payloads.

### META-V2-001 — Validate BEP 52 file trees and piece layers

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/metainfo/mod.rs`
- **Verification:** `crates/btpc-core/tests/v2_metainfo.rs`, `crates/btpc-core/tests/v2_merkle.rs`
- **Depends on:** `META-RAW-001`

V2 validation **MUST** follow BEP 52 for `meta version`, file-tree leaves, pieces
roots, piece layers, empty files, and Merkle padding. Each non-empty leaf is the
SHA-256 digest of at most 16 KiB of actual file bytes; the final short block is
**NOT** padded with bytes before hashing. Incomplete binary trees **MUST** be padded
with the recursively derived SHA-256 hashes of empty subtrees at the corresponding
layer, beginning with the all-zero 32-byte leaf hash. Empty files **MUST NOT** have
a `pieces root`. Files no larger than one piece **MUST NOT** have a `piece layers`
entry. For larger files, the layer value **MUST** contain the ordered 32-byte piece
roots and have exactly `ceil(file length / piece length) * 32` bytes. The shared
sequential Merkle oracle defines the creation, validation, and verification
primitive.

### META-HYBRID-001 — Require equivalent hybrid payloads

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/metainfo/mod.rs`
- **Verification:** `crates/btpc-core/tests/v1_metainfo.rs`
- **Depends on:** `META-V1-001`, `META-V2-001`

A hybrid torrent **MUST** contain valid v1 and v2 representations of the same real
files and **MUST** validate permitted v1 padding files separately from v2 files.
Multi-file v1 paths **MUST** form an unambiguous byte-oriented graph: duplicate
paths, file/directory prefixes, and collisions under the target platform mapping
are invalid. Hybrid padding **MUST** be explicitly marked with `p`, use the
reserved `.pad/<length>` or `.pad/<offset>-<length>` form, appear only between
real files, and exactly fill the required alignment gap. Typed inspection
**MUST** expose validated padding entries rather than silently omitting them.

## Design Rationale

Raw parsing and typed validation are distinct states. Bytes are identity; decoded
text is an optional view. Canonicality is reported independently for parseable
source bytes; protocol-invalid input never constructs `Metainfo`. Non-fatal
compatibility warnings are structured with optional field and offset context,
while the legacy message list remains available to adapters. This avoids changing
hashes or paths through decoding and prevents callers from confusing parseability,
canonical encoding, and protocol validity.
