---
spec_id: CREATE
title: "Torrent Creation"
status: Accepted
owners:
  - "core maintainers"
source_paths:
  - "crates/btpc-core/src/lib.rs"
test_paths:
  - "crates/btpc-core/tests"
last_reviewed: "2026-07-01"
---

# Torrent Creation

## Requirements

### CREATE-MANIFEST-001 — Build a deterministic safe manifest

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/create/mod.rs`
- **Verification:** `crates/btpc-core/tests/manifest.rs`
- **Depends on:** `META-V1-001`

Creation **MUST** scan payloads into a deterministic torrent-path order and apply
explicit policies for symlinks, hidden files, special files, exclusions, empty
files, unsafe components, and path collisions. A followed top-level symlink
**MUST** retain the selected link name; reject and skip policies **MUST NOT**
silently produce an empty manifest. Text glob filters **MUST** reject non-UTF-8
paths instead of matching lossy substitutions, and the Rust API **MUST** provide
exact raw-byte path filters.

Each file snapshot **MUST** record type, size, available timestamps, and the
strongest stable platform identity exposed by safe standard-library metadata.
Hashing **MUST** validate both the path and opened file handle before reading and
again after reading. If a path or handle no longer matches the snapshot, creation
**MUST** fail rather than combine bytes from ambiguous filesystem states.

### CREATE-V1-001 — Stream canonical v1 creation

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/lib.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `CREATE-MANIFEST-001`, `BENC-ENC-001`, `META-V1-001`

V1 creation **MUST** hash the concatenated logical file stream with SHA-1 pieces,
including cross-file pieces, using bounded memory and deterministic metadata.

### CREATE-PIECE-POLICY-001 — Select stable automatic piece lengths

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/create/mod.rs`
- **Verification:** `crates/btpc-core/tests/piece_length.rs`
- **Depends on:** `CREATE-MANIFEST-001`

Policy `btpc-piece-v1` **MUST** select the following inclusive payload bands:

| Payload bytes at most | Piece length |
| ---: | ---: |
| 16 MiB | 16 KiB |
| 32 MiB | 32 KiB |
| 64 MiB | 64 KiB |
| 128 MiB | 128 KiB |
| 256 MiB | 256 KiB |
| 512 MiB | 512 KiB |
| 1 GiB | 1 MiB |
| 2 GiB | 2 MiB |
| 4 GiB | 4 MiB |
| 8 GiB | 8 MiB |
| 16 GiB and larger | 16 MiB |

Zero-length payloads use 16 KiB. Explicit v1 lengths **MUST** be powers of two
from 1 KiB through 16 MiB; v2 and hybrid lengths **MUST** be powers of two from
the BEP 52 minimum of 16 KiB through 16 MiB. Changing these bands requires a new
policy identifier, compatibility note, and boundary-test update.

### CREATE-V2-001 — Create BEP 52 Merkle metadata

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/create/mod.rs`
- **Verification:** `crates/btpc-core/tests/v2_merkle.rs`, `crates/btpc-core/tests/v2_create.rs`
- **Depends on:** `CREATE-MANIFEST-001`, `META-V2-001`

V2 creation **MUST** construct compliant file trees, pieces roots, and required
piece layers using one verified BEP 52 Merkle primitive. The primitive hashes the
final block at its actual length, pads trees only with the recursively derived
empty-subtree hashes defined by `META-V2-001`, omits a root for an empty file, and
omits piece layers for files no larger than one piece.

### CREATE-HYBRID-001 — Create consistent hybrid metadata

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/create/mod.rs`
- **Verification:** `crates/btpc-core/tests/hybrid_create.rs`
- **Depends on:** `CREATE-V1-001`, `CREATE-V2-001`, `META-HYBRID-001`

Hybrid creation **MUST** emit equivalent v1 and v2 representations, deterministic
v1 padding files where required, and both applicable info hashes.

### CREATE-OUTPUT-001 — Write atomically and reproducibly

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/lib.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `BENC-ENC-001`

File output **MUST** be atomic, respect overwrite policy, and remove temporary
artifacts after failure. No-clobber publication **MUST NOT** replace a
destination created concurrently. Replace publication **MUST** replace a
destination symlink entry rather than write through to its target and **SHOULD**
preserve the permissions of an existing regular file. File contents **MUST** be
synced before publication; callers **MUST** be able to opt into parent-directory
sync where supported. Creation date **MUST** be omitted unless requested so
reproducible output is the default.

### CREATE-CREATOR-001 — Identify BTPC-created torrents by default

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/create/mod.rs`
- **Verification:** `crates/btpc-core/tests/v1_create.rs`, `crates/btpc-core/tests/v2_create.rs`, `crates/btpc-core/tests/hybrid_create.rs`
- **Depends on:** `CREATE-OUTPUT-001`, `RELEASE-VERSION-001`

New torrents **MUST** include the top-level `created by` value `btpc/<version>` by
default, where `<version>` exactly matches the package/CLI version from the release
source of truth. The value is top-level and **MUST NOT** affect v1 or v2 info hashes.
Rust, CLI, and Python callers **MUST** be able to override the value or explicitly
omit the field. Default creator insertion **MUST** remain deterministic and **MUST
NOT** imply a creation timestamp; creation date remains omitted by default.

## Design Rationale

A straightforward sequential hasher remains the correctness oracle. Optimized
pipelines are introduced only after differential tests and profiles exist.
