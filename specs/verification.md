---
spec_id: VERIFY
title: "Payload Verification"
status: Accepted
owners:
  - "core maintainers"
source_paths:
  - "crates/btpc-core/src/lib.rs"
test_paths:
  - "crates/btpc-core/tests"
last_reviewed: "2026-07-01"
---

# Payload Verification

## Requirements

### VERIFY-PATH-001 — Map torrent paths beneath the payload root

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/verify.rs`
- **Verification:** `crates/btpc-core/tests/verify.rs`
- **Depends on:** `META-V1-001`

Verification **MUST** reject absolute paths, traversal components, and symlink
escape by default, and **MUST** map every torrent path beneath the selected root.

### VERIFY-HASH-001 — Verify every applicable hash domain

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/verify.rs`
- **Verification:** `crates/btpc-core/tests/verify.rs`
- **Depends on:** `VERIFY-PATH-001`, `META-HYBRID-001`

Verification **MUST** check v1 pieces, v2 file roots/piece layers, or both for
hybrid torrents and report deterministic mismatch locations.

### VERIFY-REPORT-001 — Support fail-fast and complete reports

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/verify.rs`
- **Verification:** `crates/btpc-core/tests/verify.rs`
- **Depends on:** `VERIFY-HASH-001`

Callers **MUST** be able to choose fail-fast or collect-all behavior. Reports
**MUST** distinguish missing, extra, wrong-sized, unsafe, and hash-mismatched data.

## Design Rationale

Verification reuses creation oracles to minimize duplicated cryptographic logic
while keeping path safety as a separate mandatory boundary.
