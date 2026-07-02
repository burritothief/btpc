# BTPC Agent Guide

## Purpose

BTPC is a high-performance BitTorrent metainfo toolkit with a Rust core,
a native `btpc` CLI, and Python bindings. It supports reading, validating,
creating, inspecting, and writing BitTorrent v1, v2, and hybrid metainfo.

## Source Of Truth

Use the implementation, automated tests, `README.md`, and this guide as the
committed source of truth. Observable behavior changes must update tests and
user-facing guidance in the same change.

## Engineering Rules

- Use test-driven development: add a failing behavioral test before production
  code except for mechanical workspace/bootstrap changes.
- Keep protocol logic in `btpc-core`. CLI and Python layers must remain thin
  adapters with no independent torrent algorithms.
- Treat byte strings as bytes. Decode text only at explicit API boundaries.
- Preserve the exact raw bencoded `info` slice when parsing so existing info
  hashes are computed from source bytes, not re-serialization.
- Canonical output must sort dictionary keys by unsigned raw byte order.
- File traversal and torrent path ordering must be deterministic on every OS.
- Never load payload files wholesale. Creation and verification must use bounded
  memory proportional to piece size and configured concurrency.
- Never use Python for hashing, Merkle construction, file traversal, or bencode
  parsing hot paths.
- v1 hashes the logical concatenated file stream. v2 hashes each file using the
  BEP 52 Merkle rules. Hybrid creation must satisfy both representations.
- Do not add unsafe Rust without a measured need, a documented invariant, and
  targeted tests. Prefer safe Rust by default.
- Public errors must be structured and stable enough for CLI exit-code mapping
  and Python exception mapping; do not expose `anyhow` from library APIs.
- Avoid ambient global thread pools in library APIs. Concurrency must be
  configurable and safe when invoked from Python or embedded Rust applications.

## CLI Implementation Rules

- Do not add CLI-only protocol logic; keep protocol behavior in `btpc-core`.
- Keep default human output minimal. Machine output belongs on stdout; progress,
  warnings, deprecations, and diagnostics belong on stderr.
- Every CLI test must run with an isolated temporary config/home or `--no-config`;
  tests must never depend on a developer's real configuration.
- Config precedence is fixed: defaults, global config, presets in argument order,
  then explicit CLI values. List clearing and deduplication must be deterministic.
- Never expose tracker passkeys or config secrets in snapshots, errors, verbose
  output, JSON, progress, or CI logs. Add explicit redaction tests for every new
  config-rendering path.
- Preserve existing command names, flags, exit codes, JSON schemas, and stream
  behavior unless the relevant todo includes a tested deprecation period.
- Completion installers may write only to documented user completion directories;
  they must never edit shell startup files.
- Batch and multi-input creation must coordinate job and hashing concurrency and
  report results in input order regardless of completion order.
- Human inspect output follows `CLI-INSPECT-DISPLAY-001`: use titled sections,
  aligned labels, humanized IEC sizes, grouped tracker tiers, and stable ordering.
  Never trade pipe safety, redaction, or JSON compatibility for presentation.

## Python API Rules

- Public textual metadata inputs use Python `str` and strict UTF-8 conversion at
  the private native boundary. Keep `bytes` for raw torrent identity, paths,
  unknown fields, file attributes, hashes, and bencode values.
- Keep public domain APIs in `btpc.errors`, `btpc.types`, `btpc.metainfo`,
  `btpc.creation`, and `btpc.verification`, with common names re-exported from
  `btpc`. Keep `_native`, `_conversion`, and similar adapter machinery private.
- Operate on an owned native metainfo object after parsing; do not serialize and
  reparse it merely to implement magnet, edit, verify, or inspection methods.
- Every public Python symbol must remain fully typed for Pyrefly and
  Pyright/Pylance.
  Update runtime annotations, `_native.pyi`, external typing examples, and wheel
  package checks together whenever the Python surface changes.
- Do not mark typing complete from source-tree checks alone; test a built wheel from
  outside the checkout and verify native stub/runtime parity.

## Tooling Baseline

- Rust 2024 edition with an explicit `rust-version` MSRV.
- Stable Rust in development; MSRV and stable are both tested in CI.
- `cargo fmt`, strict `cargo clippy`, `cargo nextest`, and `cargo doc` for Rust.
- `uv`, `maturin`, `pytest`, `ruff`, and Pyrefly for Python.
- `criterion` for microbenchmarks and a reproducible end-to-end benchmark harness
  for comparisons with `mktorrent`, `mkbrr`, `torf`, and `torrenttools`.
- Add dependencies with the package manager rather than guessing versions; pin
  resolved versions in lockfiles and document intentional MSRV constraints.

## Required Verification

Run the narrowest relevant tests while developing. Before completing a todo,
run every command listed in that todo. Before a release or broad refactor, run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --workspace --doc
cargo doc --workspace --all-features --no-deps
uv run ruff check .
uv run ruff format --check .
scripts/check_python_types.sh
uv run pytest tests/python
```

Benchmarks are not a substitute for correctness tests. Record the machine,
dataset, command, tool versions, elapsed time, throughput, and peak RSS for any
performance claim.

## Change Discipline

- Keep changes narrowly scoped.
- Update tests and user-facing guidance with observable behavior changes.
- Do not fix unrelated failures; report them in `Notes:` and add or request a
  separate todo.
- Do not commit generated wheels, benchmark payloads, coverage output, or build
  artifacts.
- Do commit small protocol fixtures when their provenance and expected hashes
  are documented.
- Use conventional, descriptive names. Optimize only after profiling and retain
  a correctness oracle for every optimized path.
