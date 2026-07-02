---
spec_id: ARCH
title: "Workspace Architecture"
status: Accepted
owners:
  - "core maintainers"
source_paths:
  - "Cargo.toml"
  - "crates/btpc-core/src/lib.rs"
  - "crates/btpc-cli/src"
  - "crates/btpc-python/src/lib.rs"
test_paths:
  - "crates/btpc-core/tests"
last_reviewed: "2026-07-01"
---

# Workspace Architecture

## Requirements

### ARCH-BOUND-001 — Keep protocol behavior in the core

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/lib.rs`, `crates/btpc-cli/src`, `crates/btpc-python/src/lib.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** None

BitTorrent parsing, validation, hashing, creation, verification, and serialization
**MUST** live in `btpc-core`. CLI and Python crates **MUST** remain adapters and
**MUST NOT** implement independent protocol algorithms.

### ARCH-DEPS-001 — Preserve one-way dependency direction

- **Status:** Accepted
- **Sources:** `Cargo.toml`
- **Verification:** `Cargo.toml`
- **Depends on:** `ARCH-BOUND-001`

`btpc-cli` and `btpc-python` **MAY** depend on `btpc-core`; `btpc-core` **MUST NOT**
depend on Clap, PyO3, terminal presentation, or Python packaging.

### ARCH-SAFE-001 — Prefer safe Rust

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/lib.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** None

Production code **SHOULD** use safe Rust. Unsafe code **MUST** have a measured need,
documented invariants, focused tests, and explicit reviewer approval.

### ARCH-MODULE-001 — Keep implementation modules reviewable without fragmenting crates

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src`, `crates/btpc-cli/src`, `crates/btpc-python/src`, `python/btpc`
- **Verification:** `crates/btpc-core/tests`, `crates/btpc-cli/tests`, `tests/python`
- **Depends on:** `ARCH-BOUND-001`, `ARCH-DEPS-001`

Large implementation files **SHOULD** be decomposed by responsibility before they
become independent ownership domains, while preserving the three-crate dependency
architecture and existing public import/module paths. Mechanical decomposition
**MUST NOT** create duplicate protocol logic, change public behavior, or introduce
new crates without a measured compile-time, dependency, or ownership benefit.
Internal modules **SHOULD** expose the narrowest visibility that supports reuse.

## Design Rationale

The layered workspace keeps one protocol implementation testable and reusable.
Separate adapters avoid coupling core compatibility and performance to packaging.

The alternatives were rejected for concrete reasons:

1. A combined core/PyO3 crate would couple feature flags, compile times, release
   packaging, and compatibility boundaries.
2. A generic bencode tree often loses raw source spans or creates allocation-heavy
   structures that make exact info hashes and fast parsing harder to guarantee.
3. Optimization-first parallel hashing makes cross-file v1 pieces and BEP 52
   Merkle edge cases harder to prove before a correctness oracle exists.
