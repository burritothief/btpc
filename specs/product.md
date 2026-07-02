---
spec_id: PRODUCT
title: "Product Direction and Scope"
status: Accepted
owners:
  - "project maintainers"
source_paths:
  - "README.md"
test_paths:
  - "todos.md"
last_reviewed: "2026-07-01"
---

# Product Direction and Scope

## Vision

BTPC is a fast, reliable, embeddable BitTorrent metainfo toolkit. One Rust
implementation powers a reusable core library, a native `btpc` CLI, and a typed
Python `btpc` package. Torrent creation is the first performance target, but the
product covers parsing, inspection, validation, modification, serialization,
hashing, magnet generation, creation, and payload verification for v1, v2, and
hybrid torrents.

## Requirements

### PRODUCT-CORRECT-001 — Prioritize interoperable deterministic output

- **Status:** Accepted
- **Sources:** `README.md`
- **Verification:** `todos.md`
- **Depends on:** `META-HASH-001`, `BENC-ENC-001`, `TEST-LAYERS-001`

BTPC **MUST** prioritize protocol correctness, deterministic output, independent
fixtures, and interoperability over benchmark results. Performance work **MUST NOT**
weaken canonical encoding, exact info hashes, or v1/v2/hybrid invariants.

### PRODUCT-PERF-001 — Compete on measured creation performance

- **Status:** Accepted
- **Sources:** `README.md`
- **Verification:** `specs/benchmarking.md`
- **Depends on:** `PERF-BENCH-001`, `PERF-MEM-001`

BTPC **SHOULD** compete with `mktorrent`, `mkbrr`, `torf`, and `torrenttools` on
equivalent workloads while retaining bounded memory and keeping Python outside hot
hashing, traversal, Merkle, and bencode paths.

### PRODUCT-USABILITY-001 — Provide coherent Rust, CLI, and Python surfaces

- **Status:** Accepted
- **Sources:** `README.md`
- **Verification:** `todos.md`
- **Depends on:** `RUSTAPI-FACADE-001`, `PYAPI-PARITY-001`, `CLI-CMD-001`

The three product surfaces **MUST** share core defaults and behavior. They
**SHOULD** provide typed data, actionable contextual errors, machine-readable CLI
output, progress/cancellation, and distributable artifacts for supported systems.

### PRODUCT-SCOPE-001 — Limit the project to metainfo tooling

- **Status:** Accepted
- **Sources:** `README.md`
- **Verification:** `todos.md`
- **Depends on:** `ARCH-BOUND-001`

Before 1.0, BTPC **MUST NOT** become a peer, tracker, DHT node, download client, or
payload editor. It **MUST NOT** silently normalize malformed metainfo or promise
byte-for-byte preservation after semantic edits; unchanged parsed data may expose
original bytes, while modified data serializes canonically.

### PRODUCT-OPT-001 — Optimize only portable measured bottlenecks

- **Status:** Accepted
- **Sources:** `README.md`
- **Verification:** `specs/benchmarking.md`
- **Depends on:** `PERF-ORACLE-001`, `ARCH-SAFE-001`

Platform-specific asynchronous I/O, GPU hashing, unsafe SIMD, and similar
complexity **MUST NOT** be introduced before profiling proves a material benefit.
Optimized paths **MUST** retain portable safe fallbacks and differential oracles.

## Delivery Milestones

### 0.1 — Correct Foundation

- Strict bencode parsing and canonical encoding.
- Lossless raw `info` bytes and v1/v2 info hashes.
- Typed v1/v2/hybrid inspection and validation.
- Streaming v1 creation.
- CLI create/inspect/validate and Python load/dump/create.
- Cross-platform CI, wheels, fixtures, and baseline creation benchmarks.

### 0.2 — v2, Hybrid, and Verification

- Correct BEP 52 Merkle roots and piece layers.
- v2-only and hybrid creation with padding consistency.
- Payload verification for every mode.
- v1/v2/hybrid magnet generation.

### 0.3 — General-Purpose Editing

- Owned high-level metainfo editing.
- Trackers, web seeds, DHT nodes, comments, source, private, and common extension
  fields with unknown-field preservation.
- Include/exclude, symlink, hidden-file, empty-file, and reproducibility controls.
- Atomic writes and progress/cancellation callbacks.
- Safe CLI metainfo editing, user-scoped TOML configuration, named creation
  presets, tracker aliases/groups, multi-input batch creation, field-oriented
  inspection, and installable shell completions.

### 1.0 — Stable and Competitive

- Stable Rust, Python, CLI, JSON, and error compatibility policies.
- Demonstrated interoperability corpus.
- Representative single-file, many-small-file, and large-tree benchmarks.
- Automated signed/provenance-bearing artifacts where supported.
- No known correctness gaps in advertised v1/v2/hybrid features.

Milestones describe sequencing, not automatic support claims. Individual
requirements remain the source of implementation status.

## Design Rationale

A single Rust protocol core avoids divergent semantics and keeps Python overhead
out of the hot path. A custom bencode implementation is justified by byte identity,
exact source spans, canonical ordering, low-allocation parsing, and precise errors.
Sequential hashing and Merkle implementations remain permanent correctness oracles
before bounded parallel pipelines are accepted.
