---
spec_id: PERF
title: "Performance and Resource Behavior"
status: Accepted
owners:
  - "performance maintainers"
source_paths:
  - "crates/btpc-core/src/bencode.rs"
test_paths:
  - "crates/btpc-core/tests"
last_reviewed: "2026-07-01"
---

# Performance and Resource Behavior

## Requirements

### PERF-MEM-001 — Bound payload-processing memory

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/bencode.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `CREATE-V1-001`

Creation and verification **MUST NOT** load payload files wholesale. Working memory
**MUST** be bounded by configured piece size, in-flight buffers, metadata, and
required piece-layer output rather than total payload size.

### PERF-ORACLE-001 — Retain simple correctness oracles

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/bencode.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `CREATE-V1-001`, `CREATE-V2-001`

Sequential v1 hashing and transparent v2 Merkle implementations **MUST** remain
available to differentially verify optimized paths.

### PERF-POOL-001 — Avoid ambient global execution state

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src/lib.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `ARCH-BOUND-001`

Concurrency **MUST** be per operation and configurable. Library APIs **MUST NOT**
silently configure or depend on a process-wide mutable thread pool.

For v1 creation, `HashThreads::Exact(1)` **MUST** preserve the sequential oracle.
Automatic selection **MUST** use a bounded per-operation pipeline and a conservative
worker heuristic justified by recorded benchmarks. Explicit worker counts **MUST**
bound queued piece buffers and restore piece hashes to logical stream order.
For v2 and hybrid many-file creation, file-level concurrency **MUST** cap active
file descriptors at the selected worker count and restore results to manifest
order. Single-file creation **MUST** avoid parallel worker setup when profiling
does not demonstrate a material benefit.
Default manifest traversal **SHOULD** avoid building glob-match strings when no
include/exclude filters are configured and **SHOULD** reuse directory-entry
metadata while retaining a second snapshot check for mutation detection.

### PERF-BENCH-001 — Validate every benchmark output

- **Status:** Accepted
- **Sources:** `specs/benchmarking.md`
- **Verification:** `specs/benchmarking.md`
- **Depends on:** `VERIFY-HASH-001`, `BENCH-VALID-001`

Performance claims **MUST** record tool versions, hardware, dataset, cache state,
commands, distributions, throughput, and peak RSS. Invalid torrent output **MUST**
count as a failed run, not a performance result.

### PERF-PY-BOUNDARY-001 — Measure Python binding overhead independently

- **Status:** Accepted
- **Sources:** `benches`, `tests/benchmarks`, `crates/btpc-python/src/lib.rs`, `python/btpc`
- **Verification:** `tests/benchmarks`, `benches/btpc_bench`
- **Depends on:** `PERF-BENCH-001`, `PYAPI-NATIVE-OBJECT-001`, `PYAPI-BUFFER-001`

Benchmarks **MUST** distinguish Rust core time from Python adapter overhead for
parse, repeated inspect/property access, magnet generation, editing, verification
setup, and creation setup. Boundary benchmarks **SHOULD** record input copies,
reparses, serialization materialization, wall time, and peak RSS where measurable.
They **MUST** validate equivalent outputs and **MUST NOT** freeze private helper
functions as compatibility APIs; externally meaningful workflows and phases are
the stable benchmark units.

## Design Rationale

The project optimizes measured bottlenecks only after correctness is independently
established. Bounded resources are part of the public performance contract.
