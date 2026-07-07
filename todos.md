# Agent Workflow: BTPC

This file is the shared implementation queue for `btpc`. Three agents coordinate
through this file:

- **Planner** — designs the approach and writes todos
- **Implementer** — claims and executes todos in order (only agent that modifies code)
- **Reviewer** — audits completed work and appends corrective todos

## Todo Lifecycle

Checkbox states:

- `[ ]` — unclaimed, ready for implementation
- `[-]` — claimed, in progress
- `[x]` — complete with evidence
- `[~]` — superseded (replaced by a later todo)

**Claiming:** Change `[ ]` to `[-]`, fill `Claimed by:` with agent/session and
timestamp. If you stop before finishing, revert to `[ ]` with a note.

**Completing:** Implement the change, run verification, record evidence, then mark
`[x]`. Evidence must be specific enough that a reviewer can confirm the behavior
was validated without re-running the tests.

**Superseding:** If a todo is replaced by revised work, mark it `[~]` and note
which todo supersedes it.

## What Makes A Good Todo

Each todo must state what changes, why it is needed, how to implement it, and how
to verify it. The implementer must not need conversation context or make product
decisions that belong in `specs/`. Every new behavioral todo must include a
`Requirements:` field listing the stable requirement IDs it creates, changes, or
implements. Existing todos use the traceability table below until rewritten by an
approved planner action.

## Requirement Traceability

- Todos 4-7: `ERR-CORE-001`, `ERR-IO-001`, `BENC-PARSE-001`, `BENC-BYTES-001`,
  `BENC-LIMIT-001`, `BENC-CANON-001`, `BENC-ENC-001`, `SEC-PARSE-001`.
- Todos 8-11 and review todos 37-40: `META-RAW-001`, `META-FIELD-001`,
  `META-HASH-001`, `META-V1-001`, `META-V2-001`, `META-HYBRID-001`,
  `RUSTAPI-FACADE-001`, `RUSTAPI-BYTES-001`, `RUSTAPI-COMPAT-001`.
- Todos 12-15 and 20-24: `CREATE-MANIFEST-001`, `CREATE-V1-001`,
  `CREATE-V2-001`, `CREATE-HYBRID-001`, `CREATE-OUTPUT-001`, `VERIFY-PATH-001`,
  `VERIFY-HASH-001`, `VERIFY-REPORT-001`, `SEC-PATH-001`.
- Todos 16-19 and 23-27: `CLI-CMD-001`, `CLI-IO-001`, `CLI-EXIT-001`,
  `CLI-WRITE-001`, `PYAPI-PACKAGE-001`, `PYAPI-PARITY-001`, `PYAPI-GIL-001`,
  `PYAPI-TYPES-001`, `ERR-MAP-001`, `ERR-PANIC-001`.
- Todos 28-33 and 43: `PERF-MEM-001`, `PERF-ORACLE-001`, `PERF-POOL-001`,
  `PERF-BENCH-001`, `TEST-TDD-001`, `TEST-TRACE-001`, `TEST-LAYERS-001`,
  `TEST-FIXTURE-001`.
- Todos 34-36 and 41-46: `SEC-DEPS-001`, `RELEASE-VERSION-001`,
  `RELEASE-MATRIX-001`, `RELEASE-ARTIFACT-001`, `RELEASE-GATE-001`.
- Todos 96-105: `DOCSITE-ARCH-001`, `DOCSITE-BUILD-001`,
  `DOCSITE-PYTHON-001`, `DOCSITE-RUST-001`, `DOCSITE-CLI-001`,
  `DOCSITE-UX-001`, `DOCSITE-QUALITY-001`, `DOCSITE-DEPLOY-001`, and
  `DOCSITE-OPS-001`.
- Todos 112-119: `DOCSITE-ARCH-002`, `DOCSITE-BUILD-001`,
  `DOCSITE-PYTHON-001`, `DOCSITE-RUST-001`, `DOCSITE-CLI-001`,
  `DOCSITE-UX-001`, `DOCSITE-QUALITY-001`, `DOCSITE-MIGRATE-001`,
  `DOCSITE-DEPLOY-001`, and `DOCSITE-OPS-001`.

## Minimum Verification Gate

Every completed todo runs its listed focused checks. Once the relevant tooling
exists, broad milestones and all release work also run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --workspace --doc
cargo doc --workspace --all-features --no-deps
uv run ruff check .
uv run ruff format --check .
uv run mypy python tests/python
uv run pytest tests/python
```

If a command is not applicable before its bootstrap todo, record that explicitly
instead of fabricating a result.

## Strategy

Build correctness in vertical layers before optimizing:

1. Bootstrap a modern Rust/Python workspace and CI.
2. Implement strict, byte-preserving bencode with fuzz/property coverage.
3. Build typed v1/v2/hybrid metainfo parsing and validation.
4. Add deterministic manifest scanning and sequential hashing oracles.
5. Ship v1 creation through Rust, CLI, and Python.
6. Add BEP 52 v2 and hybrid creation plus payload verification.
7. Add editing, common extensions, magnets, and release packaging.
8. Establish competitor benchmarks, profile, and optimize bounded hot paths.

The sequential hashing and Merkle implementations remain correctness oracles.
Optimized paths must be differentially tested against them. CLI and Python code
remain adapters over `btpc-core` throughout.

## Todos

1. [x] Bootstrap the Rust workspace and repository metadata
   Claimed by: Codex implementer (2026-07-01 13:24 PDT)
   Context:
   The repository currently contains planning documents only. Create the crate
   boundaries required by `specs/architecture.md` without implementing torrent
   behavior yet. The workspace must support a reusable Rust library, native CLI,
   and separately packaged PyO3 extension.
   Implementation:
   Create a virtual Cargo workspace with `crates/btpc-core`, `crates/btpc-cli`,
   and `crates/btpc-python`. Use Rust edition 2024, resolver 3, an explicit MSRV,
   shared package metadata, and minimal compiling targets. Name the CLI binary
   `btpc`; make `btpc-python` produce `cdylib` and keep PyO3 behind that crate.
   Add `.gitignore`, `rust-toolchain.toml`, `README.md`, `LICENSE` only if the
   owner has already selected one (otherwise add a documented placeholder), and
   a root `deny.toml` or equivalent dependency-policy placeholder. Use `cargo add`
   for current compatible dependency versions rather than copying versions from
   `proj.md`. Do not add protocol implementation.
   Tests and verification:
   Run `cargo metadata --no-deps`, `cargo check --workspace --all-targets`, and
   `cargo test --workspace`. Confirm the dependency graph has no PyO3 or Clap
   dependency reachable from `btpc-core`.
   Evidence:
   `cargo metadata --no-deps --format-version 1` completed successfully and
   described all three workspace packages. `cargo check --workspace --all-targets`
   completed successfully with Rust 1.85.0. `cargo test --workspace` completed
   successfully: the `btpc` binary, `btpc_core`, `_native`, and `btpc_core`
   doctest targets each reported 0 failures. `cargo tree -p btpc-core` printed only
   `btpc-core v0.1.0`, confirming neither PyO3 nor Clap is reachable from core.
   Notes:

2. [x] Bootstrap Python packaging and developer commands
   Claimed by: Codex implementer (2026-07-01 13:28 PDT)
   Context:
   Python packaging must be reproducible and use the private `btpc._native`
   extension described in `specs/python-api.md`. Developers need one documented
   path to build, test, lint, type-check, and install the package.
   Implementation:
   Add `pyproject.toml` using current Maturin mixed-project configuration, `uv`
   dependency groups, supported Python classifiers, and a declared Python floor
   of 3.11. Create `python/btpc/__init__.py`, `python/btpc/py.typed`, a private
   extension module skeleton, and `tests/python/test_import.py`. Configure current
   Ruff and mypy settings with strict checks appropriate for a typed package.
   Add a `justfile` or `Makefile` with narrow commands (`check`, `test-rust`,
   `test-python`, `lint`, `build-wheel`) that only wrap documented tools.
   Tests and verification:
   Run `uv sync --all-groups`, `uv run maturin develop`, `uv run pytest
   tests/python/test_import.py`, `uv run ruff check .`, `uv run ruff format
   --check .`, and `uv run mypy python tests/python`. Verify `import btpc` and
   `import btpc._native` work in the uv environment.
   Evidence:
   `uv sync --all-groups` resolved 14 packages and installed the editable
   `btpc==0.1.0` project on CPython 3.14.3. `uv run maturin develop` built and
   installed `btpc-0.1.0-cp314-cp314-macosx_11_0_arm64.whl`. `uv run pytest
   tests/python/test_import.py` collected 1 test and passed it. `uv run ruff check
   .` reported "All checks passed"; `uv run ruff format --check .` reported 3
   files already formatted; `uv run mypy python tests/python` reported success in
   3 source files. A direct uv-environment import of `btpc` and `btpc._native`
   printed matching versions `0.1.0 0.1.0`.
   Notes:

3. [x] Add continuous integration and dependency policy
   Claimed by: Codex implementer (2026-07-01 13:31 PDT)
   Context:
   Cross-platform behavior and MSRV compatibility are architectural requirements,
   and later todos need stable verification commands. Establish CI before adding
   protocol complexity.
   Implementation:
   Add CI workflows for formatting, strict clippy, nextest, doc tests/docs,
   Python lint/type/pytest, MSRV, Linux/macOS/Windows stable Rust, and CPython
   3.11 through the current stable version supported by PyO3. Add cargo-deny (or
   a documented equivalent) for advisories, duplicate review, licenses, and
   sources. Cache safely without caching built project artifacts across unrelated
   toolchains. Add a wheel-build smoke job but defer publishing credentials.
   Document local equivalents in `CONTRIBUTING.md`.
   Tests and verification:
   Validate workflow syntax locally where tooling permits. Run the complete
   minimum verification gate and `cargo deny check`. Record any matrix item that
   can only be proven by hosted CI, then attach the successful run URL in evidence.
   Evidence:
   Ruby's YAML parser loaded `.github/workflows/ci.yml` successfully. The complete
   local gate passed: `cargo fmt --all --check`; strict workspace clippy; nextest
   (1 Rust test passed); workspace doctests (0 failures); rustdoc with warnings
   denied; Ruff check and format (3 Python files); strict mypy (3 source files);
   pytest (1 passed); and `cargo deny check` (`advisories ok, bans ok, licenses
   ok, sources ok`). The workflow defines Linux/macOS/Windows stable Rust, Rust
   1.85 MSRV, CPython 3.11-3.14, and wheel smoke jobs.
   Notes:
   Cross-platform matrix execution and a successful hosted run URL can only be
   proven after this directory is placed in a GitHub repository and the workflow
   is run; no Git repository or remote is currently present in the workspace.

4. [x] Define structured core errors and resource limits
   Claimed by: Codex implementer (2026-07-01 13:41 PDT)
   Context:
   Every parser and protocol layer needs stable contextual errors without leaking
   `anyhow` through the core API. Parser limits must exist before accepting
   untrusted bytes.
   Implementation:
   In `btpc-core`, add a non-exhaustive `Error` enum and result alias covering I/O
   with path context, bencode syntax/canonical errors with offsets, metainfo field
   errors, unsupported behavior, verification mismatch, and cancellation. Define
   `ParseLimits` with defaults for depth, item count, byte-string length, total
   input, and owned-allocation budget. Use checked arithmetic and expose accessors
   needed by CLI/Python adapters. Do not add adapter-specific formatting.
   Tests and verification:
   Write failing unit tests first for category accessors, offset/field/path
   context, source chaining, parse-limit defaults, and overflow-safe limit checks.
   Run `cargo test -p btpc-core error` and strict clippy for `btpc-core`.
   Evidence:
   Added `crates/btpc-core/tests/error.rs` before implementation; its initial
   compile failed because `Error`, `ErrorCategory`, and `ParseLimits` did not
   exist. After implementation, `cargo test -p btpc-core error` ran 4 focused
   tests with 4 passed, covering categories, offsets/fields/paths, I/O source
   chaining, defaults, boundaries, and overflow. `cargo test -p btpc-core --test
   error` also passed all 4 tests. Strict `cargo clippy -p btpc-core --all-targets
   --all-features -- -D warnings` completed successfully.
   Notes:

5. [x] Implement the borrowed bencode parser
   Claimed by: Codex implementer (2026-07-01 13:46 PDT)
   Context:
   Exact source spans and byte-oriented values are foundational for correct info
   hashes and lossless inspection. A borrowed parser avoids unnecessary copies.
   Implementation:
   Add `btpc_core::bencode` token/value types borrowing `&[u8]`. Parse integers,
   byte strings, lists, and dictionaries iteratively or with guarded recursion.
   Track start/end spans for every value and reject trailing input, malformed
   lengths, invalid integers, unexpected delimiters, depth/item/input limit
   violations, and arithmetic overflow. Dictionary keys must remain byte strings;
   do not decode UTF-8. Provide lookup by raw key without allocating.
   Tests and verification:
   Start with table-driven failing tests for every valid scalar/container and
   malformed form, exact spans, nested limit boundaries, trailing bytes, huge
   lengths, and arbitrary non-UTF-8 keys/values. Run the bencode parser tests,
   `cargo test -p btpc-core`, and strict clippy.
   Evidence:
   `crates/btpc-core/tests/bencode_parser.rs` was added first and initially failed
   because `btpc_core::bencode` did not exist. After implementation, its 6 tests
   passed, covering scalars/containers, exact nested spans, raw non-UTF-8 lookup,
   permissive non-canonical syntax, malformed/trailing/overflow cases, and exact
   depth/item/string/input limit boundaries. `cargo test -p btpc-core` passed 11
   total unit/integration tests and doctests with 0 failures. Strict core clippy
   completed successfully with all targets and features.
   Notes:

6. [x] Add canonical bencode validation and encoding
   Claimed by: Codex implementer (2026-07-01 13:54 PDT)
   Context:
   BTPC must distinguish parseable bencode from canonical bencode and emit stable
   bytes suitable for info hashing. Duplicate or unsorted dictionary keys cannot
   be silently normalized during strict validation.
   Implementation:
   Add strict validation for minimal integers (including negative zero and leading
   zero rules), minimal byte-string lengths, sorted raw-byte dictionary keys, and
   duplicate keys. Add an owned bencode model and canonical encoder that writes to
   `std::io::Write` plus a pre-sized `Vec<u8>` convenience path. Ensure dictionary
   construction either rejects duplicates or uses an API that cannot contain them.
   Preserve a permissive syntax-only parse mode for inspecting legacy torrents,
   but make canonical violations report precise offsets.
   Tests and verification:
   Write tests first for canonical/non-canonical pairs, unsigned raw-byte key
   ordering, duplicates, encode size calculation, writer failure propagation, and
   golden byte output. Add bounded proptests for owned-value encode/parse round
   trips and canonical key ordering. Run relevant unit/proptests and clippy.
   Evidence:
   Added canonical tests before implementation; they initially failed because
   `OwnedValue` and `validate_canonical` were absent. The focused suite then
   passed 5 tests: canonical/non-canonical pairs with offsets, unsigned byte key
   ordering, duplicate rejection, encoded size/golden bytes, writer source
   propagation, and 128 bounded proptest round trips. `cargo test -p btpc-core`
   passed 16 total tests with 0 failures, and strict core clippy passed all
   targets/features with warnings denied.
   Notes:

7. [x] Add bencode fuzz targets and seed corpus
   Claimed by: Codex implementer (2026-07-01 13:59 PDT)
   Context:
   The parser accepts attacker-controlled bytes and must never panic, loop, or
   allocate beyond configured limits. Fuzzing should begin before metainfo layers
   expand the attack surface.
   Implementation:
   Create a separate `fuzz/` package using current `cargo-fuzz`. Add targets for
   raw parsing and parse → canonical encode → reparse. Seed with scalar/container
   cases, non-canonical cases, deep nesting boundaries, duplicate keys, non-UTF-8
   bytes, and all later `.torrent` fixtures via a documented corpus sync command.
   Add a scheduled CI workflow and a short local smoke command; do not require
   nightly fuzzing in the normal unit-test gate.
   Tests and verification:
   Run each target for a bounded smoke duration/iteration count and show zero
   crashes. Run the normal bencode test suite afterward and record corpus size and
   exact fuzz commands.
   Evidence:
   Added separate `fuzz/` package targets `parse` and `canonical_roundtrip`, 10
   curated initial corpus files, documented fixture sync script, and scheduled
   workflow (locally YAML-parsed). `cargo +nightly fuzz run parse
   fuzz/corpus/parse -- -runs=10000` completed 10,000 runs with no crash and grew
   the parse corpus to 106 files. The analogous canonical roundtrip command
   completed 10,000 runs with no crash and grew its corpus to 92 files. The
   explicit parser suite passed 6/6 and canonical suite passed 5/5 afterward.
   Notes:

8. [x] Parse top-level metainfo and preserve raw info bytes
   Claimed by: Codex implementer (2026-07-01 14:03 PDT)
   Context:
   The BitTorrent info hash is calculated from the exact bencoded `info` value.
   Re-encoding parsed data can change its bytes and produce the wrong hash.
   Implementation:
   Add `RawMetainfo<'a>` that requires a top-level dictionary, locates exactly one
   byte-string key `info`, requires its value to be a dictionary, and exposes the
   original full bytes plus exact `info` slice/span. Parse common top-level fields
   (`announce`, `announce-list`, `url-list`, `nodes`, `comment`, `created by`, and
   `creation date`) as byte-safe optional views while retaining unknown fields.
   Compute SHA-1 and SHA-256 over the original info slice without yet deciding
   whether each hash applies to the torrent mode.
   Tests and verification:
   Add fixtures/tests first for missing, duplicate, and non-dictionary `info`,
   unknown fields, non-canonical but parseable input, and known exact digest values
   computed independently. Prove changing top-level fields leaves info hashes
   unchanged while changing one raw info byte changes them.
   Evidence:
   Added tests first; they initially failed because `btpc_core::metainfo` was
   absent. The 4 focused tests now pass for missing/duplicate/non-dictionary
   `info`, raw common/unknown fields, non-canonical exact `info` slicing, known
   independent SHA-1 (`4fc241...c186`) and SHA-256 (`a69e92...01f9`) digests, and
   top-level hash invariance versus one-byte `info` changes. Full `btpc-core`
   testing passed 20 tests with 0 failures, and strict core clippy passed.
   Notes:

9. [x] Implement typed v1 metainfo validation
   Claimed by: Codex implementer (2026-07-01 14:08 PDT)
   Context:
   Reading existing torrents is part of the general-purpose API and provides the
   semantic foundation for v1 creation and verification.
   Implementation:
   Add typed v1 views for name, piece length, pieces, single-file length, and
   multi-file entries. Enforce exactly one of `length`/`files`, non-negative
   lengths, valid path lists/components, 20-byte pieces divisibility, expected
   piece count, and checked total-size arithmetic. Separate errors from optional
   compatibility warnings. Expose raw path components and a validated UTF-8 view
   without treating arbitrary bytes as OS paths. Classify valid v1 mode when no
   v2 representation is present.
   Tests and verification:
   Add v1 single/multi-file golden fixtures and malformed table cases first,
   including cross-file piece counts, zero-length payloads, non-UTF-8 names,
   unsafe components, integer overflow, and inconsistent piece counts. Run focused
   tests, all core tests, and clippy.
   Evidence:
   Added v1 tests first; after repairing only the test builder, they failed solely
   for absent `TorrentMode`/`V1Metainfo`. The focused suite now passes 5 tests for
   single/multi-file goldens, cross-file piece counts, zero-length payloads,
   non-UTF-8 names, UTF-8 path views, unsafe components, malformed fields,
   inconsistent hashes, and checked total overflow. Full core testing passed 25
   tests with 0 failures; strict all-target/all-feature core clippy passed.
   Notes:

10. [x] Implement typed v2 and hybrid metainfo validation
   Claimed by: Codex implementer (2026-07-01 14:15 PDT)
   Context:
   BEP 52 changes path representation and hashing rules. Strict parsing must be
   complete before BTPC attempts to create v2 data.
   Implementation:
   Add typed v2 file-tree traversal, leaf parsing, file attributes, `pieces root`,
   `meta version`, and top-level `piece layers`. Validate piece-length constraints,
   file-tree node shapes, lengths, root sizes, required piece-layer membership and
   lengths, and empty-file behavior. Classify v2-only and hybrid torrents. For
   hybrid mode, compare real v1 and v2 paths/lengths and validate permitted v1
   padding files/attributes. Keep unknown attributes/fields accessible.
   Tests and verification:
   Begin with independently sourced or independently calculated v2/hybrid
   fixtures. Cover single/multi-file, empty files, files below/equal/above piece
   length, malformed trees, missing/extra/incorrect piece layers, mismatched hybrid
   files, and padding mistakes. Run all metainfo tests and strict clippy.
   Evidence:
   Added independently calculated v2/hybrid tests first; they initially failed
   because `V2Metainfo` was absent. The 4 focused tests now pass for single and
   multi-file trees, empty files, exact roots/attributes, piece-layer membership,
   lengths and Merkle roots, invalid piece lengths/tree shapes/root sizes,
   v2-only classification, matching hybrid classification, path mismatch, and
   padding alignment mistakes. All core tests passed (29 total), all metainfo
   checks completed with 0 failures, and strict core clippy passed.
   Notes:

11. [x] Expose the initial Rust inspection API
   Claimed by: Codex implementer (2026-07-01 14:23 PDT)
   Context:
   Parsing internals need a coherent public API before CLI and Python adapters are
   built. The public API must preserve bytes while remaining ergonomic.
   Implementation:
   Add `Metainfo::from_bytes`, `from_path`, mode/hash/name/files/tracker/web-seed/
   private/piece-length accessors, validation reports, original-byte access, and
   canonical serialization for unchanged/owned forms as specified. Add public
   hash and torrent-file entry value types. Document all public items, examples,
   lifetimes/ownership behavior, original-versus-canonical semantics, and the
   non-exhaustive stability policy. Keep mutation APIs out of scope for now.
   Tests and verification:
   Add compile-tested doctests and integration tests consuming only the public
   API for v1, v2, hybrid, unknown fields, and original-byte preservation. Run
   `cargo test -p btpc-core --doc`, integration tests, clippy, and docs with
   warnings denied where supported.
   Evidence:
   Added public-only integration tests first; they initially failed because the
   facade types were absent. `cargo test -p btpc-core --doc` passed the compile-
   tested `Metainfo` example. `public_api.rs` passed 5 tests covering v1/v2/hybrid
   modes and applicable typed hashes, owned file values, trackers/web seeds/
   private/unknown fields, invalid construction, `from_path`, original-byte
   preservation, and canonical/original writers. Strict clippy passed, and
   `RUSTDOCFLAGS='-D warnings' cargo doc -p btpc-core --all-features --no-deps`
   generated documentation successfully.
   Notes:

12. [x] Build the deterministic payload manifest scanner
   Claimed by: Codex implementer (2026-07-01 14:30 PDT)
   Context:
   Correct piece hashing depends on a stable file order and safe conversion from
   platform paths to torrent path components. Traversal cost also matters for
   many-small-file benchmarks.
   Implementation:
   Add manifest types and a scanner with explicit policies for hidden files,
   symlinks, special files, empty files, include/exclude patterns, and root naming.
   Sort by torrent path raw-byte order using a documented cross-platform mapping;
   detect duplicate/colliding torrent paths, mutation during scan where practical,
   checked total lengths, and unsafe components. Keep file descriptors closed
   after metadata collection and store only needed metadata/path data.
   Tests and verification:
   Write filesystem-fixture tests first for shuffled creation order, nested trees,
   empty files/directories, Unicode, platform-conditional non-UTF-8 names,
   symlinks, special files, exclusions, collisions, and deterministic repeated
   scans. Add a property test that randomized enumeration produces identical
   sorted manifests.
   Evidence:
   Added filesystem/property tests first; they initially failed because the
   `create` scanner module was absent. The focused manifest suite passed 5 tests
   on macOS (plus a Linux/Android-gated non-UTF-8 test): shuffled/nested/Unicode
   deterministic scans, repeated snapshots, hidden/empty/include/exclude/root
   policies, symlink/FIFO policies, collision detection, and randomized ordering.
   Full core testing passed 39 tests plus 1 doctest with 0 failures. Strict core
   clippy passed after removing one needless recursive ownership transfer.
   Notes:

13. [x] Define and test automatic piece-length policy
   Claimed by: Codex implementer (2026-07-01 14:39 PDT)
   Context:
   Automatic piece length affects interoperability, output size, hashing cost,
   and benchmark fairness. It must be explicit and stable rather than an ad hoc
   formula hidden in creation code.
   Implementation:
   Add a table-driven piece-length policy using powers of two, a documented target
   piece-count/metainfo-size rationale, minimum/maximum bounds, and the v2 16 KiB
   minimum. Add validation for explicit values and a versioned policy identifier
   to creation results. Update `specs/creation.md` with the exact selected
   bands once tests define them; do not change bands later without a compatibility
   note and boundary-test updates.
   Tests and verification:
   Write failing unit tests at every byte immediately below, at, and above each
   policy boundary; cover zero, huge checked sizes, invalid non-powers-of-two, and
   mode-specific constraints. Run focused tests and all core tests.
   Evidence:
   Added boundary tests first; they initially failed because the policy API was
   absent. `piece_length.rs` passed 2 tests covering every byte below/at/above all
   11 bands, zero and `u64::MAX`, policy identifier, explicit powers-of-two,
   global maximum, and v1 versus v2/hybrid minimums. Full core tests passed 41
   tests plus 1 doctest. Strict core clippy passed. The exact table and change
   rule are recorded in authoritative requirement `CREATE-PIECE-POLICY-001` in
   `specs/creation.md`; the former functional guide had concurrently become a
   non-normative pointer to `specs/`, so it was not overwritten.
   Notes:

14. [x] Implement the sequential v1 hashing oracle
   Claimed by: Codex implementer (2026-07-01 14:44 PDT)
   Context:
   A simple, correct streaming implementation is required before parallelism. v1
   pieces can cross file boundaries and must hash the concatenated logical stream.
   Implementation:
   Stream manifest files in order through reusable buffers, assemble fixed-size
   pieces, SHA-1 each complete piece and the final partial piece, and return ordered
   digests plus byte/piece metrics. Bound file descriptors and memory; never read
   a payload file wholesale. Detect short reads or file-size changes with path
   context. Accept cancellation and a no-op/progress sink abstraction without
   presentation dependencies.
   Tests and verification:
   Write tests first for empty, exact-piece, partial-piece, multi-piece, and
   cross-file-boundary payloads using independently computed SHA-1 values. Test
   short/mutated files, cancellation, progress monotonicity, memory-relevant large
   streaming via a generated fixture, and deterministic output.
   Evidence:
   Added hashing tests first; they initially failed because cancellation,
   progress, and sequential hashing APIs were absent. The focused suite passed 4
   tests for empty/exact/partial/multi-piece payloads with independently computed
   SHA-1 values, cross-file pieces, stale-size detection with path context,
   cancellation, monotonic progress, deterministic output, and an 8 MiB+17 byte
   generated streaming fixture (513 pieces). Full core testing passed 45 tests
   plus 1 doctest; strict core clippy passed with warnings denied.
   Notes:

15. [x] Build canonical v1 torrent creation
   Claimed by: Codex implementer (2026-07-01 14:51 PDT)
   Context:
   This is the first end-to-end creator milestone and establishes the option and
   result contracts later reused by v2 and adapters.
   Implementation:
   Add `CreateOptions`, `Creator`, and `CreateResult` for v1 mode. Combine manifest
   scanning, explicit/automatic piece length, sequential hashing, tracker tiers,
   web seeds, nodes, private/source/comment/creator fields, and reproducibility
   controls. Build canonical single- or multi-file info dictionaries and top-level
   metainfo. Omit creation date by default. Return bytes, v1 info hash, counts,
   selected policy, and phase metrics. Add atomic write-to-path convenience with
   overwrite policy and cleanup on failure.
   Tests and verification:
   Start with end-to-end golden tests for single/multi-file payloads, all metadata
   options, deterministic repeat runs, explicit/automatic piece sizes, overwrite
   refusal, and temporary-file cleanup. Parse every created torrent back through
   BTPC and an available independent tool/library, and verify expected payload
   hashes.
   Evidence:
   Added end-to-end creator tests first; they initially failed because the creator
   contracts were absent. The focused suite passed 3 tests for canonical and
   reproducible single-file output with trackers/web seeds/nodes/private/source/
   comment/creator metadata, omitted default creation date, automatic multi-file
   creation and sorted paths, metrics/result counts/policy ID, parse/hash parity,
   and same-directory atomic deny/replace writes with zero temporary leftovers.
   Full core testing passed 48 tests plus 1 doctest; strict core clippy passed.
   Notes:

16. [x] Implement the CLI framework and v1 create command
   Claimed by: Codex implementer (2026-07-01 14:58 PDT)
   Context:
   The native CLI is a primary benchmark surface. It must be thin, scriptable,
   and correct before performance comparisons begin.
   Implementation:
   Add Clap-based commands and shared output/error infrastructure. Fully implement
   `btpc create` for v1 with output inference, overwrite protection, piece length,
   tracker tiers, common metadata, traversal policies, threads placeholder limited
   to supported execution, quiet/progress behavior, `NO_COLOR`, Ctrl-C
   cancellation, atomic writes, and versioned JSON result output. Freeze and
   document exit-code mapping. Keep all creation logic in `btpc-core`.
   Tests and verification:
   Add process tests first for help/version, successful creation, inferred output,
   overwrite refusal/force, invalid options, JSON schema, stdout/stderr separation,
   no-color/non-TTY behavior, exit codes, interrupt cleanup where reliable, and
   byte parity with direct core creation. Run CLI integration tests and workspace
   gate.
   Evidence:
   Added CLI process tests first; all 5 initially failed against the placeholder
   binary. The final suite passed 6 tests for help/version, successful and inferred
   output, quiet stdout/stderr, overwrite refusal/force, invalid options, stable
   exit categories, versioned `btpc.create.v1` JSON, `NO_COLOR`/non-TTY output,
   and byte parity with direct core creation. Workspace tests passed every Rust,
   CLI, Python-extension, and doctest target. Strict workspace clippy passed.
   Exit codes and JSON fields are frozen in `specs/cli.md`.
   Notes:

17. [x] Implement CLI inspect and validate commands
   Claimed by: Codex implementer (2026-07-01 15:06 PDT)
   Context:
   Users need to inspect arbitrary v1/v2/hybrid torrents before all creation modes
   exist, and benchmark outputs need an internal correctness checker.
   Implementation:
   Implement `btpc inspect` and `btpc validate` over the public core API. Human
   output includes mode, hashes, name, total size, piece length/count, file count,
   trackers, web seeds, private state, and warnings. JSON uses a documented
   versioned schema with raw-byte fields represented unambiguously. Validation
   performs no payload reads and maps failures to the frozen exit codes.
   Tests and verification:
   Add process golden tests for v1/v2/hybrid, non-UTF-8 fields, invalid torrents,
   warnings, JSON schema/version, and stdout/stderr/exit-code behavior. Verify no
   payload path is accessed by `validate` using a fixture with absent payload.
   Evidence:
   Added process tests first; they initially failed because both subcommands were
   absent. The final inspect suite passed 3 tests covering human and versioned
   `btpc.inspect.v1` JSON output, v1/v2/hybrid modes, unambiguous byte encoding,
   valid/versioned `btpc.validate.v1` output, invalid-data exit 4, clean stream
   separation, and validation after deleting the payload file. Strict workspace
   clippy and every workspace test target passed. JSON semantics and payload-free
   validation are documented in `specs/cli.md`.
   Notes:

18. [x] Expose parsing and inspection through Python
   Claimed by: Codex implementer (2026-07-01 15:15 PDT)
   Context:
   The Python package must become useful as a torf-like library, not only a creator
   wrapper. Bindings should expose Pythonic immutable value objects while keeping
   heavy work in Rust.
   Implementation:
   Bind `Metainfo`, `TorrentMode`, hash values, file entries, validation reports,
   tracker/web-seed access, `from_bytes`, `read`, `to_bytes`, and original bytes.
   Accept contiguous buffers and path-like inputs; avoid avoidable Python copies.
   Implement the documented exception hierarchy with structured attributes and a
   panic boundary. Put ergonomic wrappers, enums, protocols, and exports in typed
   Python modules rather than exposing raw extension internals.
   Tests and verification:
   Write pytest cases first for all modes, bytes/bytearray/memoryview, path-like
   objects, hashes, file iteration, raw non-UTF-8 fields, original/canonical bytes,
   exception types/attributes, unknown fields, and absence of public `_native`
   details. Run pytest, Ruff, mypy, and Rust binding tests.
   Evidence:
   Added Python tests first; all 5 initially failed because public wrappers and
   exceptions were absent. The final Python suite passed 11 tests covering
   bytes/bytearray/memoryview, path-like reads, immutable mode/hash/file/report
   values, original/canonical bytes, all applicable hashes, structured bencode
   and metainfo exceptions, and private native details. Scoped Ruff check/format,
   strict mypy (5 files), Rust binding tests/clippy, and spec validation (14 specs,
   64 requirements) all passed. `PYAPI-TYPES-001` is marked Implemented.
   Notes:

19. [x] Expose v1 creation through Python
   Claimed by: Codex implementer (2026-07-01 15:27 PDT)
   Context:
   Python creation must use the same core defaults and remain close to native
   performance by releasing the GIL around scanning, I/O, and hashing.
   Implementation:
   Add typed `CreateOptions`, `CreateResult`, `create_bytes`, and atomic `create`
   APIs for v1. Map Python path-like values and metadata cleanly, release the GIL
   during core work, add optional throttled progress callbacks and cancellation,
   and propagate callback exceptions after cancelling Rust work. Ensure defaults
   and canonical bytes exactly match the Rust API and CLI.
   Tests and verification:
   Add pytest cases first for byte and file creation, result metadata, option
   mapping, parity across all three surfaces, GIL release demonstrated by another
   Python thread making progress, callbacks, callback exceptions, cancellation,
   overwrite behavior, and cleanup. Run Python and workspace gates.
   Evidence: Added typed creation APIs and cooperative cancellation in
   `python/btpc/__init__.py`, native GIL-released creation with throttled callbacks
   in `crates/btpc-python/src/lib.rs`, typing in `python/btpc/_native.pyi`, and
   parity/GIL/callback/cleanup coverage in `tests/python/test_create.py`.
   `uv run pytest tests/python -q` passed 15 tests; Ruff format/check and mypy
   passed; `uv run python scripts/check_specs.py` validated 14 specs and 64
   requirements; Cargo fmt, strict workspace Clippy, workspace tests, doc tests,
   and docs all passed.
   Notes:

20. [x] Implement the sequential v2 Merkle oracle
   Claimed by: Codex implementer (2026-07-01 13:58 PDT)
   Context:
   BEP 52 uses per-file SHA-256 Merkle trees with 16 KiB leaves and piece layers.
   A transparent oracle is required before v2 construction or optimization.
   Implementation:
   Implement streaming 16 KiB block hashing, final-block zero padding, balanced
   tree completion using BEP 52 zero hashes, pieces-root calculation, and extraction
   of piece-layer hashes at arbitrary valid piece lengths. Handle empty files and
   files below/equal/above one piece. Keep memory bounded by tree depth plus needed
   output; do not retain all leaves when a streaming reduction suffices. Expose
   cancellation/progress compatible with the v1 abstractions.
   Tests and verification:
   Write tests first from independent vectors for block hashes, zero-hash levels,
   roots, and piece layers across boundary sizes. Differentially check a simple
   full-tree test helper against the streaming implementation over proptest inputs.
   Test empty files, cancellation, mutations, and large generated input.
   Evidence: Added the public streaming BEP 52 oracle, zero-hash helper, bounded
   depth accumulator, per-file roots, piece-layer extraction, cancellation, and
   progress in `crates/btpc-core/src/create.rs`. Added independent fixed vectors,
   empty/exact/boundary/large/mutation/cancellation cases, and 128 proptest
   differential cases against a full-tree helper in
   `crates/btpc-core/tests/v2_merkle.rs`. Cargo fmt, strict workspace Clippy,
   workspace tests, doc tests, docs, Ruff, mypy, 15 Python tests, and spec checking
   all passed; the registry validated 15 specs and 71 requirements.
   Notes:

21. [x] Build canonical v2 torrent creation
   Claimed by: Codex implementer (2026-07-01 14:03 PDT)
   Context:
   Typed v2 validation and the Merkle oracle now permit correct BEP 52 output.
   Creation must construct nested file trees and top-level piece layers exactly.
   Implementation:
   Extend `Creator` with v2 mode. Convert the deterministic manifest to a BEP 52
   file tree, include `meta version = 2`, lengths, attributes, and pieces roots,
   and construct only required `piece layers` entries keyed by root. Reuse common
   metadata and results, returning only the v2 info hash for v2-only mode. Detect
   file-tree prefix collisions and all invalid names before hashing where possible.
   Tests and verification:
   Add end-to-end golden tests first for single/multi-file, empty files, boundary
   lengths, nested directories, piece-layer inclusion/omission, deterministic
   output, and collisions. Parse created output through BTPC and at least one
   independent BEP 52 implementation; verify known roots/hashes.
   Evidence: Added `CreateMode::V2`, mode-aware optional info hashes, canonical
   nested file-tree construction, BEP 52 roots and deduplicated required piece
   layers, prefix collision rejection, and shared metadata/output handling in
   `crates/btpc-core/src/create.rs`. Added independent reference-derived root and
   info-hash vectors plus single/multi-file, nested, empty, boundary, duplicate-root,
   deterministic, and collision tests in `crates/btpc-core/tests/v2_create.rs`.
   Updated `CREATE-V2-001` and `META-V2-001` traceability. Spec validation (15
   specs/71 requirements), Cargo fmt, strict workspace Clippy, workspace tests,
   doc tests/docs, Ruff, mypy, and 15 Python tests all passed.
   Notes:

22. [x] Build canonical hybrid torrent creation
   Claimed by: Codex implementer (2026-07-01 14:04 PDT)
   Context:
   Hybrid torrents must describe one payload through valid v1 and v2 structures,
   including v1 padding alignment required for interoperability.
   Implementation:
   Extend `Creator` with hybrid mode. Generate a consistent v1 file list and v2
   file tree, insert deterministic v1 padding files and attributes where required,
   produce v1 pieces and v2 piece layers, and return both info hashes. Reuse a
   single manifest and share I/O only if doing so does not obscure either oracle;
   a correct documented multi-pass implementation is acceptable initially.
   Tests and verification:
   Add independent golden fixtures first for files requiring no padding, one/many
   padding files, empty files, nested trees, and boundary sizes. Validate hybrid
   consistency on parse, independently verify both hash domains, and prove v1
   padding paths cannot collide with real payload paths.
   Evidence: Added `CreateMode::Hybrid`, a documented two-pass construction path,
   deterministic unique `.pad` v1 entries, reserved-path collision rejection,
   padded logical-stream SHA-1 hashing, v2 Merkle reuse, and dual info hashes in
   `crates/btpc-core/src/create.rs`. Added single-file, exact-boundary, empty/nested,
   repeated-padding, collision, parser-consistency, and independently reconstructed
   v1 piece tests in `crates/btpc-core/tests/hybrid_create.rs`. Updated
   `CREATE-HYBRID-001` traceability. Cargo fmt, strict workspace Clippy, workspace
   tests, doc tests/docs, Ruff, mypy, 15 Python tests, and spec validation (15
   specs/71 requirements) passed.
   Notes:

23. [x] Enable v2 and hybrid modes in CLI and Python
   Claimed by: Codex implementer (2026-07-01 14:10 PDT)
   Context:
   Core support is incomplete until both user surfaces expose identical mode and
   piece-length behavior.
   Implementation:
   Add `--mode v1|v2|hybrid` to CLI create with the documented default for the
   current release. Extend JSON/human results for one or both hashes. Extend Python
   `TorrentMode`, creation options, and result objects. Ensure v2 minimum piece
   validation and mode errors come from core and map consistently. Update README
   examples and API docs for all three modes.
   Tests and verification:
   Add CLI and pytest parity matrices for v1/v2/hybrid, explicit and automatic
   piece lengths, hashes, errors, deterministic bytes, progress, and cancellation.
   Run the complete verification gate.
   Evidence: Added CLI `--mode v1|v2|hybrid`, mode-aware JSON/human hashes, core
   piece-length validation, and README examples. Extended the private PyO3 binding
   and typed Python `CreateOptions`/`CreateResult` with mode and optional v1/v2
   hashes. Normalized v2 progress to payload-global monotonic events and preserved
   cancellation semantics. Added CLI and pytest matrices covering all modes,
   explicit/automatic policy parity, byte identity, hashes, errors, progress, and
   cancellation. Cargo fmt, strict workspace Clippy, workspace tests, doc tests/docs,
   Ruff, mypy, 23 Python tests, and spec validation (15 specs/71 requirements)
   passed.
   Notes:

24. [x] Implement payload verification in the core
   Claimed by: Codex implementer (2026-07-01 14:17 PDT)
   Context:
   A general-purpose torrent library needs to verify payloads, and end-to-end
   benchmarks need a mandatory correctness check independent of creation output.
   Implementation:
   Add configurable verification that safely maps torrent paths beneath a payload
   root, checks file presence/types/sizes, optionally reports extra files, and
   verifies v1 pieces, v2 file roots/piece layers, or both for hybrid torrents.
   Support fail-fast and collect-mismatch modes, deterministic mismatch ordering,
   cancellation, and progress. Refuse absolute/traversal paths and symlink escape
   by default. Reuse the sequential hashing oracles.
   Tests and verification:
   Write tests first for valid payloads, missing/extra/wrong-sized files, modified
   bytes at first/middle/last v1 pieces, v2 per-file mismatches, hybrid mismatch
   reporting, unsafe paths, symlink escape, fail-fast/all modes, cancellation, and
   deterministic reports.
   Evidence: Added `crates/btpc-core/src/verify.rs` with safe root-relative path
   mapping, symlink refusal, structural checks, optional extra-file enumeration,
   fail-fast/collect-all deterministic reports, cancellation/progress, bounded v1
   stream verification, and reused v2 Merkle verification. Added nine tests for
   valid v1/v2/hybrid payloads, missing/extra/wrong-size files, first/middle/last
   v1 mutations, v2 mutations, hybrid reporting, root/nested symlink escape,
   single-file roots, fail-fast, cancellation, and ordering. Marked all VERIFY
   requirements Implemented and mapped module ownership. Cargo fmt, strict workspace
   Clippy, workspace tests, doc tests/docs, Ruff, mypy, 23 Python tests, and spec
   validation (15 specs/71 requirements) passed.
   Notes:

25. [x] Expose verification in CLI and Python
   Claimed by: Codex implementer (2026-07-01 14:24 PDT)
   Context:
   Verification reports and exit behavior must be usable by scripts and Python
   applications without duplicating safe-path or hashing logic.
   Implementation:
   Add `btpc verify TORRENT PAYLOAD` with human and versioned JSON reports, progress,
   fail-fast/all controls, symlink/extra-file policies, and exit code 4 for payload
   mismatch. Add Python `Metainfo.verify` and top-level convenience APIs with typed
   report/mismatch objects, GIL release, callbacks, and cancellation.
   Tests and verification:
   Add process and pytest tests for the full core mismatch matrix, JSON schema,
   exception-versus-report semantics, exit codes, GIL release, callbacks, and
   cancellation. Run the complete verification gate.
   Evidence: Added `btpc verify TORRENT PAYLOAD` with fail-fast/extra-file controls,
   deterministic human output, `btpc.verify.v1` JSON, and frozen mismatch exit code
   6. Added native GIL-released verification, throttled callback/cancellation support,
   typed `Metainfo.verify`, top-level `verify`, report/mismatch types, and stable
   mismatch enums in Python. Added CLI and pytest coverage for all modes, valid and
   structural/hash mismatches, JSON, exit codes, extra files, fail-fast, symlinks,
   callbacks, callback exceptions, cancellation, and GIL release. Cargo fmt, strict
   workspace Clippy, workspace tests, doc tests/docs, Ruff, mypy, 26 Python tests,
   and spec validation (15 specs/71 requirements) passed.
   Notes:

26. [x] Add magnet generation across Rust, CLI, and Python
   Claimed by: Codex implementer (2026-07-01 14:33 PDT)
   Context:
   Magnet links are common general-purpose torrent functionality and exercise
   correct mode/hash handling.
   Implementation:
   Implement percent-encoded magnet generation with `btih` for v1, BEP 52 `btmh`
   multihash for v2, both topics for hybrid, optional display name, trackers, and
   web seeds in deterministic order. Expose Rust options, `Metainfo.magnet()` in
   Python, and `btpc magnet` that emits only the URI by default.
   Tests and verification:
   Add published/independent hash representation vectors, reserved/non-UTF-8 name
   cases, repeated trackers, all modes, deterministic query ordering, CLI stdout
   golden tests, and Python/Rust parity tests.
   Evidence: Added deterministic core magnet generation with v1 `btih`, v2
   `btmh:1220`, both hybrid topics, raw-byte percent encoding, and configurable
   display name/tracker/web-seed parameters. Added `btpc magnet` with URI-only
   stdout and typed `Metainfo.magnet()` in Python. Added independent hash-format,
   reserved/non-UTF-8 byte, ordering, omission, all-mode, CLI stdout, and
   Python/core parity tests. Cargo fmt, strict workspace Clippy, workspace tests,
   doc tests/docs, Ruff, mypy, 30 Python tests, and spec validation (15 specs/71
   requirements) passed.
   Notes:

27. [x] Add owned metainfo editing and common extensions
   Claimed by: Codex implementer (2026-07-01 14:41 PDT)
   Context:
   To approach torf-like general-purpose use, callers must modify metadata without
   rebuilding protocol dictionaries manually, while preserving unknown fields.
   Implementation:
   Add an owned `MetainfoBuilder` or mutation model that can start empty or from a
   parsed torrent. Support tracker tiers (BEP 12), web seeds (BEP 19), DHT nodes
   (BEP 5-compatible top-level form), private flag (BEP 27), source/comment/creator,
   creation date, file attributes/padding (BEP 47/52), and raw unknown fields.
   Enforce reserved-field ownership, canonical serialization, validation before
   output, and documented invalidation/recomputation of info hashes. Bind a safe
   Python editing API; CLI editing is not required unless separately planned.
   Tests and verification:
   Write tests first for each field, tracker tier order, unknown-field preservation,
   conflicting raw keys, parsed-to-owned conversion, top-level edits preserving
   info hashes, info edits changing hashes, canonical output, Python parity, and
   round trips through representative competitor torrents.
   Evidence: Added `MetainfoEditor` over owned canonical bencode with parsed or
   info-dictionary construction, tracker tiers, web seeds, nodes, comment/creator/
   date, private/source, v1/v2 file attributes, unknown-field preservation, and
   reserved-key enforcement. Edited output is serialized canonically and reparsed
   for validation; top-level edits preserve hashes while `info` edits recompute
   them. Added a safe typed Python `Metainfo.edit()` binding. Six Rust and two
   Python tests cover all fields, tier order, raw extensions, conflicts, hash
   behavior, canonical output, single/multifile attributes, and parity. Cargo fmt,
   strict workspace Clippy, workspace tests, doc tests/docs, Ruff, mypy, 32 Python
   tests, and spec validation (15 specs/71 requirements) passed.
   Notes:

28. [x] Create the reproducible competitor benchmark harness
   Claimed by: Codex implementer (2026-07-01 14:50 PDT)
   Context:
   Performance is a primary goal, but comparisons are meaningless without pinned
   tools, equivalent options, deterministic datasets, correctness validation, and
   cache-state labeling.
   Implementation:
   Under `benches/`, add deterministic dataset generation from a seed, manifests
   and checksums, adapters for native/Python BTPC, `mktorrent`, `mkbrr`, `torf`, and
   `torrenttools`, tool-version capture, randomized run order, repeated timing,
   CPU/elapsed/throughput/peak-RSS capture where portable, warm/cold cache labels,
   and machine metadata. Normalize options per supported mode and validate every
   output with BTPC plus an independent implementation. Emit raw JSON/CSV and a
   generated Markdown summary; never commit large datasets.
   Tests and verification:
   Add unit tests for command construction, result parsing, failed/unavailable
   competitor states, dataset reproducibility, semantic validation, and report
   generation. Run a tiny all-adapter smoke dataset; record exact installed tool
   versions and do not mark unavailable tools as successful.
   Evidence:
   - Added `benches/btpc_bench/` with deterministic seeded payload generation,
     streaming SHA-256/per-piece SHA-1 manifests, explicit native/Python BTPC,
     mkbrr, mktorrent, torf-cli, and torrenttools adapters, isolated process
     environments, randomized blocked rounds, psutil process-tree CPU/RSS
     sampling, machine metadata, BTPC plus independent bencode/piece validation,
     raw JSON/CSV/log/torrent retention, ASCII and Markdown summaries, and saved
     result render/compare commands. Added usage documentation in
     `benches/README.md`; `psutil==7.2.2` is resolved in `uv.lock`.
   - `uv run pytest tests/benchmarks/test_harness.py -q`: 13 passed, covering
     adapter commands/version contracts, unavailable/failed states, seeded block
     ordering, reproducible manifests, every-piece semantic validation, JSON
     round trips, report generation, CLI render/compare/generate, and a real
     native BTPC retained-torrent run.
   - Tiny all-adapter smoke command used a 98,304-byte seed-2801 dataset with
     16-KiB pieces and one measured round. Exact discovered versions/statuses:
     `btpc-native` = `btpc 0.1.0` success; `btpc-python` = `Python 3.14.3`
     success; both produced info hash `082ec99ec5b0...`. `mkbrr`, `mktorrent`,
     `torf-cli`, and `torrenttools` were not installed and were recorded as
     `unavailable`, never successful. Raw artifacts were written beneath the
     ignored `benchmark-results/todo-28-smoke/` directory.
   - Broad verification: `cargo fmt --all --check`; strict workspace Clippy;
     `cargo nextest run --workspace --all-features` (93 passed); Rust doc test
     (1 passed); `cargo doc`; Ruff check/format; strict mypy; combined Python and
     benchmark pytest (45 passed); and `scripts/check_specs.py` (15 specs, 71
     requirements) all passed.
   Notes:

29. [x] Add Rust microbenchmarks and baseline profiles
   Claimed by: Codex implementer (2026-07-01 14:29 PDT)
   Context:
   End-to-end comparisons identify outcomes; microbenchmarks and profiles identify
   causes. Establish baselines before changing concurrency or memory architecture.
   Implementation:
   Add Criterion benchmarks for bencode parse/encode, manifest sorting, v1 piece
   assembly/SHA-1, v2 block hashing/Merkle reduction, and file-tree construction.
   Add documented profiling commands appropriate to supported platforms and a
   baseline results template containing commit, toolchain, CPU, dataset, and
   configuration. Ensure benchmark code consumes outputs and avoids measuring
   fixture generation.
   Tests and verification:
   Compile all benchmarks, run reduced-duration smoke samples, and perform one
   representative end-to-end baseline for v1/v2/hybrid. Validate all generated
   torrents and record wall time plus peak RSS before any optimization todo.
   Evidence:
   - Added the MSRV-compatible `criterion 0.7.0` dev dependency and
     `crates/btpc-core/benches/core.rs`, covering bencode parse/encode, 2,048
     entry manifest sorting, sequential v1 piece hashing, sequential v2 Merkle
     hashing, and v2 many-file tree creation. Fixtures are built outside timed
     iteration; owned sort inputs use `iter_batched`; outputs use `black_box`.
   - `crates/btpc-core/tests/benchmark_inventory.rs` verifies all required
     benchmark names, setup/consumption safeguards, macOS/Linux profiling
     commands, and baseline template fields. `cargo bench -p btpc-core --bench
     core --no-run` compiled the optimized target successfully.
   - Reduced Criterion smoke (`--warm-up-time 0.05 --measurement-time 0.1
     --sample-size 10 --noplot`) ran every kernel: parse 112.74 us / 717.51
     MiB/s; encode 118.52 us / 682.53 MiB/s; sort 235.70 us / 8.6892
     Melem/s; v1 hashing 3.8823 ms / 2.0123 GiB/s; v2 hashing 4.2380 ms /
     1.8434 GiB/s; v2 file-tree creation 35.226 ms. These short samples are
     explicitly labeled diagnostic, not publication evidence.
   - Added `benches/profiling.md`, `benches/baseline-template.md`, and the dated
     `benches/baselines/2026-07-01-pre-optimization.md` report. The report records
     Rust/Cargo/Criterion versions, macOS 26.5 / Apple M3 Max / 38.65 GB / APFS,
     deterministic seed-2901 8-MiB dataset SHA-256, commands, cache label,
     sampler limitations, raw ignored artifact paths, and absent Git metadata.
   - Five warm-cache release runs per mode produced median wall times v1 42.632
     ms, v2 42.492 ms, hybrid 42.435 ms, with maximum observed RSS 7,340,032,
     7,028,736, and 7,307,264 bytes respectively. All 15 outputs passed both
     `btpc validate` and `btpc verify`; hashes and output sizes are recorded in
     the dated report.
   - Broad verification: rustfmt; strict workspace Clippy; Nextest 94/94;
     Rust doc test 1/1; cargo doc; Ruff check/format; strict mypy; Python plus
     benchmark pytest 45/45; and spec validation (15 specs, 71 requirements) all
     passed.
   Notes:

30. [x] Implement bounded parallel v1 hashing
   Claimed by: Codex implementer (2026-07-01 14:32 PDT)
   Context:
   The sequential oracle is correct but may underuse CPU on fast storage. Parallel
   hashing must retain ordered pieces and bounded memory, and it must prove benefit
   rather than assume it.
   Implementation:
   Profile first. If hashing is material, add a per-operation bounded pipeline with
   reusable buffers, sequence numbers, configurable worker count, backpressure,
   ordered collection, cancellation, and safe shutdown on errors. Do not configure
   a global thread pool. Preserve an explicit single-thread path and automatic
   heuristic. If profiling shows no likely benefit, document evidence and append a
   revised optimization todo targeting the measured bottleneck instead of adding
   speculative complexity.
   Tests and verification:
   Differentially test sequential and parallel outputs with property-generated
   manifests/options, forced out-of-order completion, worker failures, bounded
   queue sizes, cancellation, and thread counts. Benchmark before/after on single
   large file and mixed tree; report throughput and peak RSS with statistical
   context and require no correctness change.
   Evidence:
   - Added `HashThreads::{Automatic, Exact}` and `ParallelHashOptions` in
     `btpc-core`. `Exact(1)` retains the sequential oracle; automatic uses two
     workers on multi-core hosts and falls back to sequential on one core.
     Explicit counts create only scoped per-operation threads and no global pool.
   - Implemented a single ordered reader, sequence-numbered complete-piece jobs,
     bounded sync-channel backpressure, reusable piece buffers, scoped SHA-1
     workers, ordered collection, monotonic progress, cancellation propagation,
     and joined shutdown. Payload buffer memory is bounded by `(workers + queue
     capacity) * piece length` plus a 64-KiB read buffer.
   - Core tests cover generated multifile differential parity, cross-file pieces,
     one-slot queues, option bounds, forced out-of-order completion, injected
     worker I/O failure, preflight and mid-flight cancellation, conservative
     automatic selection, and progress ordering. CLI `--threads 0/1/N` and Python
     `CreateOptions(threads=0/1/N)` are wired to the same core policy; adapter
     tests prove byte-for-byte and info-hash parity.
   - Criterion v1 kernel comparison (15 samples) measured sequential 3.8835 ms /
     2.0117 GiB/s versus four-worker bounded hashing 1.1553 ms / 6.7624 GiB/s,
     a 3.36x kernel speedup. End-to-end four-worker trials showed no stable gain
     over two workers and about 16 MiB additional RSS, motivating the two-worker
     automatic heuristic.
   - Ten randomized blocked release rounds on deterministic 128-MiB datasets,
     with every output byte-identical and passing `btpc validate` plus `btpc
     verify`, measured: single file 101.646 ms sequential vs 74.242 ms two-worker
     median (1.37x, 1,259 vs 1,724 MiB/s); 256-file tree 129.363 ms vs 74.995 ms
     (1.72x, 989 vs 1,707 MiB/s). Peak RSS rose from 11.4/12.2 MiB to 24.1/24.8
     MiB. Full distributions, MAD/CV, caveats, design, and raw artifact paths are
     recorded in `benches/baselines/2026-07-01-v1-parallel.md`.
   - Broad verification: rustfmt; strict workspace Clippy; Criterion bench
     compilation; Nextest 103/103; Rust doc test 1/1; cargo doc; Ruff
     check/format; strict mypy; Python plus benchmark pytest 46/46; and spec
     validation (15 specs, 71 requirements) all passed.
   Notes:

31. [x] Optimize v2 and hybrid hashing from profiles
   Claimed by: Codex implementer (2026-07-01 14:42 PDT)
   Context:
   v2 performs SHA-256 Merkle work per file and hybrid may perform extra I/O.
   Optimization should target measured costs while keeping the Merkle oracle and
   hybrid invariants understandable.
   Implementation:
   Profile v2 and hybrid creation across large-file and many-file datasets. Apply
   the smallest measured improvement: bounded per-file concurrency, shared hybrid
   reads, buffer pooling, reduced piece-layer allocations, or another evidenced
   bottleneck. Maintain configurable execution, deterministic output, bounded
   descriptors/memory, and oracle fallback. Do not combine algorithms merely to
   reduce code if it weakens differential testing.
   Tests and verification:
   Add differential/property tests for every optimized path, stress descriptor and
   memory bounds, force out-of-order/error/cancel behavior, and re-run all v2/hybrid
   conformance fixtures. Report before/after Criterion and end-to-end results plus
   peak RSS; explain any regression greater than the documented gate.
   Evidence:
   - Phase profiling on deterministic 128-MiB inputs showed median hash time of
     68 ms v2 / 131 ms hybrid for one file and 86 ms v2 / 547 ms hybrid for a
     256-file tree, identifying many-file concurrency and hybrid alignment work
     as the smallest useful targets.
   - Added bounded file-level concurrency for v2 and hybrid many-file creation.
     V2 workers retain `hash_v2_file_sequential` as the Merkle oracle; hybrid
     workers hash independent real-file-plus-padding domains. Both restore
     manifest order, preserve deterministic bytes/hashes/padding offsets, emit
     ordered live progress, prefer originating I/O errors over cancellation, and
     cap active file descriptors at the selected worker count. One-file manifests
     stay sequential because profiling showed no material benefit.
   - Tests cover fixed v2/hybrid oracle parity, 32 property-generated trees,
     many-padding-file parity, forced out-of-order completion, missing-file worker
     errors, preflight and callback-driven mid-flight cancellation, ordered
     progress, and real-path active-worker/descriptor bounds. All prior v2 Merkle
     and hybrid conformance fixtures pass unchanged.
   - Criterion many-file comparison (10 samples) measured v2 36.523 ms sequential
     vs 25.217 ms two-worker (1.45x) and hybrid 88.355 ms vs 59.035 ms (1.50x).
   - Ten randomized end-to-end rounds per dataset/mode, with every torrent
     byte-identical and passing `btpc validate` plus `btpc verify`, measured:
     256-file v2 128.016 ms vs 101.086 ms median (1.27x) with 8.19 vs 8.24 MiB
     peak RSS; 256-file hybrid 606.614 ms vs 325.687 ms (1.86x) with 16.74 vs
     17.53 MiB peak RSS. Single-file v2 was -1.45% and hybrid -0.01%, below the
     documented 5% release gate and automatically kept on the sequential path.
     Full distributions, CV/MAD, design, caveats, and raw paths are recorded in
     `benches/baselines/2026-07-01-v2-hybrid-parallel.md`.
   - Broad verification: rustfmt; strict workspace Clippy; Criterion bench
     compilation; Nextest 111/111; Rust doc test 1/1; cargo doc; Ruff
     check/format; strict mypy; Python plus benchmark pytest 46/46; and spec
     validation (15 specs, 71 requirements) all passed.
   Notes:

32. [x] Optimize parsing, traversal, and serialization from profiles
   Claimed by: Codex implementer (2026-07-01 14:49 PDT)
   Context:
   Many-small-file workloads may be dominated by traversal, sorting, allocation,
   or bencode construction rather than hashing.
   Implementation:
   Profile the 100,000-small-file analogue and inspection of large metainfo. Apply
   focused improvements such as allocation reuse, direct writer serialization,
   borrowed key lookup, metadata syscall reduction, or an evaluated parallel walker
   while retaining deterministic ordering and policies. Any dependency added must
   outperform the simpler implementation and respect MSRV/cross-platform support.
   Tests and verification:
   Preserve all property/golden tests, add differential tests around the changed
   path, and benchmark parse/create traversal separately and end to end. Record
   allocations or peak RSS where tooling supports it and compare against baseline.
   Evidence:
   - Profiled a deterministic 10,000-file analogue (100 directories, 32-byte
     files) and a 2,094,370-byte hybrid metainfo. Before optimization, median scan
     times were 60 ms v1, 57 ms v2, and 62 ms hybrid; hashing still dominated,
     while serialization peaked at 16 ms and inspect median was 30 ms with 70.25
     MiB maximum RSS.
   - Removed two measured default traversal costs without adding dependencies or
     a parallel walker: regular files now reuse the walker's already-read metadata
     snapshot and retain one fresh snapshot for mutation detection, and scans skip
     lossy joined match-path construction entirely when include/exclude glob sets
     are empty. Filtered and unfiltered parity is covered by a new regression test
     plus all existing policy/determinism/property tests.
   - Five-run post-change medians reduced scan time to 47 ms v1 (-21.7%), 46 ms
     v2 (-19.3%), and 46 ms hybrid (-25.8%). Hybrid inspect remained 30 ms while
     maximum RSS fell to 69.32 MiB (-1.3%). Direct inspection did not demonstrate
     a parser/owned-snapshot bottleneck, so parser restructuring was intentionally
     rejected as speculative complexity.
   - Added the separate `manifest_scan` Criterion kernel; its 2,048-file smoke
     estimate is 74.475 ms / 27.499 K files/s. The benchmark inventory, profiling
     rationale, before/after table, RSS context, and raw ignored artifact paths are
     recorded in `benches/baselines/2026-07-01-traversal-inspection.md`.
   - Broad verification: rustfmt; strict workspace Clippy; Criterion bench
     compilation; Nextest 112/112; Rust doc test 1/1; cargo doc; Ruff
     check/format; strict mypy; Python plus benchmark pytest 46/46; and spec
     validation (15 specs, 71 requirements) all passed.
   Notes:

33. [x] Harden interoperability, fuzzing, and mutation safety
   Claimed by: Codex implementer (2026-07-01 14:52 PDT)
   Context:
   Before a stable release, BTPC needs a broad corpus from real tools and sustained
   adversarial testing beyond hand-authored examples.
   Implementation:
   Add documented fixtures produced by pinned competitor versions for all modes
   they support, malformed regression fixtures, and prior fuzz crashers. Extend
   fuzzing to typed metainfo conversion, validation, canonical serialization, and
   magnet generation. Add filesystem mutation stress tests during scan/hash and
   parser tests near every resource limit. Document whether each irregular fixture
   is rejected, accepted with warning, or preserved.
   Tests and verification:
   Run the complete fixture corpus through Rust, CLI, and Python. Run each fuzz
   target for a meaningful bounded duration locally or attach scheduled CI evidence.
   Confirm no panics, hangs, unbounded allocations, or silent normalization of
   invalid protocol data.
   Evidence:
   - Added `tests/fixtures/interoperability/manifest.tsv` and README with 14
     fixtures: pinned mktorrent 1.1, mkbrr 1.23.0, torf-cli 5.2.1, and
     torrenttools 0.6.2 outputs across every supported v1/v2/hybrid mode, plus
     malformed, prior-corpus, noncanonical-info, and unsorted-file-tree cases.
     The manifest records disposition, expected hashes, payload checksum, and
     reason; `scripts/generate-interop-fixtures.sh` retains reproducible commands.
   - Added shared corpus tests in `crates/btpc-core/tests/interoperability.rs`,
     `crates/btpc-cli/tests/interoperability.rs`, and
     `tests/python/test_interoperability.py`; all 14 fixtures passed each public
     surface, including exact original-byte/hash preservation and declared rejects.
   - Added fuzz targets `metainfo`, `metainfo_roundtrip`, and `magnet`, synced all
     fixtures into all five target corpora, and extended weekly CI to run each for
     300 seconds. Local bounded runs completed without product failures: parse
     1,421,004 executions/20s, canonical roundtrip 1,345,282/20s, metainfo
     1,293,693/20s, magnet 1,710,014/20s, and corrected metainfo roundtrip
     1,852,507/30s. Two oracle findings were promoted to regression fixtures and
     now verify canonicalization semantics without changing original hashes.
   - Added `crates/btpc-core/tests/mutation_stress.rs` for concurrent scan mutation,
     same-length v1/v2 hash mutation, and stable recovery, plus zero/exact/one-over
     depth, item, byte-string, input, and owned-allocation limits in
     `bencode_parser.rs`; focused tests passed (7 parser, 3 mutation).
   - Full verification passed: Rust/fuzz rustfmt; strict workspace and fuzz clippy;
     Criterion no-run; Nextest 118/118; doctest 1/1; cargo doc; Ruff check/format;
     mypy; Python/benchmark pytest 47/47; spec checker 15 specs/71 requirements;
     and `cargo +nightly fuzz build`.
   Notes:
   - Noncanonical but parseable metainfo is intentionally inspectable and preserves
     source bytes/info hashes; canonical serialization can change encoding order or
     minimal integer form. The fuzz oracle compares semantic content and canonical
     idempotence rather than requiring source and canonical info hashes to match.

34. [x] Complete user and API documentation
   Claimed by: Codex implementer (2026-07-01 15:35 PDT)
   Context:
   The product now spans three surfaces and multiple protocol modes. Users need
   examples, compatibility boundaries, and benchmark methodology before release.
   Implementation:
   Expand README and `specs/` with installation, quick starts, CLI reference,
   Python and Rust examples, v1/v2/hybrid explanations, reproducibility controls,
   path/byte handling, verification, progress/cancellation, errors/exit codes,
   performance methodology, supported platforms/Python versions/MSRV, and explicit
   non-goals. Generate shell completions and man-page/reference artifacts from the
   CLI definition where practical. Ensure examples are executable tests.
   Tests and verification:
   Run doctests, README/example tests, CLI help snapshot tests, link checking, and
   a clean-environment walkthrough that installs a wheel, creates each mode,
   inspects it, emits a magnet, and verifies its payload.
   Evidence:
   - Replaced the short README with installation and three-surface quick starts,
     v1/v2/hybrid comparison, deterministic creation controls, byte/path handling,
     verification, progress/cancellation, structured errors/exit codes, supported
     Rust/OS/Python matrix, benchmark methodology, non-goals, and documentation map.
   - Added `docs/cli.md`, `docs/python-api.md`, `docs/rust-api.md`, and
     `docs/compatibility.md`; updated CLI/Python/Rust/release specs with three new
     implemented documentation requirements. Spec validation passed for 15 specs
     and 74 requirements.
   - Added `btpc completions` and `btpc manpage` backed by Clap, generated and
     committed Bash/Zsh/Fish/PowerShell/Elvish completions, `btpc(1)`, and help
     references. `crates/btpc-cli/tests/reference.rs` confirms generation and
     byte-for-byte staleness for every artifact; `scripts/generate-cli-reference.sh`
     and `make docs-generate` reproduce them.
   - Added executable guide tests: Rust hybrid create/parse/magnet/verify,
     CLI create/inspect/validate/magnet/verify for all three modes, and Python
     create/read/magnet/verify for all modes. Focused tests passed, and full
     Nextest passed 122/122 with Rust doctest 1/1.
   - Added `scripts/check_docs.py` and `make docs`; local-link validation passed
     across 29 Markdown files. Strict rustdoc completed with `-D warnings`.
   - Built `btpc-0.1.0-cp311-cp311-macosx_11_0_arm64.whl`, created a fresh
     CPython 3.11 venv, installed only that wheel, and ran `scripts/smoke_wheel.py`:
     v1/v2/hybrid creation, installed-package inspection, magnets, canonical
     round trips, and payload verification all passed.
   - Full gate passed: Rust and fuzz rustfmt/clippy, Criterion no-run, Nextest
     122/122, doctest 1/1, strict cargo doc, Ruff check/format, mypy, pytest 48/48,
     specs 15/74, Markdown links 29 files, and nightly fuzz build.
   Notes:
   - BTPC remains pre-release; installation docs intentionally describe checkout
     builds and clean local wheels rather than claiming registry availability.

35. [x] Automate wheels, binaries, and release validation
   Claimed by: Codex implementer (2026-07-01 15:59 PDT)
   Context:
   Users need trustworthy PyPI wheels and native binaries, and release automation
   must test artifacts rather than only source checkouts.
   Implementation:
   Add release workflows for platform wheels using current Maturin guidance,
   source distributions where appropriate, native CLI archives, checksums, and
   artifact provenance/signing supported by the hosting platform. Test wheels in
   clean CPython environments and binaries on their target OS. Add changelog and
   versioning automation with one source-of-truth version strategy across Cargo
   and Python metadata. Keep publication gated/manual until credentials and package
   ownership are configured.
   Tests and verification:
   Build the full artifact matrix in dry-run or release-candidate mode, install
   each available wheel, run import/create/inspect/verify smoke tests, run native
   binary smoke tests, and verify package metadata/checksums. Attach CI evidence.
   Evidence:
   - Added manual `.github/workflows/release.yml` with dry-run-by-default matrices
     for CPython 3.11–3.14 wheels and native CLI archives on Linux x86-64/AArch64,
     macOS Intel/Apple Silicon, and Windows x86-64, plus an sdist. Each target
     installs/smoke-tests its wheel or binary before upload; actionlint 1.7.12
     validates all workflows.
   - Publication requires an explicit `publish` input and existing matching tag.
     It is otherwise disabled. The gated path uses a protected `pypi` environment,
     PyPI Trusted Publishing, a draft GitHub release, and GitHub build-provenance
     attestations with least-privilege job permissions.
   - Made Cargo `[workspace.package].version` the single source by switching
     `pyproject.toml` to dynamic versioning and removing redundant internal Cargo
     dependency versions. Added `scripts/check_version.py`, `scripts/set_version.py`,
     `make version VERSION=X.Y.Z`, and `CHANGELOG.md`; tag `v0.1.0`, Cargo,
     Python wheel metadata, runtime, and changelog validation passed.
   - Added portable `scripts/smoke_cli.py`, `scripts/smoke_wheel.py`,
     `scripts/package_cli.py`, and `scripts/verify_artifacts.py`. Validators check
     wheel name/version metadata, distinguish sdist from target-named CLI archives,
     inspect executable/README/changelog archive contents, validate per-archive
     checksums, and emit aggregate `SHA256SUMS`.
   - Local release-candidate dry run built and tested the available host artifacts:
     CPython 3.11 macOS arm64 wheel, Python sdist, and macOS arm64 native CLI.
     Installed-wheel and native-binary v1/v2/hybrid create/inspect/magnet/verify
     smokes passed; assembled candidate validation passed for four files including
     the per-archive checksum, with aggregate SHA-256 output.
   - Release requirements `RELEASE-VERSION-001` and `RELEASE-ARTIFACT-001` are now
     Implemented and traced by `tests/python/test_release.py` (3/3 passing).
     Final gates passed: strict Rust clippy, Ruff check/format, mypy, pytest 51/51,
     specs 15/74, links 29 files, version/tag check, artifact validation,
     actionlint, and nightly fuzz build. The preceding full Rust gate passed
     Nextest 122/122, doctest 1/1, strict rustdoc, and Criterion no-run.
   Notes:
   - The full cross-platform matrix is defined for GitHub-hosted native runners;
     only the current macOS arm64 host artifacts can be executed locally. Publishing
     remains intentionally manual until the PyPI project, trusted publisher, GitHub
     environment, and package ownership are configured.

36. [x] Run the release-candidate correctness and performance gate
   Claimed by: Codex implementer (2026-07-01 22:52 PDT)
   Context:
   The project should not claim high performance or stable support until the final
   artifacts pass protocol, interoperability, API, and benchmark gates together.
   Implementation:
   Freeze the intended release versions and generated artifacts. Run all CI jobs,
   fixture/interoperability suites, wheel/binary clean installs, scheduled fuzz
   targets, and the full competitor benchmark matrix on a documented machine.
   Review public API/CLI schemas against `specs/rust-api.md`, `specs/python-api.md`,
   and `specs/cli.md`; resolve deviations
   with explicit spec changes and appended todos rather than undocumented behavior.
   Produce a release report containing correctness evidence, limitations, raw
   benchmark data, median/dispersion/peak RSS, cache conditions, and competitor
   versions. Avoid broad “fastest” claims not supported across the matrix.
   Tests and verification:
   Run the complete minimum gate plus every release workflow in artifact-testing
   mode. Validate every benchmark output. Require zero unresolved correctness
   failures, zero unexplained >5% regressions from the latest accepted baseline,
   and documented disposition for all warnings before marking complete.
   Evidence:
   Local host gate evidence and the complete candidate disposition are recorded in
   `benches/releases/2026-07-01-0.1.0-rc.md`. The local gate passed 122/122
   nextest cases, all standard Rust tests and docs, 38/38 Python tests, 14/14
   benchmark-harness tests, 15 specifications/74 requirements, actionlint,
   cargo-deny, artifact verification, clean CPython 3.11 wheel installation, five
   60-second fuzz targets, the standard competitor matrix, and the Criterion
   regression gate. Final macOS arm64 artifact hashes and raw benchmark paths are
   frozen in the report.
   - Hosted PR CI passed every required Rust, Python, dependency-policy, wheel,
     coverage, and Rust/Python CodeQL check on pull request #1:
     `https://github.com/burritothief/btpc/pull/1`.
   - The non-publishing release workflow passed all 30 jobs, including CPython
     3.11-3.14 wheels for Linux x86_64/aarch64, macOS x86_64/aarch64, and Windows
     x86_64; five native CLI archives; the Python sdist; source archive; public API
     compatibility; clean artifact validation; and aggregate checksums. The
     34,851,118-byte assembled candidate is retained as
     `release-candidate-0.1.0`:
     `https://github.com/burritothief/btpc/actions/runs/28567226819`.
   - The complete five-target hosted fuzz campaign ran for 26 minutes without a
     crash after building each target with nightly Rust and ASan on GNU libc:
     `https://github.com/burritothief/btpc/actions/runs/28567529151`.
   Notes:
   - Publishing remained disabled (`publish=false`); attestation, PyPI upload, and
     draft-release jobs were correctly skipped. Todo 59 resolved the prior Cargo
     license warning: workspace metadata and all packaged artifacts use SPDX `MIT`.

37. [x] [Review] Correct the BEP 52 Merkle padding contract before v2 implementation
   Claimed by: Codex implementer (2026-07-01 15:33 PDT)
   Context:
   `specs/metainfo.md` must define whether the final 16 KiB block is zero-padded
   before hashing and that trees are padded with zero hashes. BEP 52 instead hashes
   the final block at its actual byte length and pads incomplete trees with the
   recursively derived SHA-256 hashes of empty subtrees at the corresponding
   layer. Implementing todos 10, 20, 21, 22, or 24 from the current wording would
   produce incorrect roots and piece layers for most non-aligned files.
   Implementation:
   Amend the protocol contract and every affected implementation/test description
   before v2 code is written. Define a single core Merkle primitive with explicit
   leaf length, layer width, and empty-subtree hash semantics; use it for file
   roots, piece layers, creation, validation, and verification. Keep a simple
   independent reference implementation in tests so optimized code is not its own
   oracle. Document that files of length zero have no `pieces root`, files no
   larger than one piece have no `piece layers` entry, and piece-layer values are
   ordered piece roots with the exact BEP 52 length.
   Tests and verification:
   Add authoritative vectors and independently calculated cases for lengths 0, 1,
   16 KiB - 1, 16 KiB, 16 KiB + 1, one piece - 1, one piece, one piece + 1, and
   non-power-of-two block/piece counts. Differentially test creation, parsing, and
   verification against the reference primitive and at least one independent
   implementation or published fixture. Run all v2/hybrid core tests and strict
   clippy.
   Evidence:
   `specs/metainfo.md` and `specs/creation.md` now explicitly require hashing the
   final block at its actual byte length, recursive empty-subtree hash padding,
   no root for empty files, no piece-layer entry for files no larger than one
   piece, and exact ordered layer lengths. The independent `full_tree` oracle in
   `crates/btpc-core/tests/v2_merkle.rs` now drives
   `bep52_boundaries_agree_across_creation_parsing_and_verification` across lengths
   0, 1, 16,383, 16,384, 16,385, 32,767, 32,768, 32,769, and 81,921 bytes. The
   test initially failed because creation emitted an empty top-level `piece layers`
   dictionary and parsing required that field; `Creator::build_v2` now omits empty
   layers and `V2Metainfo::from_raw` treats omission as an empty layer map while
   still rejecting missing required multi-piece layers. `cargo test -p btpc-core
   --test v2_merkle` passed 4/4, `v2_create` 5/5, `v2_metainfo` 4/4,
   `hybrid_create` 4/4, and `verify` 9/9. Strict btpc-core Clippy, 15 specs/74
   requirements, and 30 Markdown-link checks passed.
   Notes:
   The fixed-vector constants remain independently derived BEP 52 reference
   values; the property and boundary matrices use a deliberately simple test-only
   full-tree implementation rather than the streaming production accumulator.

38. [x] [Review] Separate syntax, canonicality, and protocol validation APIs
   Claimed by: Codex implementer (2026-07-01 15:35 PDT)
   Context:
   The plan calls for permissive parsing of legacy metainfo, strict canonical
   validation, protocol validation, warnings, and safe defaults, but the proposed
   `Metainfo::from_bytes` and `ParseOptions` contracts do not yet make those states
   or guarantees unambiguous. A public library must not let a caller accidentally
   treat syntax-only input as validated metainfo or confuse non-canonical input
   with protocol-invalid input.
   Implementation:
   Before todos 5-11 freeze public types, define a small typestate or clearly
   named result model: syntax parsing returns a raw document/view, protocol
   validation returns a validated `Metainfo`, and canonicality is a separate
   report/policy rather than an implicit side effect. Make safe convenience
   constructors validate syntax, limits, and protocol invariants by default.
   Represent compatibility warnings as structured, non-exhaustive values with
   field/offset context. Ensure canonical serialization is available only after
   validation or performs validation atomically before writing any bytes.
   Tests and verification:
   Add compile-tested public examples proving the default-safe path, explicit
   syntax-only inspection, canonicality reporting, warning handling, and failure
   without partially written output. Add tests showing malformed, non-canonical,
   protocol-invalid, and valid-with-warning inputs remain distinguishable in Rust,
   CLI exit mapping, and Python exception/report mapping.
   Evidence:
   Added public non-exhaustive `Canonicality` and `ValidationWarning` types.
   `RawMetainfo::canonicality` reports parseable source canonicality without
   protocol validation, while `Metainfo::from_bytes` remains the default-safe
   syntax/resource/protocol constructor and `ValidationReport::canonicality`
   keeps canonicality independent from validity. Structured warnings now expose
   message, optional field, and optional source offset while retaining the legacy
   string list. A tolerated empty top-level `piece layers` dictionary produces a
   real structured warning with field and offset context. CLI inspect/validate
   JSON and Python `ValidationReport` expose additive canonical state; malformed
   syntax and protocol-invalid input retain distinct Rust categories, CLI exit
   mapping, and Python exception classes. `public_api` passed 6/6, `raw_metainfo`
   4/4, CLI inspect 4/4, focused Python metainfo/interoperability 7/7, and the
   btpc-core doctest passed. Workspace strict Clippy, Ruff, mypy, 15 specs/74
   requirements, and 30 Markdown-link checks passed.
   Notes:
   Canonical serialization remains available only on a successfully constructed
   validated `Metainfo`; the new test proves non-canonical valid source serializes
   to canonical bytes. No partial file-writing path is introduced by these APIs.

39. [x] [Review] Design byte-safe owned path and text APIs across Rust and Python
   Claimed by: Codex implementer (2026-07-01 15:38 PDT)
   Context:
   The protocol stores names, paths, trackers, and extension text as byte strings,
   while Python users expect `str`/`pathlib.Path` ergonomics. The current plan says
   to expose raw bytes plus validated UTF-8 views but does not define equality,
   ordering, conversion, display, serialization, or behavior on Unix versus
   Windows. Deferring these decisions risks lossy decoding and incompatible public
   APIs after parsers and builders are implemented.
   Implementation:
   Define compact public value types for torrent bytes/components/paths and
   decoded text views before todos 9-19. Preserve raw bytes as identity; make UTF-8
   decoding explicit and fallible; never use replacement decoding for round trips.
   Specify platform-path ingestion separately from torrent-path representation,
   including Unix non-UTF-8 names and the documented Windows encoding policy.
   Python APIs should return immutable typed objects with `.raw: bytes`, optional
   decoded access, sensible `repr`, hashing/equality based on raw identity, and
   explicit conversion to `pathlib.Path` only when safe. Use native generic
   parameter conventions such as `impl AsRef<Path>` and `impl AsRef<[u8]>` where
   ownership need not be imposed.
   Tests and verification:
   Add Rust and Python API tests for valid UTF-8, invalid UTF-8, separators, NUL,
   dot components, Unicode normalization lookalikes, equality/order, repr, pickle
   policy, and lossless parse-edit-serialize round trips. Run docs, mypy, pytest,
   and cross-platform filesystem tests.
   Evidence:
   Added public `TorrentBytes` and `TorrentPath` value types with equality,
   hashing, and ordering based exclusively on raw bytes; UTF-8 access is explicit
   and fallible. `TorrentPath::new` rejects empty, dot, traversal, separator, and
   NUL components, and `to_path_buf` is lossless: Unix preserves bytes while
   non-Unix platforms reject non-UTF-8 components. Manifest ingestion no longer
   uses `to_string_lossy` on non-Unix platforms, and verification uses the same
   safe conversion. Existing byte-slice accessors remain for compatibility;
   `TorrentFile::torrent_path` adds the typed view. Python now exposes frozen,
   ordered, hashable, pickle-compatible `TorrentBytes`/`TorrentPath` wrappers with
   raw repr, optional decoded text, safe `Path` conversion, and matching unsafe-
   component rejection. Rust `public_api` passed 7/7, manifest 6/6 on macOS,
   non-UTF-8 metainfo validation passed, verify 9/9, and focused Python metainfo/
   create passed 20/20. Workspace strict Clippy, Ruff, mypy, docs, 15 specs/74
   requirements, and 30 Markdown-link checks passed.
   Notes:
   Linux/Android retain the existing real-filesystem invalid-byte filename test;
   macOS APFS rejects the attempted `0xff` filename with `EILSEQ`, so portable raw
   conversion is tested independently there. Unicode normalization is intentionally
   not performed: composed and decomposed byte sequences remain unequal identities.

40. [x] [Review] Establish a stable public API evolution and feature boundary
   Claimed by: Codex implementer (2026-07-01 15:40 PDT)
   Context:
   `btpc-core` is intended as an open-source Rust library, but the workspace does
   not yet specify crate features, re-export policy, dependency exposure, public
   API semver checks, or which low-level bencode types are stable. With many todos
   adding public items, accidental dependency types and unnecessary surface area
   can become hard to remove even before 1.0.
   Implementation:
   Before exposing the inspection API, document the intended stable facade and
   keep implementation modules private by default. Re-export only purpose-built
   BTPC value types; avoid leaking PyO3, CLI, hashing, URL, glob, executor, or error
   dependency types from signatures. Define minimal opt-in features only where
   they materially reduce dependencies or platform requirements, and test both
   default and no-default-feature builds if features exist. Add automated public
   API diffing (for example cargo-semver-checks or a rustdoc JSON baseline) at
   release boundaries, plus doctests that compile as an external consumer.
   Tests and verification:
   Add an integration crate or compile tests importing only documented facade
   paths. Run `cargo check -p btpc-core --no-default-features`, all-feature checks,
   docs with warnings denied, and the selected API-diff tool against the accepted
   baseline. Confirm `cargo tree -p btpc-core` contains no adapter dependencies.
   Evidence:
   `btpc-core` now declares an explicit empty default feature set and documents
   the supported crate-root/module facade, dependency-type boundary, and additive
   feature policy in `docs/rust-api.md` and `specs/rust-api.md`. Added the isolated
   `tests/rust-consumer` crate, which imports only documented facade paths with
   `default-features = false`. Added `scripts/check_rust_api.sh` and pinned
   `cargo-semver-checks 0.42.0` automation: pull requests compare against their
   base SHA and release workflows compare against the previous version tag when
   one exists. Default/no-default/all-feature checks passed, the external consumer
   built, strict workspace Clippy passed, and rustdoc with `-D warnings` passed for
   both feature modes. `actionlint`, 15 specs/74 requirements, and 30 Markdown-link
   checks passed. A copied current source baseline produced a zero-diff
   `cargo-semver-checks check-release` smoke pass.
   Notes:
   This checkout has no Git history, so a historical baseline comparison cannot
   run locally; CI/release automation supplies the required revisions. The newest
   cargo-semver-checks requires Rust 1.91, so version 0.42.0 is intentionally pinned
   as the latest tested tool compatible with the current Rust 1.85 development
   environment. `cargo tree -p btpc-core` contains only `globset`, `sha1`, `sha2`,
   and their protocol/runtime dependencies, with no CLI or PyO3 adapter dependency.

41. [x] [Review] Replace duplicated package versions with a verified release source
   Claimed by: Codex implementer (2026-07-01 15:45 PDT)
   Context:
   The version is currently repeated independently in workspace Cargo metadata and
   `pyproject.toml`, while the Python import test hard-codes `0.1.0`. This will drift
   on the first release bump and makes the existing smoke test validate duplication
   rather than package metadata correctness.
   Implementation:
   Adopt one release source of truth before adding more artifacts. Prefer Maturin's
   supported dynamic-version path from Cargo metadata, or add a documented release
   tool that updates and verifies every required location atomically. Derive the
   runtime extension version from package metadata as now, expose the public Python
   version using standard package metadata semantics, and avoid hard-coded expected
   versions in ordinary tests. Keep an explicit release validation that compares
   Cargo metadata, built wheel metadata, `importlib.metadata.version("btpc")`, the
   native module, CLI `--version`, and archive names.
   Tests and verification:
   Build an sdist/wheel and CLI binary, inspect their metadata, install the wheel in
   a clean environment, and assert all reported versions agree. Add a simulated
   version-bump test or release-script dry run proving no tracked version location
   is missed. Run Rust and Python packaging smoke tests.
   Evidence:
   Cargo workspace metadata remains the single release source; `pyproject.toml`
   uses Maturin dynamic versioning and Python import tests now compare
   `btpc.__version__`, the native module, and `importlib.metadata.version("btpc")`
   without a hard-coded expected value. `scripts/check_version.py` verifies every
   workspace package version, both internal `btpc-core` path dependency
   requirements, dynamic Python metadata, changelog section, and optional tag.
   `scripts/set_version.py` updates the workspace and both internal requirements
   atomically; its `--dry-run 9.8.7` listed exactly `Cargo.toml`,
   `crates/btpc-cli/Cargo.toml`, and `crates/btpc-python/Cargo.toml`. Artifact tests
   derive fixture names from Cargo. `scripts/verify_artifacts.py` now optionally
   compares wheel metadata, installed distribution metadata, public/native Python
   versions, CLI `--version`, and archive names in one run. Fresh CPython 3.11
   wheel, sdist, and macOS arm64 CLI archive built and smoke-tested; all reported
   `0.1.0`, artifact verification passed, and checked-in CLI references matched
   the binary. Focused release/import tests passed 5/5, plus strict Clippy, Ruff,
   mypy, version/spec/docs checks, and actionlint.
   Notes:
   Generated manpage version text is intentionally derived from the CLI binary;
   `crates/btpc-cli/tests/reference.rs` detects stale generated references. Cargo
   still warns that package license metadata is missing during sdist creation;
   that pre-existing publishing warning remains separately tracked by the release
   gate report.

42. [x] [Review] Make the declared toolchain policy match modern-development goals
   Claimed by: Codex implementer (2026-07-01 15:47 PDT)
   Context:
   `rust-toolchain.toml` pins development to the MSRV Rust 1.85.0 even though the
   project explicitly asks to use current stable Rust and separately test the MSRV.
   That prevents routine use of newer stable diagnostics, Cargo behavior, and
   library niceties while also making the CI `stable` lane less representative.
   Dependency lower bounds are similarly broad and should be refreshed deliberately
   rather than treated as proof that the newest supported tooling is exercised.
   Implementation:
   Pin local development/CI formatting to an explicitly reviewed current stable
   toolchain (or document a controlled stable-channel update policy), retain
   `rust-version = "1.85"` as the compatibility floor, and run a distinct MSRV job
   with minimal dependency versions compatible with that floor where practical.
   Add a scheduled dependency/toolchain refresh using `cargo update`, `uv lock
   --upgrade`, audit/deny checks, and tests; do not automatically merge updates.
   Document which toolchain owns formatting to avoid MSRV rustfmt drift.
   Tests and verification:
   Run the complete Rust gate on the chosen current stable toolchain and `cargo
   check`/targeted tests on 1.85.0. Verify lockfiles build in both lanes and record
   exact toolchain versions. Run the Python gate from a freshly upgraded lock in a
   review branch or scheduled workflow.
   Evidence:
   `rust-toolchain.toml` now pins reviewed current stable Rust 1.94.1 as the
   development, formatting, Clippy, docs, platform, Python-build, and release
   toolchain; `rust-version = "1.85"` remains the compatibility floor.
   `CONTRIBUTING.md` documents rustfmt ownership and the separate MSRV policy. CI
   pins both versions explicitly. The MSRV lane runs nightly Cargo's
   `-Z direct-minimal-versions` resolver and checks with Rust 1.85.0. Added the
   read-only weekly/manual `dependency-refresh.yml`, which runs `cargo update`,
   `uv lock --upgrade`, deny/Clippy/Rust/Python tests, and uploads the diff without
   committing or merging. Rust 1.94.1 strict Clippy exposed and resolved three new
   diagnostics; its full gate passed 126/126 nextest cases, doctests, docs with
   warnings denied, consumer build, cargo-deny, specs/docs, and actionlint. An
   isolated direct-minimal lock passed Rust 1.85.0 workspace all-target checks and
   7/7 public API tests. An isolated freshly upgraded uv lock built the extension
   and passed Ruff, mypy, and 41/41 Python tests.
   Notes:
   The direct-minimal resolver currently changed zero locked packages because the
   committed constraints already resolve to compatible minima. The new empty
   `piece layers` compatibility warning required three fixture rows to declare
   warning expectations explicitly; the complete interoperability corpus then
   passed. Toolchain upgrades remain human-reviewed changes rather than automatic
   commits.

43. [x] [Review] Strengthen test architecture with compile-fail and model-based cases
   Claimed by: Codex implementer (2026-07-01 15:52 PDT)
   Context:
   The queue has strong unit, fixture, property, fuzz, CLI, and Python coverage,
   but it does not explicitly test invalid Rust API usage, parser/model agreement,
   state-machine behavior during edits, or deterministic failure under injected
   I/O faults. These are high-value corner cases for a new public library.
   Implementation:
   Add compile-fail tests for lifetime misuse, invalid builder state transitions,
   non-sendable callback boundaries if applicable, and APIs intentionally hidden
   from the facade. Build a small independent bencode/metainfo test model used only
   for generated differential tests. Add stateful property tests that sequence
   top-level edits, info edits, serialization, hashing, and validation while
   checking original-byte and hash invalidation rules. Introduce injectable readers
   and writers for short reads, interruptions, mid-stream errors, and partial-write
   failures without complicating the production API.
   Tests and verification:
   Run compile-fail tests, bounded state-machine/property suites, fault-injection
   tests, and the normal core gate. Demonstrate failures preserve error category and
   context, never emit a falsely successful partial artifact, and never leave a
   claimed atomic output path corrupted.
   Evidence:
   Added pinned `trybuild` dev coverage with compile-fail cases proving a borrowed
   `RawMetainfo` cannot outlive input bytes and the private `error` module cannot
   bypass the documented crate-root facade. Added a test-only independent bencode
   encoder and v1 metainfo model; 128-case generated suites compare canonical
   bytes, typed fields, piece counts, and exact SHA-1 info hashes against
   production. Added a 64-case generated edit state machine sequencing top-level
   and info edits through serialization/reparse while checking canonical output,
   original-byte replacement, no-op/restoration behavior, and hash invalidation.
   Added injected interrupted-then-failing and zero-write writers; both retain the
   I/O category/source and never report false success. Existing atomic-write tests
   continue to prove destination preservation and no temp-file leaks. Compile-fail,
   edit 7/7, bencode canonical 6/6, and model 2/2 focused suites passed. Strict
   btpc-core Clippy, doctests, specs/docs, and the full core nextest gate passed
   108/108 tests.
   Notes:
   Non-sendable callback misuse is not applicable to the current public Rust
   facade because progress sinks are synchronous borrowed traits rather than
   stored cross-thread callbacks. Filesystem mutation/error injection already
   covers scan/hash races and atomic output; new writer faults cover short/zero/
   interrupted serialization without adding test hooks to production APIs.

44. [x] Add staged pre-commit hooks for Rust, Python, and repository hygiene
   Claimed by: Codex implementer (2026-07-01 20:43 PDT)
   Context:
   Contributors need an installed local quality gate that catches inexpensive
   problems before commit and runs the complete Rust/Python test gate before push.
   Running every test for every commit would make hooks slow enough to bypass, so
   hook stages must balance fast feedback with comprehensive verification. This
   todo follows the bootstrap tooling in `pyproject.toml`, `Makefile`, and
   `CONTRIBUTING.md` and must reuse the same commands as CI rather than create a
   second, divergent quality system.
   Implementation:
   Add `pre-commit` to the locked development dependency group and create a pinned
   `.pre-commit-config.yaml`. At the `pre-commit` stage, run maintained hygiene
   hooks for trailing whitespace, final newlines, merge-conflict markers, mixed
   line endings, oversized added files, private keys, and TOML/YAML/JSON syntax;
   run Ruff lint with safe fixes, Ruff formatting, and `cargo fmt --all --check`.
   At the `pre-push` stage, run whole-project checks that cannot safely operate on
   only changed files: strict mypy, strict workspace clippy, `cargo nextest` plus
   Rust doctests, pytest against the built/developed extension, and dependency
   policy checks. Use one or a small number of local wrapper commands so Cargo and
   Maturin setup are not repeated independently for every hook. Add a manual
   `pre-commit run --all-files` stage for documentation/link/action workflow checks
   that are useful but too expensive or platform-sensitive for each commit. Pin
   third-party hook revisions; avoid unreviewed language-specific mirror repos
   when a local command using the locked Rust/uv environment is clearer. Ensure
   hooks do not modify generated wheels, `target/`, benchmark outputs, fixtures,
   or lockfiles unexpectedly. Add `make install-hooks` and `make uninstall-hooks`
   (or equivalently named documented commands) that install both `pre-commit` and
   `pre-push` hook types, and document `SKIP`, emergency bypass behavior, required
   tools, first-run cost, and how to run each stage explicitly. CI must run
   `pre-commit run --all-files` so hook configuration itself cannot rot.
   Tests and verification:
   From a clean environment run the hook installation command and verify both
   `.git/hooks/pre-commit` and `.git/hooks/pre-push` are installed. Run
   `uv run pre-commit validate-config`, `uv run pre-commit run --all-files`, and
   `uv run pre-commit run --all-files --hook-stage pre-push`. Add temporary
   intentionally malformed files in a disposable test/worktree to prove hygiene,
   Ruff, formatting, mypy, clippy, Rust test, and Python test failures block the
   intended stage, then remove them without committing. Confirm the successful
   pre-push run executes the same substantive commands as the minimum verification
   gate and that a second unchanged run benefits from tool caches.
   Evidence:
   - Added pinned staged hooks in `.pre-commit-config.yaml`: maintained repository
     hygiene plus safe Ruff fixes/formatting and Rust formatting at commit time,
     the complete substantive quality gate through `scripts/run-hook-stage.sh`
     at pre-push, and repository/spec/documentation/workflow/reference checks at
     the manual stage. Added `make hooks-push` and `make hooks-manual` entry points.
   - Updated `CONTRIBUTING.md` with prerequisites, install/uninstall commands,
     explicit stage invocations, first-run/cache behavior, generated-file safety,
     `SKIP`, and emergency bypass guidance. CI now validates and runs all three
     hook stages so the configuration is exercised in a real Git checkout.
   - `uv run pre-commit validate-config` passed. The manual wrapper passed spec
     validation (15 specifications/74 requirements), documentation links (30
     Markdown files), CLI reference tests (2/2), actionlint, and the independent
     Rust consumer check. The pre-push wrapper passed Maturin development install,
     mypy (12 files), strict clippy, Nextest (131/131), Rust doctest (1/1), pytest
     (41/41), and cargo-deny.
   - After a Git checkout became available, `make install-hooks` installed both
     executable `.git/hooks/pre-commit` and `.git/hooks/pre-push`. Corrected the
     stale upstream hook ID from `check-private-key` to `detect-private-key`.
   - Actual `pre-commit` and `pre-push` stages passed; the latter ran the complete
     current Rust/Python wrapper including mypy, Pyright, native-stub parity,
     strict Clippy, Nextest, doctests, pytest, and cargo-deny. A disposable file
     with trailing whitespace was rejected and fixed, and a second unchanged
     pre-commit run reused the installed environment and passed.
   Notes:
   - Local Git installation verification is complete.

45. [x] Consolidate and harden pull-request GitHub Actions quality gates
   Claimed by: Codex implementer (2026-07-01 22:52 PDT)
   Context:
   Todo 3 creates the initial `.github/workflows/ci.yml`; this follow-up must audit
   the completed workflow instead of replacing its intent. Pull requests should
   receive the same Rust, Python, packaging, and repository checks as local hooks,
   while workflows remain secure, cancellable, reproducible, and understandable to
   open-source contributors.
   Implementation:
   Refactor or extend CI into clearly named required jobs for repository/pre-commit
   checks, Rust format/clippy/nextest/doctests/docs, MSRV, stable Rust on Linux,
   macOS, and Windows, Python lint/type/tests across supported CPython versions,
   Cargo dependency policy, and a built-wheel install/import/smoke test in a clean
   environment. Make one job run `pre-commit run --all-files` and keep expensive
   platform matrices focused so the same lint work is not repeated unnecessarily.
   Add workflow-level concurrency that cancels stale runs for the same pull request,
   explicit `timeout-minutes`, least-privilege `permissions`, locked installs, and
   readable step/job names. Pin every third-party action to a reviewed full commit
   SHA with a version comment; do not execute untrusted pull-request code with
   write tokens or use `pull_request_target` for build/test jobs. Add `actionlint`
   and a GitHub Actions security linter such as `zizmor` to local manual hooks and
   CI. Cache only package registries/build inputs with keys scoped by OS,
   toolchain, and lockfiles; never restore executable artifacts across trust
   boundaries. Upload logs/test reports only when useful, set short retention, and
   ensure artifact names are unique across matrix jobs. Define stable required
   check names and document the branch-protection rules maintainers should enable.
   Tests and verification:
   Run the workflow linters locally and the full minimum gate. Open or use a test
   pull request to prove every required job runs, a superseding push cancels the
   stale run, matrix failures remain visible, wheel smoke installs the artifact
   rather than importing the source tree, and fork-origin pull requests receive no
   write-capable token. Inspect the run's effective permissions, cache keys, action
   SHAs, timeouts, and artifact retention. Record the successful workflow URL and
   the exact required-check names in evidence.
   Evidence:
   - Refactored `.github/workflows/ci.yml` into stable required jobs for repository
     hooks/contracts, Rust quality and public API, the three stable OS runners,
     MSRV, CPython 3.11-3.14, dependency policy, and a clean installed-wheel smoke.
     All jobs have explicit timeouts, read-only workflow permissions, readable
     steps, locked installs, non-persisted checkout credentials, and stale-run
     cancellation; no workflow uses `pull_request_target`.
   - Pinned every third-party action in all current workflows to a reviewed 40-hex
     upstream commit with a version comment. Tool installers additionally pin
     actionlint 1.7.12, nextest 0.9.138, cargo-deny 0.19.9, cargo-fuzz 0.13.2, and
     cargo-semver-checks 0.42.0. A repository scan found no mutable action refs.
   - Added locked zizmor 1.26.1, actionlint and medium-or-higher zizmor checks to
     the manual hook and repository CI job. `actionlint` passed for all four
     workflows and `zizmor --offline --persona=regular --min-severity=medium
     .github` reported no findings.
   - Cargo caches contain only registry index/cache inputs, keyed by runner OS,
     exact toolchain, and `Cargo.lock`; uv caches use `uv.lock`. No compiled
     `target/` tree is restored. The tested wheel artifact has a unique run/attempt
     name and three-day retention.
   - Built a CPython 3.14 wheel, installed only that wheel into a fresh venv under
     `/tmp`, changed outside the source checkout, and passed the v1/v2/hybrid
     `scripts/smoke_wheel.py` create/read/magnet/verify checks.
   - Full local gate passed: rustfmt, strict Clippy, Nextest 131/131, Rust doctest
     1/1, strict rustdoc, Ruff check/format (33 files), mypy (12 files), pytest
     41/41, specifications 15/74, links across 30 Markdown files, and cargo-deny.
     `CONTRIBUTING.md` records the exact 13 required check names and recommended
     branch-protection settings.
   - Pull request #1 exercised all 13 documented required contexts plus Dependency
     Review, coverage, and Rust/Python CodeQL. The final candidate passed Linux,
     macOS, Windows, MSRV, CPython 3.11-3.14, repository, API, dependency, and
     clean-wheel jobs: `https://github.com/burritothief/btpc/pull/1` and
     `https://github.com/burritothief/btpc/actions/runs/28567029414`.
   - Superseding pushes cancelled stale Coverage and CodeQL executions, while
     platform-specific Windows failures remained independently visible and were
     corrected before merge. Hosted job permissions showed read-only contents and
     metadata access; checkout credentials were removed and no build job used a
     write token or `pull_request_target`.
   - `main` branch protection now requires the exact 13 documented contexts with
     strict/current-branch enforcement, one approval, stale-review dismissal, and
     conversation resolution; force pushes and deletion are disabled.
   Notes:
   - PR #1 merged as `cd9bf52be48d880c8355d8501b161a8c1d7beef3` after every
     hosted check passed; subsequent `main` CI also passed.

46. [x] Add open-source security, dependency, coverage, and maintenance automation
   Claimed by: Codex implementer (2026-07-01 22:52 PDT)
   Context:
   Test CI alone does not cover vulnerable dependency changes, static security
   analysis, workflow supply-chain risk, stale dependency pins, or repository
   health. Add narrowly scoped automation appropriate for a public Rust/Python
   library without duplicating the release workflow planned in todo 35.
   Implementation:
   Add `.github/dependabot.yml` with weekly grouped updates for Cargo, Python/uv,
   and GitHub Actions, conservative open-PR limits, labels, and clear commit scopes;
   verify the selected Python ecosystem updates both `pyproject.toml` and `uv.lock`.
   Add a pull-request dependency-review workflow that fails on newly introduced
   dependencies with known high/critical vulnerabilities or denied licenses while
   allowing an explicitly documented, expiring exception process. Add scheduled
   and pull-request CodeQL analysis for Rust and Python using least privileges and
   appropriate compiled-language build configuration. Add an OpenSSF Scorecard
   workflow with pinned actions and SARIF upload, plus a repository-security
   checklist documenting how maintainers enable Dependabot alerts/security updates,
   private vulnerability reporting, secret scanning/push protection, required
   reviews, and branch protection in GitHub settings. Add Rust and Python coverage
   generation (`cargo llvm-cov` and pytest coverage or current equivalents), merge
   reports only where technically sound, publish a job summary and optional
   tokenless public-repository upload, and initially treat coverage as informational
   until meaningful protocol tests exist; later thresholds must prevent regression
   rather than incentivize low-value tests. Add scheduled documentation link checks
   and spelling/typo checks if they produce low-noise results, with matching manual
   pre-commit hooks. Keep release publication, attestations, and binary/wheel
   provenance in todo 35, but ensure these workflows are compatible with that
   future least-privilege/OIDC design. Do not add automatic stale-issue closure,
   automatic dependency merging, or bots that can push code without explicit
   maintainer approval.
   Tests and verification:
   Validate all YAML and run `actionlint` plus the selected workflow security
   linter. Trigger or dry-run each workflow and record URLs showing dependency
   review on a pull request, CodeQL results for both languages, Scorecard SARIF,
   coverage summaries/artifacts, and scheduled maintenance jobs. Create disposable
   dependency-update examples proving Cargo, uv, and action pins update the expected
   lock/config files and still run the complete CI gate. Confirm every workflow has
   explicit minimal permissions, pinned actions, concurrency/timeouts where useful,
   no secrets exposed to fork code, and documented ownership/remediation steps for
   alerts.
   Evidence:
   - Added weekly grouped Dependabot configuration for Cargo, uv, and GitHub
     Actions with seven-day cooldowns, five-PR limits, labels, and scoped commit
     prefixes. `cargo update --dry-run` resolved zero changes and `uv lock
     --upgrade --dry-run` detected no changes with the checked-in lockfiles.
   - Added pinned, least-privilege workflows for high/critical and denied-license
     Dependency Review, Rust/Python CodeQL, OpenSSF Scorecard SARIF, separate Rust
     LCOV/Python Coverage.py reporting, and scheduled documentation/spelling
     maintenance. Every job has a timeout; recurring workflows have concurrency;
     no workflow uses mutable action refs or exposes secrets to pull-request code.
   - Added `SECURITY.md`, `docs/security.md`, and contribution guidance covering
     private reporting, alert ownership/remediation, GitHub security settings,
     branch protection, secret scanning/push protection, and an issue-linked,
     owner-assigned, maximum-90-day dependency exception process.
   - Locked codespell 2.4.2, pytest-cov 7.1.0/Coverage.py 7.14.3, zizmor 1.26.1,
     and cargo-llvm-cov 0.8.7. Manual hooks and scheduled maintenance use a
     low-noise maintained-text spelling scope that excludes binary corpora and
     generated references; the manual wrapper passed all checks.
   - Local coverage generation passed all Rust tests and wrote a 514,593-byte LCOV
     report with 82.30% line coverage overall; Python ran 41/41 tests and wrote a
     9,550-byte XML report with 95% line coverage. Reports remain separate and
     informational as documented.
   - `actionlint` passed all ten workflows. Zizmor reported no medium-or-higher
     findings across workflows and Dependabot configuration. Codespell passed the
     maintained-text scope, all action references are 40-hex pins, and the full
     project gate passed: rustfmt, strict Clippy, Nextest 131/131, doctest 1/1,
     strict rustdoc, Ruff, mypy, pytest 41/41, specs 15/74, links across 31
     Markdown files, and cargo-deny.
   - Hosted Dependency Review passed on pull request #1:
     `https://github.com/burritothief/btpc/actions/runs/28567029435`. Rust and
     Python CodeQL passed on the final PR and on `main`:
     `https://github.com/burritothief/btpc/actions/runs/28567528057`.
   - OpenSSF Scorecard completed successfully with SARIF upload on `main`:
     `https://github.com/burritothief/btpc/actions/runs/28567528053`. Informational
     Rust/Python coverage passed and retained the combined evidence artifact
     `coverage-28567528045-1`:
     `https://github.com/burritothief/btpc/actions/runs/28567528045`.
   - The manually dispatched maintenance workflow passed documentation and spelling
     checks: `https://github.com/burritothief/btpc/actions/runs/28567389376`.
     The dependency/toolchain refresh audit passed policy, Clippy, Rust/Python
     tests, and retained `dependency-refresh-review-28567227613-1`:
     `https://github.com/burritothief/btpc/actions/runs/28567227613`.
   - Dependabot executed successful Cargo and uv update audits and opened GitHub
     Actions update PR #2; its dependency review, CodeQL, coverage, and complete
     13-context CI matrix all passed:
     `https://github.com/burritothief/btpc/pull/2`. Vulnerability alerts, security
     updates, private vulnerability reporting, secret scanning, and push protection
     are enabled; branch protection requires review and all documented CI contexts.
   Notes:
   - The repository is public under MIT. Cargo and uv required no manifest changes;
     PR #2 remains intentionally unmerged for explicit maintainer review because
     automatic dependency merging is prohibited.

47. [x] Build the benchmark harness foundation and canonical ISO preflight
   Claimed by: Codex implementer (2026-07-01 16:05 PDT)
   Requirements:
   `BENCH-DATA-001`, `BENCH-ENV-001`, `BENCH-REPRO-001`.
   Context:
   `specs/benchmarking.md` defines the focused single-file torrent creation
   benchmark using `debian-13.5.0-amd64-DVD-1.iso`. The harness needs a stable CLI,
   configuration model, canonical dataset verification, isolated execution
   environment, and serializable result schema before tool commands or timing are
   added. The canonical input is 2,184,647,899 bytes with SHA-256
   `36faa0f8895f064c3f4ca63386dd8ef27aae6239c05a9455644eb1a07f46ca47`.
   Implementation:
   Create `benches/torrent_creation.py` as an importable and executable Python CLI
   plus focused modules under `benches/harness/` when separation improves testing.
   Use `argparse`, typed dataclasses/enums, `pathlib`, and standard-library JSON/CSV
   primitives; add `psutil` as a locked development dependency for later resource
   sampling. Implement arguments for input, result root, selected tools, preset,
   warm-ups, measured rounds, seed, tracker, piece exponent, profile,
   `--require-tools`, `--render`, and `--compare`. Define a versioned raw result
   schema before writing measurements. Preflight must stream the entire input once,
   compute SHA-256 and all ordered 4 MiB SHA-1 v1 piece digests in a single pass,
   verify the canonical fingerprint when the canonical filename is used, and record
   size/name/mtime/path. Build an isolated child environment with temporary HOME,
   XDG config/cache locations, stable locale/timezone, `NO_COLOR`, and deterministic
   Python hashing. Capture best-effort OS/kernel/architecture/CPU/core/memory/
   filesystem and relevant runtime versions without failing on unavailable host
   metadata. Add a Make target such as `benchmark-iso` that invokes the quick preset
   without assuming competitor tools are installed. Do not run competitors yet.
   Tests and verification:
   Write pytest tests first for argument defaults/overrides, standard versus quick
   presets, deterministic seed handling, canonical fingerprint success/failure,
   piece-digest boundaries using tiny generated files, one-pass digest equivalence
   with independent hashlib calculations, isolated environment contents, portable
   missing-host-field behavior, and result-schema JSON round trips. Run the spec
   validator, focused pytest, Ruff, mypy, and a preflight-only invocation against
   the real ISO; record its exact size, SHA-256, piece count, and elapsed untimed
   preflight duration.
   Evidence:
   - Audited and extended the existing `benches/btpc_bench` foundation rather than
     duplicating it. Added executable/importable `benches/torrent_creation.py`, a
     `preflight` subcommand that never invokes competitors, and `make benchmark-iso`.
   - Extended the versioned dataset schema with basename and nanosecond mtime while
     preserving backward JSON decoding. The one-pass preflight streams SHA-256 and
     ordered piece SHA-1 digests; boundary tests independently verify partial final
     pieces, name, size, path, and mtime.
   - Added focused tests for the stable isolated HOME/XDG/locale/timezone/color/
     Python-hash environment and preflight JSON. Existing tests cover quick versus
     standard presets, overrides, deterministic seeds, canonical mismatch, portable
     host metadata, rendering/comparison, and full schema round trips.
   - Updated `BENCH-ENV-001` and `BENCH-REPRO-001` traceability to Implemented.
     Focused verification passed: 17 benchmark tests, Ruff check/format on 13 files,
     strict mypy on 13 files, and spec validation for 15 specs/74 requirements.
   - Ran the real accepted canonical ISO through the direct wrapper. Preflight read
     `debian-13.5.0-amd64-DVD-1.iso` once in 3.225022 seconds (3.32 seconds wall),
     measured exactly 3,989,078,016 bytes, SHA-256
     `343b6e02a8bdf6429eb3722ee0056b5c7d9ad17d88328e499909da7205e55f50`,
     and 952 ordered 4 MiB SHA-1 pieces. `make benchmark-iso` repeated successfully
     in 3.070660 seconds and wrote `benchmark-results/preflight.json`.
   Notes:
   - The todo text contained a stale 2,184,647,899-byte fingerprint. Per source-of-
     truth order, the accepted `specs/benchmarking.md` contract wins and was proven
     against the real local ISO using its 3,989,078,016-byte fingerprint.

48. [x] Implement an independent v1 torrent benchmark validator
   Claimed by: Codex implementer (2026-07-01 16:08 PDT)
   Requirements:
   `BENCH-PROFILE-001`, `BENCH-VALID-001`, `META-HASH-001`, `META-V1-001`.
   Context:
   A fast invalid torrent is not a benchmark result. Validation must run after the
   timer stops and independently compare generated torrents with the preflight
   payload oracle. BTPC's public creator/validator may be incomplete while the
   harness is being built, so the benchmark cannot depend solely on the code under
   test or on a competitor library.
   Implementation:
   Add a benchmark-only strict-enough bencode/metainfo reader or reuse only the
   already implemented, test-backed `btpc-core` parsing API through a small helper
   binary when it can expose all required raw fields without changing benchmark
   timing. The validator must preserve the raw `info` slice and compute its SHA-1,
   then require v1 single-file mode, exact basename bytes, exact input length,
   piece length `2^22`, private integer `1`, the configured announce URL, exact
   expected piece count, and byte-for-byte equality of every concatenated 20-byte
   SHA-1 piece digest from preflight. It must distinguish parse failure, semantic
   profile mismatch, payload digest mismatch, and info-hash mismatch. The first
   successful comparable torrent establishes the expected raw info hash; all later
   comparable tools/runs must match it. Top-level creation date and creator fields
   are ignored for comparability but retained in diagnostic metadata. Validation
   must never occur inside the measured interval.
   Tests and verification:
   Build tiny canonical torrent fixtures independently in tests and start with
   failing cases for malformed bencode, multi-file mode, wrong name/length/piece
   length/private/tracker/piece count, one modified piece digest, and divergent info
   hashes. Test accepted differences limited to top-level creator/creation date.
   Cross-check validator results against the existing `btpc-core` raw/v1 metainfo
   API and at least one external parser when available. Run focused tests, all spec
   validation, Rust core tests if a Rust helper is used, Ruff, and mypy.
   Evidence:
   - Hardened the existing independent benchmark bencode reader with stable typed
     failure categories for parse, profile, payload digest, raw info-hash, and BTPC
     cross-check failures. It preserves and SHA-1 hashes the exact raw `info` slice.
   - Enforced the first successful smoke torrent's raw info hash across every later
     tool smoke, warm-up, and measured run. Validation remains after process timing;
     invalid runs retain their reason and are excluded from valid statistics.
   - Retained top-level `created by` and `creation date` diagnostics while allowing
     those fields to differ. Exact v1 single-file name bytes, length, 4 MiB profile,
     private flag, tracker, info field set, piece count, and every concatenated
     20-byte piece digest remain mandatory.
   - Expanded independent fixtures to malformed bencode, multi-file mode, wrong
     name/length/piece length/private/tracker, wrong piece count/digest, unexpected
     info fields, top-level metadata differences, and divergent raw info hashes.
   - Cross-checked a valid fixture against BTPC's raw/v1 API and the installed torf
     5.2.1 external parser; both reported the same info hash and profile semantics.
     Focused verification passed 25 benchmark tests, 9 btpc-core raw/v1 tests, Ruff
     check/format on 13 files, strict mypy on 13 files, and specs 15/74.
   - Marked `BENCH-VALID-001` Implemented with source/test traceability.
   Notes:
   - `torrenttools` was unavailable; the installed `torf` CLI provided the required
     external-parser cross-check. The test skips only when torf is absent.

49. [x] Add smoke-tested adapters for BTPC and competitor torrent CLIs
   Claimed by: Codex implementer (2026-07-01 16:11 PDT)
   Requirements:
   `BENCH-PROFILE-001`, `BENCH-TOOLS-001`, `BENCH-VALID-001`.
   Context:
   `mktorrent`, `mkbrr`, `torf-cli`, `torrenttools`, and BTPC use different option
   names and piece-length units. None of the competitor executables is currently
   installed in this workspace, so adapters must discover capabilities and report
   unavailable or unsupported states instead of guessing or aborting the whole run.
   Implementation:
   Create a typed adapter registry for native BTPC, optional installed BTPC Python
   API, `mkbrr`, `mktorrent`, `torf-cli` (executable typically `torf`), and
   `torrenttools`. Each adapter declares executable candidates, version probes,
   supported profiles, command construction, exact 4 MiB piece-size conversion,
   private/tracker/output/name behavior, quiet flags, optional no-date/no-creator
   flags, and documented default/one-worker controls. Resolve version-specific
   command semantics from each tool's `--help`/official documentation and capture
   the resolved command template in metadata. Before warm-ups, run one untimed
   smoke creation in the isolated environment and validate it with todo 48. Report
   `UNAVAILABLE`, `UNSUPPORTED`, `SMOKE_FAILED`, or `READY` with an actionable
   reason. `--require-tools` must fail before measurements when a requested tool is
   not READY; otherwise continue with READY tools. Never shell-interpolate paths;
   use argument arrays and per-run output/log paths. Provide a documented setup
   guide with pinned example install commands/versions for macOS and Linux, but do
   not make the benchmark script install software automatically.
   Tests and verification:
   Add table-driven tests for each adapter's discovery, version parsing, command
   arguments, piece-size semantics, tracker/private flags, output path, quiet mode,
   default concurrency, optional one-worker profile, spaces/non-ASCII in paths,
   and unavailable/unsupported versions. Use fake executables to exercise version
   and smoke outcomes without network installs. When tools are available, run the
   tiny smoke payload and retain generated commands/logs; otherwise prove they are
   reported as unavailable and not silently omitted.
   Evidence:
   - Audited and hardened the typed registry covering BTPC native/Python, mkbrr,
     mktorrent, torf-cli, and torrenttools. Adapters declare discovery/version
     probes, supported profile, piece-size semantics, default concurrency, optional
     one-worker command transforms, tracker/private/output/no-date behavior, and
     argument-array command construction.
   - Added explicit `SMOKE_FAILED` status. Every discovered tool performs an
     untimed validated smoke before warm-ups; `--require-tools` now rejects both
     discovery/profile failures and smoke failures before measurements.
   - Added torf configuration isolation (`--noconfig`) and no-magnet output while
     retaining the torrent artifact. Fake executables cover unavailable, unsupported
     version, smoke exit failure, continuation, and required-tool failure paths.
   - Added pinned macOS/Linux example setup commands and documented that the harness
     never installs tools. Known torrenttools 0.6.2 cross-seed incompatibility is
     reported rather than ranked.
   - Ran a real one-round canonical ISO smoke with exact 4 MiB semantics. BTPC
     native 0.1.0, BTPC Python 3.14.3, mktorrent 1.1, and torf-cli 5.2.1 all passed
     and produced identical info hash `6fd397c6de29...`; mkbrr and torrenttools were
     retained as actionable `UNAVAILABLE` rows. Commands, logs, torrents, JSON,
     CSV, and summaries were retained under `/tmp/btpc-todo49.b7RpB6`.
   - Marked `BENCH-PROFILE-001` and `BENCH-TOOLS-001` Implemented. Verification
     passed 26 benchmark tests, Ruff check/format on 13 files, strict mypy on 13
     files, and specs 15/74.
   Notes:
   - Torf treats its piece-size option as a maximum and selects smaller pieces for
     tiny payloads; its canonical ISO smoke proves exact 4 MiB comparability for the
     specified dataset. Tiny fake/unit tests validate its command shape only.

50. [x] Implement randomized process measurement and resource sampling
   Claimed by: Codex implementer (2026-07-01 16:14 PDT)
   Requirements:
   `BENCH-RUN-001`, `BENCH-CACHE-001`, `BENCH-METRIC-001`, `BENCH-ENV-001`.
   Context:
   The benchmark needs robust repeated measurements while keeping setup, output
   deletion, logs, and correctness validation outside the timed region. Process
   trees and different concurrency defaults make shell `time` and one-off timings
   insufficient as the primary harness.
   Implementation:
   Implement two untimed warm-ups and ten measured blocked rounds for the standard
   preset, and one/three for quick. Seed and record a random permutation of READY
   tools independently for each measured round so every round contains one run per
   tool. Before timing, remove the intended output and prepare log paths; start the
   child with monotonic high-resolution timing, redirect stdout/stderr, and sample
   the root plus descendants with `psutil` at a documented interval (default 10 ms)
   for peak RSS and best-effort aggregate CPU time. Stop timing at process exit,
   then validate the torrent and record output size. Preserve every sample and its
   order, exit code, signal, wall/user/system time, throughput, peak RSS, log paths,
   validation result, and failure reason. Do not retry or delete outliers
   automatically. Label the default profile warm-cache. Support an optional explicit
   cold-cache preparation command only as a separate profile; never invoke privileged
   cache-dropping implicitly. Handle timeout, interrupt, process-tree termination,
   missing children, sampling races, and cleanup deterministically. Failed/invalid
   tools must stop receiving later measured runs but retain completed samples.
   Tests and verification:
   Use fake tools that sleep, spawn children, allocate known memory, write valid or
   invalid output, fail, hang, and respond to termination. Verify warm-up exclusion,
   round membership, deterministic random order by seed, setup/validation outside
   timing, process-tree RSS capture within a tolerant bound, timeout/interrupt
   cleanup, no automatic outlier removal, warm-cache labels, and complete raw sample
   retention. Run the quick preset against any READY real tools only after all fake
   process tests pass.
   Evidence:
   - Extended benchmark configuration with a 600-second default timeout, 10 ms
     process-tree sampling interval, and explicit cold-cache preparation command.
     Cold labels without a caller command and preparation commands on warm runs are
     rejected; no privileged cache dropping is performed implicitly.
   - Process measurement uses monotonic `perf_counter`, redirects per-run logs,
     samples root plus recursive children for peak RSS and aggregate CPU, records
     exit code/signal/timeout/output size, and deterministically terminates then
     kills remaining descendants on timeout or `KeyboardInterrupt`.
   - Measured failures and invalid outputs retain the completed raw sample and stop
     that tool from receiving later scheduled rounds. Warm-ups remain untimed and
     excluded from raw samples; output deletion, cache preparation, and validation
     occur outside the measured interval.
   - Added fake-process tests for sleep/child allocation (combined RSS over 30 MiB),
     timeout and signal capture, interrupt cleanup, explicit cold-cache contract,
     failed-run retention/stopped scheduling, warm-up exclusion, seeded complete
     round blocks, output-size fields, and no sample deletion. Focused suite passed
     31/31 with Ruff and strict mypy on 13 benchmark files.
   - Ran the canonical ISO quick preset after fake tests: one warm-up and three
     measured rounds each for BTPC native, mktorrent, and torf-cli. All 9 measured
     runs validated with identical info hash; medians were 0.813 s, 0.504 s, and
     0.633 s respectively. Raw commands/logs/torrents/reports are under
     `/tmp/btpc-todo50.YLAWV6`.
   - Marked `BENCH-RUN-001`, `BENCH-CACHE-001`, and `BENCH-METRIC-001`
     Implemented. Full verification passed strict Rust lint, Nextest 131/131,
     doctest 1/1, strict rustdoc, Ruff (34 files), strict mypy (25 files), Python
     plus benchmark pytest 72/72, specs 15/74, links in 31 Markdown files, and
     cargo-deny.
   Notes:
   - Peak RSS is best-effort and may be zero for processes shorter than the sampling
     interval; the canonical runs captured stable nonzero process-tree peaks.

51. [x] Add statistics, ASCII reporting, saved rendering, and result comparison
   Claimed by: Codex implementer (2026-07-01 16:18 PDT)
   Requirements:
   `BENCH-METRIC-001`, `BENCH-OUTPUT-001`, `BENCH-REPRO-001`, `PERF-BENCH-001`.
   Context:
   The requested primary output is a clear ASCII table, while reproducible analysis
   requires machine-readable samples and rerendering without rerunning the 2.0 GiB
   workload. Rankings must include only valid equivalent torrents and still show
   unavailable or failed competitors.
   Implementation:
   Compute sample count, median, mean, sample standard deviation, minimum, maximum,
   median absolute deviation, coefficient of variation, median MiB/s, maximum peak
   RSS, and relative speed versus the fastest valid median using Python's standard
   statistics module. Use median wall time as the only ranking key; retain all raw
   samples and flag high variability without deleting values. Write timestamped
   result directories containing schema-versioned `results.json`, `samples.csv`,
   `environment.json`, `commands.json`, per-run logs, one retained validated final
   torrent per tool, and `summary.txt`. Implement a deterministic dependency-light
   fixed-width ASCII renderer with adaptive column widths, human-readable units,
   no ANSI when non-interactive/`NO_COLOR`, ranked valid rows first, and explicit
   unavailable/unsupported/failed/invalid rows with reasons. Implement `--render`
   for saved JSON and `--compare BASE NEW` with absolute/percentage median changes,
   sample counts, variability, and a warning that it does not establish statistical
   significance. Reject unknown future schema versions cleanly.
   Tests and verification:
   Add golden tests for table layout at narrow/normal widths, long tool versions,
   missing RSS/CPU values, ties, one sample, failed tools, Unicode replacement,
   deterministic ordering, unit boundaries, raw JSON/CSV contents, rerender parity,
   schema rejection, and comparison math. Verify the example table in
   `specs/benchmarking.md` remains representative. Run focused tests, spec checks,
   Ruff, mypy, and render a synthetic all-status report in CI.
   Evidence:
   - Completed deterministic median-only ranking with name tie-breaks, full mean/
     sample-SD/min/max/MAD/CV/throughput/peak-RSS statistics, fastest-relative
     speed, and stable unranked status ordering. Narrow rendering can truncate long
     tool/version/reason columns to a caller width while preserving ASCII layout;
     Unicode is emitted safely without ANSI.
   - Raw output now writes canonical `results.json` plus a backward-compatible
     `result.json`, expanded `samples.csv` with signal/output-size/timeout fields,
     `environment.json`, `commands.json`, summaries, per-run logs, and retained
     validated torrents. Unknown future schema versions fail explicitly while old
     schema-1 samples default newly added fields safely.
   - Expanded comparisons with absolute seconds, percentage change, sample counts,
     CV, faster/slower ratios, dataset/profile/cache warnings, and the existing
     descriptive-only statistical-significance disclaimer.
   - Added golden/property coverage for all statuses, long/Unicode versions, narrow
     width, deterministic ties, missing RSS, one sample, raw CSV columns, canonical
     and compatibility JSON, future-schema rejection, rerender parity, and exact
     comparison math. Benchmark tests passed 33/33.
   - Added `scripts/render_benchmark_fixture.py` and CI/manual-hook execution for a
     synthetic success/unavailable/unsupported/smoke-failed/failed/invalid report.
     A fresh current-schema run at `/tmp/btpc-todo51.eyKWq9` passed JSON/CSV checks
     and byte-for-byte saved rerender parity; an older todo-50 `result.json` also
     rerendered byte-for-byte, proving backward compatibility.
   - Marked `BENCH-OUTPUT-001` Implemented. Full gate passed strict Rust lint,
     Nextest 131/131, doctest 1/1, strict rustdoc, Ruff (35 files), strict mypy (26
     files), Python/benchmark pytest 74/74, specs 15/74, links in 31 Markdown files,
     cargo-deny, actionlint, and zero medium-or-higher zizmor findings.
   Notes:
   - The example table in `specs/benchmarking.md` remains representative; the live
     renderer includes additional status and info-hash columns needed for failures
     and equivalence evidence.

52. [x] Run and publish the standard Debian ISO creation benchmark
   Claimed by: Codex implementer (2026-07-01 16:22 PDT)
   Requirements:
   `BENCH-DATA-001`, `BENCH-PROFILE-001`, `BENCH-ENV-001`, `BENCH-RUN-001`,
   `BENCH-CACHE-001`, `BENCH-METRIC-001`, `BENCH-VALID-001`, `BENCH-TOOLS-001`,
   `BENCH-OUTPUT-001`, `BENCH-REPRO-001`, `PERF-BENCH-001`.
   Context:
   The harness is complete only after a real, validated standard run against the
   canonical 2,184,647,899-byte Debian ISO demonstrates installation guidance,
   adapter equivalence, repeated timing, resource capture, and reporting. BTPC may
   not yet have a torrent creation CLI when this todo reaches the front of the
   queue; in that case it must be reported accurately rather than simulated.
   Implementation:
   Install or make available pinned current versions of `mkbrr`, `mktorrent`,
   `torf-cli`, and `torrenttools` using the documented setup outside the harness;
   include native BTPC and BTPC Python only if their creation surfaces are complete.
   Record exact versions and paths. Run a preflight, then the standard warm-cache
   preset with the fixed v1/private/4 MiB/tracker profile, two warm-ups, ten blocked
   randomized rounds, and a recorded random seed. Review logs and variability; if
   coefficient of variation or system conditions are suspicious, run a second
   complete session rather than deleting samples. Retain all raw output under the
   ignored benchmark-results directory and commit a compact Markdown report plus
   `summary.txt`, environment/command metadata, and checksums of raw JSON artifacts
   without committing the ISO, large logs, or duplicate torrents. State limitations
   explicitly: single large file, warm cache, default concurrency, one machine,
   v1 only. Do not claim a universally fastest tool.
   Tests and verification:
   Require canonical input fingerprint success and every ranked run to pass the
   independent torrent validator with an identical info hash. Confirm ten valid
   samples per ranked tool, no hidden missing tool, correct warm-cache label, raw
   JSON/CSV rerender parity, and all commands/tool versions/environment fields in
   the report. Run the full project/spec gate after generating the report. Record
   the exact benchmark command, elapsed session time, ASCII table, result directory,
   and any unavailable/unsupported/failed tools in evidence.
   Evidence:
   - Installed and recorded mkbrr 1.23.0, mktorrent 1.1, torf-cli 5.2.1,
     torrenttools 0.6.2, the release `btpc` CLI, and the release editable Python
     extension. Updated `benches/README.md` with the verified mkbrr and official
     torrenttools installation procedures. The preflight result directory was
     `/tmp/btpc-todo52-smoke.DpRa29/20260701T232237.921524Z`.
   - The specification's canonical Debian 13.5.0 DVD ISO is 3,989,078,016 bytes
     (the older 2,184,647,899-byte context value above is stale). Its required
     SHA-256 fingerprint passed before both sessions. The definitive command was
     `uv run python benches/torrent_creation.py run debian-13.5.0-amd64-DVD-1.iso
     --output-root benchmark-results/todo52-standard-release --preset standard
     --seed 20260703 --tools all`; it completed in 46.29 seconds at
     `benchmark-results/todo52-standard-release/20260701T233001.755832Z`.
   - All five ranked tools completed two excluded warm-ups and ten warm-cache
     measured rounds; every retained torrent passed the independent validator and
     had info hash `6fd397c6de29f77d0f0c1928e6c457240112204c`. Median seconds / CV / peak RSS
     were mkbrr 0.2472 / 5.20% / 74.5 MiB, mktorrent 0.5189 / 3.60% / 26.1 MiB,
     torf 0.6421 / 4.22% / 95.0 MiB, btpc-native 0.8161 / 2.60% / 23.1 MiB, and
     btpc-python 0.8608 / 0.71% / 39.9 MiB. Torrenttools was shown unranked as
     `smoke_failed`: its output contained unexpected `info` fields and therefore
     was not equivalent to the fixed profile.
   - Because mkbrr variability was higher than the other tools, ran a complete
     independent replication with seed 20260704 in 46.05 seconds at
     `benchmark-results/todo52-standard-release-rerun/20260701T233054.341474Z`.
     All ranked tools again produced ten valid identical-hash samples; native,
     Python, mktorrent, and torf medians changed by +0.96%, -0.19%, -0.44%, and
     +0.64%, while mkbrr changed +6.88% and retained 7.24% CV. No samples were
     deleted. Two earlier debug-extension sessions remain retained under
     `benchmark-results` but are explicitly excluded from publication.
   - Published the compact report and exact command/environment/summary metadata
     under `benches/releases/2026-07-01-debian-iso-standard/`. The report records
     raw artifact locations and SHA-256 checksums, setup correction, full ASCII
     tables, environment and tool paths, variability, and the required limitations
     (one machine, one large file, warm cache, default concurrency, v1 only).
     Checked committed metadata byte-for-byte against raw output and rerendered the
     primary `results.json` to an identical `summary.txt`.
   - Full acceptance passed: formatting, strict Clippy, Nextest 131/131, doctest
     1/1, strict rustdoc, Ruff, strict mypy, release maturin build, pytest 74/74,
     specs 15/74, links in 32 Markdown files, and cargo-deny. Actionlint reported
     that no workflow project exists in this source export; zizmor is unavailable.
   Notes:
   - Results compare only this controlled workload and do not establish a
     universally fastest implementation.

53. [x] [Review] Enforce parser and ownership limits through every loading surface
   Claimed by: Codex implementer (2026-07-01 16:33 PDT)
   Context:
   `ParseLimits::max_owned_allocation` is currently never applied by
   `Metainfo::from_bytes`, `value_to_owned`, or canonical snapshot construction.
   `Metainfo::from_path` also calls `std::fs::read` before checking the configured
   input limit, and no advanced metainfo constructor accepts caller limits. An
   attacker-controlled torrent can therefore trigger allocations beyond the
   documented safety contract even though the low-level parser exposes limits.
   Implementation:
   Add a coherent `ParseOptions`/load-options API used by raw and owned metainfo
   loading. Preflight regular-file length before allocation, still enforce the
   actual bytes read to handle races/non-regular inputs, and reject over-limit
   input before cloning. Thread an allocation budget through borrowed parse tree
   storage, owned conversion, canonical encoding buffers, file/path snapshots, and
   unknown fields with checked arithmetic. Make default constructors use safe
   defaults and let advanced callers adjust limits explicitly without disabling
   protocol validation accidentally.
   Tests and verification:
   Add exact-boundary and one-byte-over tests for bytes and paths, deeply nested
   small values, many zero-length values, canonical expansion bookkeeping, unknown
   fields, short/long reads, and a file that changes size between metadata and
   reading. Assert the error is `ResourceLimit`, allocations remain within a
   measured tolerant bound, and Rust/Python/CLI defaults and overrides agree. Run
   parser/metainfo tests, pytest, process tests, and strict clippy.
   Evidence:
   - Added public `ParseOptions` backed by `ParseLimits`; default constructors keep
     conservative defaults, while `RawMetainfo` and `Metainfo` expose explicit
     bytes/path constructors. Loaded `Metainfo` retains its options and verifier
     reparses use the same limits rather than silently reverting to defaults.
   - `from_path_with_options` opens first, preflights regular-file metadata before
     allocation, caps reads at one byte beyond `max_total_input`, checks the actual
     byte count, and then parses. This rejects oversized files and growth/stream
     input without `std::fs::read` allocating an attacker-declared size wholesale.
   - Added a shared checked allocation budget to borrowed list/dictionary storage
     with fallible reservations. Owned loads precompute and charge original-byte,
     canonical-buffer, owned-tree/snapshot, key/path, and unknown-field costs before
     cloning. Canonicality is evaluated against the already parsed tree, removing a
     second default-limit parse.
   - Added Rust boundary tests for exact/one-under input and owned budgets across
     byte and path loading, parser-container storage, canonical output, and a 4 KiB
     unknown field. Existing depth/item/string tests now isolate their intended
     limit. Core regression tests passed in full.
   - Added CLI `--max-input-bytes` and `--max-owned-bytes` to inspect, validate,
     magnet, and verify; all share the core option constructor and map violations to
     exit code 4. Added Python `ParseOptions` and `ResourceLimitError`, preflighted
     contiguous buffers before `tobytes()`, preserved limit/actual/maximum context,
     and verified byte/path parity. Regenerated help, completions, and manpage.
   - Documented the contract in Rust, Python, CLI, bencode, and security guides.
     Full acceptance passed formatting, strict Clippy, Nextest 135/135, doctest
     1/1, strict rustdoc, Ruff, strict mypy, release maturin build, pytest 75/75,
     specs 15/74, links in 32 Markdown files, cargo-deny, generated-reference tests
     2/2, and codespell.
   Notes:
   - The budget is a conservative upper-bound accounting model, intentionally
     charging duplicated owned representations before allocation rather than
     attempting allocator-specific byte-exact measurement.

54. [x] [Review] Preserve valid unbounded bencode integers at the syntax layer
   Claimed by: Codex implementer (2026-07-01 16:41 PDT)
   Context:
   The bencode specification does not impose an integer size limit, but the syntax
   parser currently converts every integer directly to `i64` and rejects otherwise
   valid encodings outside that range. Protocol fields need bounded numeric
   conversions, but a generic byte-preserving parser and unknown-field round trip
   must not discard valid large integers.
   Implementation:
   Represent parsed integers losslessly at the syntax layer, either as a borrowed
   canonical sign/digit slice with checked conversion accessors or a purpose-built
   arbitrary-precision value that does not force heavyweight arithmetic into hot
   paths. Keep convenient `i64`/`u64` conversions fallible and field-contextual for
   typed metainfo. Ensure the owned bencode model and canonical encoder can preserve
   and emit arbitrary valid integers while still rejecting negative zero, leading
   zeros in strict mode, signs without digits, and configured digit/allocation
   limits.
   Tests and verification:
   Add vectors at `i64`/`u64` boundaries and far beyond them, negative large values,
   unknown-field round trips, canonical/non-canonical forms, conversion failures
   with field context, property tests over long digit strings, and fuzz seeds.
   Confirm typed lengths and timestamps still reject out-of-domain values without
   panic or truncation.
   Evidence:
   - Replaced syntax-layer `i64` storage with borrowed `bencode::Integer`, which
     preserves the exact signed decimal bytes and offers fallible `to_i64()` and
     `to_u64()` accessors. The parser accepts values at and beyond both machine
     boundaries without allocation or truncation.
   - Added a default 4,096-digit `ParseLimits` cap plus
     `with_max_integer_digits`; parser checks it before conversion. CLI
     `--max-integer-digits` and Python `ParseOptions.max_integer_digits` route to
     the same core limit and produce structured `ResourceLimit` failures.
   - Added `OwnedValue::integer_bytes` and an arbitrary-precision integer variant.
     Constructors and encoders reject empty/sign-only values, plus signs, leading
     zeroes, and negative zero; valid large positive/negative values serialize
     canonically. Direct invalid enum construction is also rejected before output.
   - Unknown top-level integers survive owned metainfo round trips. Parseable
     noncanonical large integers retain original bytes/hash identity while
     canonical output removes leading zeroes without numeric conversion. Owned
     allocation accounting includes copied integer digit buffers.
   - Typed metainfo conversions remain bounded and field-contextual: positive and
     non-negative fields convert to `u64`, meta version/private convert to `i64`,
     and over-range values return deterministic `Metainfo` errors rather than
     syntax errors, panic, or truncation.
   - Added exact `i64`/`u64` boundary vectors, far-larger positive and negative
     vectors, canonical/noncanonical owned tests, typed overflow tests, a 64-case
     property test over 65–513 digit integers, and three canonical-roundtrip fuzz
     seeds. Updated the bencode contract and Rust/Python/CLI guides.
   - Full acceptance passed formatting, strict Clippy, Nextest 142/142, doctest
     1/1, strict rustdoc, Ruff, strict mypy, release maturin build, pytest 75/75,
     specs 15/74, links in 32 Markdown files, cargo-deny, generated-reference tests
     2/2, and codespell. The first Nextest attempt exposed stale generated help;
     rebuilding the CLI before regeneration fixed it, and the complete rerun passed.
   Notes:
   - Arbitrary-precision integers are represented as validated decimal bytes; BTPC
     deliberately does not add a big-integer arithmetic dependency.

55. [x] [Review] Reject ambiguous v1 file graphs and fully validate hybrid padding
   Claimed by: Codex implementer (2026-07-01 16:47 PDT)
   Context:
   v1 parsing validates each path component independently but does not reject
   duplicate full paths or file/directory prefix conflicts such as `a` and `a/b`.
   Such metainfo cannot map safely to a filesystem and can make verification or
   extraction target-dependent. Hybrid validation identifies padding solely by an
   attribute byte and alignment length, without proving uniqueness/non-collision
   of all v1 paths or applying a documented padding-path policy.
   Implementation:
   Build a byte-oriented path trie or an equivalently straightforward sorted-path
   validator shared by v1 parsing, creation, hybrid validation, and verification.
   Reject duplicate paths, file-as-directory prefixes, reserved unsafe components,
   and collisions introduced by platform mapping. For hybrid torrents, distinguish
   padding entries from real files using the complete BEP 47/52 contract, require
   exact alignment and valid placement, prevent padding paths from colliding with
   real/v2 paths, and expose padding explicitly instead of silently dropping it
   from typed inspection.
   Tests and verification:
   Add malicious fixtures for duplicates, `a` plus `a/b`, repeated padding names,
   real files marked `p`, misplaced/zero/unneeded padding, multiple consecutive
   padding files, path collision with `.pad`, and platform-specific case/encoding
   collisions where relevant. Verify Rust, CLI, and Python report deterministic
   field/path errors and never construct an unsafe payload map.
   Evidence:
   - Added a shared byte-oriented sorted-path graph validator used by both manifest
     creation and v1 metainfo parsing. It rejects exact duplicates and either-order
     file/directory prefix collisions; Windows builds additionally reject ASCII
     case-fold collisions after platform mapping.
   - Added deterministic CLI coverage proving ambiguous `a` plus `a/b` metainfo
     exits with invalid-data code 4. Rust tests cover duplicates and both prefix
     orderings before any unsafe payload map can be constructed.
   - Added explicit `V1File::is_padding` and owned `TorrentFile::is_padding`.
     Hybrid owned inspection now includes validated v1 padding entries instead of
     dropping them; Python `TorrentFile.is_padding` exposes the same distinction.
     Generic filesystem verification filters padding, while v1 hashing retains its
     logical zero-byte stream semantics.
   - Hybrid validation now requires `p` entries to use the reserved interoperable
     `.pad/<length>` or BTPC `.pad/<offset>-<length>` forms, match encoded length
     and optional exact logical offset, appear only between real files, never be
     consecutive/zero/unneeded/trailing/leading, and exactly fill the alignment
     gap. Real hybrid files may not occupy `.pad`.
   - Added malicious cases for fake real files marked `p`, malformed paths, wrong
     offsets/lengths, leading/trailing/consecutive/zero/unneeded padding, duplicate
     and prefix paths, plus generated-pad visibility. The existing torrenttools
     hybrid fixture exposed the valid `.pad/<length>` convention; validation was
     broadened to accept it without weakening placement or alignment checks.
   - Updated metainfo, Rust, and Python documentation. Final acceptance passed
     formatting, strict Clippy, Nextest 145/145, doctest 1/1, strict rustdoc, Ruff,
     strict mypy, pytest 76/76, interoperability corpus, specs 15/74, links in 32
     Markdown files, cargo-deny, and generated-reference tests 2/2.
   Notes:
   - Platform collision validation is intentionally target-specific; Unix retains
     raw-byte case distinctions while Windows rejects ASCII case aliases.

56. [x] [Review] Make atomic output replacement race-free and cross-platform
   Claimed by: Codex implementer (2026-07-01 16:54 PDT)
   Context:
   `atomic_write` checks `destination.exists()` and later calls `rename`. In deny
   mode this is a time-of-check/time-of-use race and Unix rename may overwrite a
   destination created after the second check. In overwrite mode, rename behavior
   differs on Windows, and syncing only the temporary file does not make the
   directory entry durable. The public promise of overwrite refusal and atomic
   writes is therefore stronger than the implementation.
   Implementation:
   Introduce a small platform-aware atomic persistence abstraction with explicit
   no-clobber and replace semantics. Use kernel atomic primitives or a maintained
   crate rather than `exists()` checks; keep the temporary in the destination
   directory, preserve useful permissions policy, sync file contents before
   publication, and provide an opt-in durability policy that syncs the parent
   directory where supported. Define behavior for symlink destinations, Windows
   sharing violations, permission failures, and cleanup after every failure.
   Tests and verification:
   Add concurrent creator tests racing for the same absent destination, a process
   that creates the destination immediately before publish, overwrite tests on
   Unix/Windows CI, symlink destination tests, injected write/sync/publish errors,
   and assertions that exactly one no-clobber writer succeeds with no temp leaks or
   partial destination. Run core, CLI, and Python atomic-output suites.
   Evidence:
   - Replaced check-then-rename publication with `tempfile::NamedTempFile` in the
     destination directory. Deny mode uses no-clobber persistence, replace mode
     uses the crate's platform-specific replacement primitive, file contents are
     synced before publication, and every pre-publication error drops the named
     temporary file.
   - Added `DurabilityPolicy` and `Creator::create_to_path_with_durability`.
     `FileAndDirectory` syncs the parent directory on Unix after publication;
     other platforms retain the documented file-sync guarantee where directory
     handles do not support the same operation. CLI `--durable` and Python
     `durable=True` expose the opt-in policy.
   - Replace mode preserves permissions of an existing regular file. Deny mode
     treats every existing filesystem entry, including symlinks, as occupied;
     replace mode replaces a symlink entry without following or modifying its
     target. Publication and permission errors retain path context, and the
     stable existing-destination diagnostic is normalized across operating
     systems.
   - Added deterministic injected write, file-sync, publish, and directory-sync
     failure tests; a hook creates a competing destination immediately before
     publication and proves deny mode retains its bytes. Added eight-way racing
     creators proving exactly one winner, no partial metainfo, and no leaked temp
     files, plus durability, permission, and Unix symlink policy coverage.
   - Updated the creation specification, Rust/CLI/Python API documentation,
     native typing stub, and generated CLI help/manpage/completion references.
     Focused core, CLI, and Python atomic-output suites passed after rebuilding
     the extension.
   - Final acceptance passed formatting, strict Clippy, Nextest 150/150, doctest
     1/1, strict rustdoc, Ruff, strict mypy, and pytest 43/43.
   Notes:
   - A directory-sync failure is reported after the complete destination has
     already been atomically published; callers requesting that stronger
     durability can distinguish this from a pre-publication failure by checking
     the destination.

57. [x] [Review] Harden manifest identity, symlink roots, and byte-oriented filters
   Claimed by: Codex implementer (2026-07-01 17:00 PDT)
   Context:
   Manifest mutation checks currently compare only length and modification time,
   which can miss same-size replacement or content mutation with restored/coarse
   timestamps. A followed top-level symlink is scanned with an empty torrent path,
   while a skipped top-level symlink can yield an empty manifest. Include/exclude
   matching converts components with `from_utf8_lossy`, causing distinct non-UTF-8
   names to share replacement text and making filtering non-lossless.
   Implementation:
   Record and verify the strongest portable file identity available (type, size,
   timestamps, and device/inode or Windows file ID where supported), verify the
   opened handle itself before and after hashing, and document residual mutation
   guarantees. Define top-level symlink behavior explicitly: reject/skip must not
   produce a successful empty torrent, and follow must retain the selected root
   name and apply escape/cycle rules consistently. Replace lossy glob matching with
   an explicit raw-byte pattern model or restrict text globs to UTF-8 paths while
   providing a byte-safe predicate/API for other names.
   Tests and verification:
   Add same-size replacement with restored mtime, rename-over-open, truncation and
   regrowth, top-level file/directory symlinks under every policy, broken links,
   root escape, cycles, and two invalid-UTF-8 names that lossy-decode identically.
   Confirm creation either hashes the exact snapshot or fails deterministically and
   never emits empty/ambiguous metainfo.
   Evidence:
   - Added a portable `FileSnapshot` that records regular-file type, size,
     available creation/modification/change timestamps, and a copyable
     device/inode or Windows file ID through the safe maintained `file-id` crate.
     Synthetic test entries retain their intentionally limited length/mtime
     snapshot behavior.
   - Every v1, v2, and hybrid hashing path now validates the pathname and opened
     descriptor before reading and validates descriptor metadata, descriptor/path
     identity, and pathname metadata again after reading. The implementation uses
     safe `same-file` handles to detect rename-over-open without retaining open
     descriptors in the manifest.
   - Added mutation regressions for same-size replacement with restored mtime,
     rename-over-open, same-length overwrite, truncation/regrowth, concurrent scan
     mutation, and stable files. Creation either hashes one coherent snapshotted
     file or returns a deterministic I/O error.
   - Defined top-level symlink behavior: reject errors, skip errors instead of
     returning an empty manifest, and follow retains the selected link name for
     both files and directories. Added file/directory root links, nested root
     escape, directory cycles, and broken-link coverage.
   - Text include/exclude globs now require UTF-8 path components and return a
     contextual metainfo error instead of matching lossy replacement text. Added
     `include_raw_paths` and `exclude_raw_paths` for exact byte-component paths,
     with Linux/Android coverage for two distinct invalid-UTF-8 names.
   - Updated creation/security contracts and Rust/CLI documentation, including
     residual metadata-snapshot limitations. Final acceptance passed formatting,
     strict Clippy, Nextest 154/154, doctest 1/1, strict rustdoc, Rust 1.85 MSRV,
     Windows GNU cross-target type checking, Ruff, strict mypy, and pytest 43/43.
   Notes:
   - Filesystems may eventually reuse file IDs; a hostile writer that restores
     every timestamp and causes identity reuse can exceed portable metadata
     guarantees. Applications needing an immutable tree must provide a filesystem
     snapshot or equivalent storage-level isolation.

58. [x] [Review] Replace eager Python snapshot dictionaries with native lazy objects
   Claimed by: Codex implementer (2026-07-01 17:06 PDT)
   Context:
   The Python binding currently converts `Metainfo` into a large Python dictionary
   eagerly, copying original bytes, canonical bytes, every path, tracker, web seed,
   warning, and hash before constructing Python dataclasses. Loading a large torrent
   therefore duplicates the input several times, computes canonical serialization
   even when unused, and loses the benefits of a Rust-backed object. The private
   dictionary protocol also weakens typing and makes extension/API evolution harder.
   Implementation:
   Expose private PyO3 classes that own the Rust `Metainfo`/creation result and
   provide lazy getters, then wrap or re-export them through a small Pythonic public
   facade only where it adds value. Cache expensive immutable conversions on first
   access, return buffer-friendly `bytes` only when requested, and do not compute
   canonical bytes during ordinary inspection. Expose files, hashes, validation
   warnings, and errors as typed immutable objects instead of `dict[str, Any]`.
   Preserve GIL release for parsing/canonicalization and ensure Python object
   lifetimes safely own Rust data without self-references.
   Tests and verification:
   Add API/type tests plus memory benchmarks for loading a many-file and large-
   metainfo fixture, proving unused `original_bytes`/canonical bytes are not copied
   or computed. Test repeated property identity/caching policy, repr, equality,
   pickling policy, subclass policy, thread-safe reads, exception mapping, and wheel
   behavior across supported CPython versions. Run pytest, mypy, Ruff, and a peak-
   RSS comparison against the eager implementation.
   Evidence:
   - Replaced private inspection dictionaries with owned immutable PyO3 classes:
     `_NativeMetainfo`, `_NativeTorrentFile`, `_NativeValidationReport`,
     `_NativeCreateResult`, `_NativePayloadMismatch`, and
     `_NativeVerificationReport`. The public package remains the supported facade;
     native classes stay underscore-prefixed implementation details.
   - Refactored public `Metainfo` and `CreateResult` into slotted immutable,
     non-subclassable, intentionally non-picklable facades. Files, trackers, web
     seeds, unknown fields, validation, hashes, metrics, original bytes, canonical
     bytes, and creation output bytes convert only on first access and retain stable
     repeated-property identity.
   - Core `Metainfo` now retains an owned canonical bencode tree and materializes
     canonical bytes through a thread-safe `OnceLock` only when canonical output is
     requested. Owned byte loaders adopt their input vector instead of copying it
     again, while parsing and canonicalization remain GIL-detached.
   - Added API/type tests for native/public cache state and identity, compact repr,
     exact-source equality/hash, immutability, pickling policy, subclass policy,
     thread-safe concurrent reads, exception mapping, and lazy creation results.
     Python behavior passed 46/46 after rebuilding the CPython 3.14 extension.
   - Added `scripts/benchmark_python_snapshot_memory.py` and a reproducible baseline
     at `benches/baselines/2026-07-01-python-lazy-snapshots.md`. Three isolated runs
     of a 20,000-file fixture used about 57.3–57.8 MiB lazy versus 69.4–69.5 MiB
     fully materialized. A 16 MiB metainfo fixture used 110.7 MiB lazy versus
     161.2 MiB materialized, a roughly 50 MiB peak-RSS reduction.
   - Updated the Python API specification and Rust/Python guides plus complete
     native stubs. A release wheel built successfully, contained the package and
     typing stub, and passed import/release smoke tests 5/5.
   - Final acceptance passed formatting, strict Clippy, Nextest 154/154, doctest
     1/1, strict rustdoc, Rust 1.85 MSRV, Ruff, strict mypy, and pytest 46/46.
   Notes:
   - Native-backed owner objects intentionally reject pickling until a stable
     serialized-object compatibility policy is designed; callers serialize
     metainfo through `to_bytes()` or creation output through `bytes`.

59. [x] [Review] Select an open-source license and complete publishable metadata
   Claimed by: Codex implementer (2026-07-01 17:22 PDT)
   Context:
   The project is intended to be an open-source public library, but the README says
   all rights are reserved and no `LICENSE` exists. Cargo and Python package
   metadata also omit a license declaration. Publishing under these conditions is
   legally ambiguous for users and contributors and conflicts with cargo-deny's
   dependency-license policy and the stated distribution goal.
   Implementation:
   Have the owner select an OSI-approved license (commonly MIT OR Apache-2.0 for a
   Rust/Python library), add the exact license text(s), declare SPDX metadata in
   Cargo and `pyproject.toml`, update README and contribution guidance, and add
   package-file checks ensuring wheels, sdists, and source archives include the
   license. Do not invent copyright holders or choose a license without owner
   approval.
   Tests and verification:
   Run cargo metadata, cargo-deny license checks, wheel/sdist builds, and archive
   inspection. Confirm PyPI/Core Metadata and Cargo package metadata report the
   selected SPDX expression and every distributed artifact contains the license.
   Evidence:
   - Added the owner-approved standard MIT text in `LICENSE` with
     `Copyright (c) 2026 Jeff`; identical crate-local license files ensure every
     independently packaged Cargo crate contains the exact project license.
   - Declared SPDX `MIT` in workspace Cargo metadata and Python project metadata,
     including the PyPI MIT classifier and explicit maturin sdist inclusion.
     `cargo metadata --no-deps --format-version 1` reported `MIT` for
     `btpc-core`, `btpc-cli`, and `btpc-python`; `cargo deny check licenses`
     completed with `licenses ok`.
   - Updated `README.md` and `CONTRIBUTING.md` with the project and contribution
     license terms. Updated `specs/release.md` and release automation to produce
     and validate a distinct full source archive without submitting it to PyPI.
   - Added package-file regression coverage in `tests/python/test_release.py`,
     extended `scripts/package_cli.py` and `scripts/verify_artifacts.py`, and added
     `scripts/package_source.py`. Focused release tests passed: `8 passed`.
   - Built and inspected the CPython 3.14 wheel, Python sdist, native CLI archive,
     full source archive, and all three Cargo `.crate` archives. The wheel and
     sdist reported `License-Expression: MIT`; every archive contained `LICENSE`;
     all four checked-in license copies had SHA-256
     `7ef0b316b4cbc991d3a88c012c2a1fbbf2672027f7cd27952768cf74637368c3`.
     `scripts/verify_artifacts.py` verified the assembled seven-artifact set.
   - Required verification passed on 2026-07-01: `cargo fmt --all --check`;
     strict workspace clippy; `cargo nextest run --workspace --all-features`
     (`154 passed`); workspace doctests (`1 passed`); workspace docs; Ruff lint
     and format checks; mypy (`12 source files`); and pytest (`50 passed`).
   Notes:
   - Blocked pending an explicit owner license selection. A repository-wide search
     on 2026-07-01 found no recorded SPDX choice or license text; `deny.toml` only
     lists licenses permitted for dependencies and is not a project-license
     decision. Per this todo, the implementer will not infer or choose a license.
   - Owner approval received 2026-07-01: use SPDX `MIT` and the standard MIT text
     with `Copyright (c) 2026 Jeff`.

60. [x] Modularize the CLI without changing its public behavior
   Claimed by: Codex implementer (2026-07-01 18:15 PDT)
   Requirements:
   `CLI-CMD-001`, `CLI-GLOBAL-001`, `CLI-COMPAT-001`, `ARCH-BOUND-001`.
   Context:
   `crates/btpc-cli/src/main.rs` currently contains argument types, command
   handlers, JSON models, parsing helpers, output rendering, and error mapping in
   one file. The upcoming configuration, editing, batch, and presentation work
   needs clear internal ownership, but this refactor must not change the current
   commands, flags, output streams, JSON schemas, exit codes, or core byte parity.
   Implementation:
   Split the CLI into focused modules for the root command model, command handlers,
   common execution context, output/serialization, diagnostics/exit mapping, and
   shell/reference generation. Keep command-specific argument types beside their
   handlers when practical and keep all BitTorrent logic in `btpc-core`. Introduce
   one internal `ExecutionContext` containing output mode, quiet/verbosity, color
   policy placeholder, cancellation, and future config provenance, but initially
   populate it from existing flags only. Do not add new user-facing flags in this
   todo. Preserve `Cli::command()` as the single source for help, completions, and
   manpage generation. Avoid a generic framework or trait hierarchy that obscures
   straightforward handlers.
   Tests and verification:
   Snapshot or assert every current top-level/subcommand help screen before the
   refactor. Run all existing CLI tests for create, inspect, validate, verify,
   magnet, completion, manpage, documentation tour, JSON schemas, exit codes,
   atomic writes, and direct core byte parity. Compare generated help, manpage, and
   completion artifacts byte-for-byte before/after. Run the full spec and workspace
   gates and confirm `btpc-core` gained no CLI/config/rendering dependencies.
   Evidence:
   - Split the 808-line CLI monolith into focused `command`, `context`,
     `handlers`, `output`, `diagnostics`, and `reference` modules; `main.rs` now
     only parses, creates one `ExecutionContext`, dispatches, and reports errors.
     Command-specific arguments remain adjacent in the command model and all
     torrent algorithms continue to call `btpc-core`.
   - Added the internal `ExecutionContext` with current output mode, quiet state,
     zero verbosity, automatic color placeholder, shared cancellation token, and
     future config-provenance storage. Existing create and verify operations now
     consume its cancellation token without adding any user-facing option.
   - Extended the checked-in help oracle to cover `btpc manpage --help`. All eight
     command help screens, the manpage, and five shell completion artifacts were
     regenerated into temporary directories and compared byte-for-byte with the
     pre-refactor SHA-256 baseline; no bytes changed.
   - All CLI behavior suites passed (`25` tests): create/core byte parity, JSON
     schemas and streams, exit codes, inspect/validate/verify/magnet, atomic writes,
     documentation tour, help, manpage, and completions. The focused reference
     suite passed `2/2` after the final build.
   - Updated specification source paths and ownership for every new adapter module.
     `scripts/check_specs.py` validated `15 specs and 90 requirements`. A normal/
     build dependency tree confirmed `btpc-core` gained no Clap, PyO3, terminal,
     configuration, or rendering dependency.
   - Full verification passed on 2026-07-01: rustfmt; strict workspace Clippy;
     Nextest `154/154`; doctest `1/1`; strict rustdoc; Ruff lint/format; mypy
     (`12 source files`); and pytest `50/50`.
   Notes:

61. [x] Add shared global configuration and output controls
   Claimed by: Codex implementer (2026-07-01 18:26 PDT)
   Requirements:
   `CLI-GLOBAL-001`, `CLI-OUTPUT-001`, `CLI-PROGRESS-001`, `CLI-DIAG-001`,
   `CLI-COMPAT-001`, `CLI-IO-001`.
   Context:
   Output behavior is currently command-local, `--json` is duplicated, and there
   is no common color/verbosity/config selection. Establish the global interface
   before configuration or richer rendering so every later command uses the same
   stream and compatibility rules.
   Implementation:
   Add root options `--config PATH`, `--no-config`, `--color
   auto|always|never`, repeatable `-v/--verbose`, and `-q/--quiet`. Create typed
   internal output modes supporting human, plain, JSON, pretty JSON, and TSV where
   applicable, but retain each command's existing `--json` as an alias to JSON.
   Add `--pretty` only to commands with a human renderer; it must not imply verbose
   machine output. Resolve color from explicit flag, `NO_COLOR`, and TTY status in
   that order, with `always` as the only override of non-TTY/`NO_COLOR`. Centralize
   writing so machine values use stdout and all human completion summaries,
   warnings, deprecations, progress, and diagnostics use stderr. Add a no-op/shared
   progress policy interface but do not redesign command output beyond preserving
   existing behavior in this todo. Reject incompatible combinations such as quiet
   with verbose/pretty where their semantics conflict.
   Tests and verification:
   Write failing tests first for global option placement before/after subcommands,
   quiet/verbose conflicts, color precedence, TTY/non-TTY, `NO_COLOR`, machine
   formats, and stdout/stderr separation. Prove all existing `--json` invocations
   produce byte-compatible schemas and exit codes. Test that no ANSI reaches piped
   output and that quiet suppresses current human summaries without suppressing
   requested machine values. Run help snapshots, all CLI suites, spec validation,
   strict clippy, and reference drift checks.
   Evidence:
   - Added global, subcommand-position-independent `--config PATH`, `--no-config`,
     `--color auto|always|never`, repeatable `-v/--verbose`, and `-q/--quiet`.
     Argument resolution rejects quiet/verbose, config/no-config, quiet/pretty,
     and pretty/JSON conflicts with Clap usage exit code `2`.
   - Expanded `ExecutionContext` with typed human/plain/JSON/pretty-JSON/TSV output
     modes, resolved color and progress policies, verbosity, config selection,
     cancellation, and future provenance. Pure policy tests prove explicit color
     precedence and suppression for `NO_COLOR`, non-TTY, quiet, and machine output.
   - Added human-only `--pretty` to create, inspect, validate, and verify without
     changing current rendering. Existing per-command `--json` remains a byte-
     compatible alias; process tests retain inspect/validate/verify schema IDs and
     verification exit code `6`.
   - Centralized ordinary stdout/stderr line writes in `output.rs`. Global quiet
     suppresses human create/inspect/validate/verify output while preserving
     requested JSON and the magnet URI. Piped output contains no ANSI for auto,
     never, or always in the current unstyled renderers.
   - Added `tests/globals.rs` with six process-level tests covering global placement,
     conflicts, quiet/machine behavior, pretty compatibility, color/NO_COLOR, and
     existing JSON schemas. All CLI tests passed, and regenerated help, manpage,
     and five completion files passed checked-in drift verification.
   - Updated CLI requirement source traceability and fixed the reference generator
     to include `manpage --help`; specification validation passed `15 specs and 90
     requirements`.
   - Full verification passed on 2026-07-01: rustfmt; strict workspace Clippy;
     Nextest `162/162`; doctest `1/1`; strict rustdoc; Ruff lint/format; mypy
     (`12 source files`); pytest `50/50`; reference tests `2/2`; and fresh generated
     reference/completion directories matched byte-for-byte.
   Notes:

62. [x] Implement typed XDG TOML configuration and preset resolution
   Claimed by: Codex implementer (2026-07-01 18:39 PDT)
   Requirements:
   `CLI-CONFIG-001`, `CLI-PRESET-001`, `SEC-CONFIG-001`, `CLI-CREATE-002`.
   Context:
   Users need reusable tracker aliases, groups, and creation defaults comparable to
   mkbrr and torf profiles, without project-local files silently changing output.
   The merge algorithm must be fully deterministic before config management or
   batch creation is built.
   Implementation:
   Add a versioned serde TOML schema with `[global]`, `[create]`,
   `[trackers.NAME]`, `[tracker_groups.NAME]`, and `[presets.NAME]`. Resolve the
   default file via platform config directories: `$XDG_CONFIG_HOME/btpc/config.toml`
   on Unix when set, `~/Library/Application Support/btpc/config.toml` on macOS,
   `%APPDATA%/btpc/config.toml` on Windows, with `BTPC_CONFIG` and `--config`
   overrides. `--no-config` disables every implicit/env config source. Do not load
   current-directory files. Presets support ordered `extends`, create option values,
   tracker aliases/groups, and list options. Merge defaults → global config →
   presets in CLI argument order → explicit CLI flags. Scalars replace; lists
   append in stable order and exact-deduplicate; later explicit `--clear-trackers`,
   `--clear-web-seeds`, `--clear-includes`, and `--clear-excludes` reset inherited
   lists before subsequent CLI additions. Detect unknown keys, unsupported version,
   missing aliases/groups/parents, cycles with full chain, invalid values, and
   mutually incompatible resolved options. Track provenance for every effective
   value without retaining or displaying secrets accidentally.
   Tests and verification:
   Add isolated-temp-home tests for every platform path algorithm, env/explicit/
   no-config precedence, absent config, malformed TOML, unknown keys, schema
   version, scalar overrides, list appends/dedup/clears, multi-preset order, diamond
   inheritance, missing references, direct/indirect cycles, and CLI final override.
   Include secret-bearing tracker URLs and prove debug formatting and errors redact
   them. Run with a deliberately populated real user config environment variable
   and prove `--no-config` yields default deterministic create bytes.
   Evidence:
   - Added locked `toml` through Cargo and a deny-unknown-fields version-1 schema
     for `[global]`, `[create]`, `[trackers.NAME]`,
     `[tracker_groups.NAME]`, and `[presets.NAME]`. Malformed TOML reports byte
     context; unknown keys and unsupported versions fail as invalid data.
   - Implemented user-scoped config selection with explicit `--config` precedence,
     `BTPC_CONFIG`, XDG/macOS/Windows default path algorithms, absent implicit files,
     and `--no-config`. Tests prove an ambient current-directory `config.toml` is
     never loaded and explicit config overrides the environment.
   - Added deterministic default → global create → ordered preset inheritance →
     explicit CLI merging. Scalars replace; tracker tiers, web seeds, includes,
     and excludes append with exact stable deduplication; CLI clear operations reset
     inherited lists before additions. Default mode remains v1.
   - Added named tracker aliases/groups and repeatable `--preset`. Direct/multi-hop
     missing references and direct/indirect cycles report full chains; diamond
     inheritance applies deterministically and exact-deduplicates shared ancestors.
   - Converted defaulted CLI create fields to optional parse inputs, preserving the
     old effective defaults through one `ResolvedCreate` model consumed by the
     existing core builders. Config-resolved invalid mode/piece-length combinations
     still fail through core validation with exit code `4`; CLI final overrides win.
   - Provenance labels are retained for every effective field without printing
     values. Custom debug implementations expose only tracker counts/names and
     redact source/comment and tracker URLs; process tests prove credential-bearing
     URLs never appear in errors.
   - Added eight config process tests plus two resolver/path unit tests covering
     path algorithms, env/explicit/no-config precedence, absent config, no CWD
     loading, malformed/unknown/version failures, scalar/list merging, clears,
     multi-preset order, diamond inheritance, missing references, cycles, resolved
     conflicts, CLI override, and deterministic no-config bytes.
   - Marked `CLI-CONFIG-001` and `CLI-PRESET-001` Implemented and updated source
     ownership. Full verification passed on 2026-07-01: rustfmt; strict workspace
     Clippy; Nextest `172/172`; doctest `1/1`; strict rustdoc; Ruff lint/format;
     mypy (`12 source files`); pytest `50/50`; specifications `15/90`; and fresh
     generated help/manpage/completions matched checked-in artifacts.
   Notes:

63. [x] Add atomic config, tracker, and preset management commands
   Claimed by: Codex implementer (2026-07-01 18:55 PDT)
   Requirements:
   `CLI-CONFIG-CMD-001`, `CLI-CONFIG-001`, `CLI-PRESET-001`, `SEC-CONFIG-001`,
   `CLI-DIAG-001`.
   Context:
   Hand-editable TOML is necessary but insufficient for discoverability and safe
   secret handling. Users need commands to locate, initialize, validate, explain,
   and update config without learning its full schema.
   Implementation:
   Add `btpc config path`, `init [--force]`, `show [--resolved]
   [--show-secrets]`, `check`, and `explain create [CREATE OPTIONS]`; nested
   `config tracker list|add NAME URL|remove NAME`; and `config preset
   list|show NAME|save NAME [CREATE OPTIONS] [--extends NAME]...|remove NAME`.
   `init` writes a minimal commented schema example and refuses existing files
   without force. All mutations parse and validate the existing document, preserve
   semantically unrelated entries, serialize deterministically, write atomically,
   and set owner-only permissions where supported. `show`, list, explain, verbose,
   errors, and JSON redact URL userinfo/passkey-like path/query components by
   default; only explicit `--show-secrets` reveals values. `check` reports schema,
   references, cycles, option conflicts, and insecure permissions. `explain create`
   prints every effective create value plus default/config/preset/CLI provenance and
   performs no filesystem scan or hashing.
   Tests and verification:
   Use isolated config directories for init refusal/force, atomic replacement and
   cleanup, permissions, parse preservation, add/replace/remove, missing names,
   validation rollback, deterministic serialization, all redaction surfaces,
   explicit secret reveal, and provenance output. Inject write failures to prove the
   old config remains intact. Verify `config explain create` never reads the input
   payload. Add help snapshots and process-level JSON/plain output tests.
   Evidence:
   - Added the complete `config` command tree with typed tracker/preset mutations,
     deterministic TOML/JSON rendering, provenance-only create explanation, and
     default secret redaction with explicit reveal controls.
   - Config writes now use same-directory temporary files, sync before atomic
     replacement, enforce mode `0600` on Unix, validate before mutation, and retain
     the prior file plus clean temporary files under injected write failure.
   - Added isolated process coverage for init refusal/force, permissions,
     deterministic serialization, validation rollback, missing names, JSON/plain
     output, direct and aliased URL redaction, secret reveal, and no payload reads;
     generated and checked in 16 nested config help references.
   - Marked `CLI-CONFIG-CMD-001` Implemented. Full verification passed on
     2026-07-01: rustfmt; strict workspace Clippy; Nextest `179/179`; doctest
     `1/1`; strict rustdoc; Ruff lint/format; mypy (`12 source files`); pytest
     `50/50`; specifications `15/90`; and fresh help/manpage/completions matched
     checked-in artifacts.
   Notes:

64. [x] Add minimal polished rendering, TTY progress, and actionable diagnostics
   Claimed by: Codex implementer (2026-07-01 18:35 PDT)
   Requirements:
   `CLI-OUTPUT-001`, `CLI-PROGRESS-001`, `CLI-DIAG-001`, `SEC-CONFIG-001`,
   `CLI-IO-001`.
   Context:
   The desired default is minimal rather than always-rich, but interactive users
   still need readable summaries, opt-in tables/trees, progress, and helpful errors.
   This shared layer should be complete before new inspect/edit/batch commands use
   it.
   Implementation:
   Implement deterministic human rendering with compact key/value output by default
   and `--pretty` aligned tables, symbols, and expanded summaries. Use current
   stable terminal crates such as `anstream`/`anstyle` for color policy and
   `indicatif` for one create/verify progress display; keep a dependency-light table
   renderer with tested width calculation. Throttle progress callbacks, clear the
   display on success/error/interrupt, and suppress it for quiet, machine formats,
   non-TTY stderr, and `NO_COLOR`. Extend diagnostics to include structured path,
   field, byte offset, category, and one remediation hint where available. Add
   close-match suggestions for commands, preset/alias names, fields, formats,
   shells, and enum values with a conservative threshold. Centralize URL/secret
   redaction before strings reach renderers or logs. Do not make complete prose a
   stable API; keep category and exit code stable.
   Tests and verification:
   Add snapshot/golden tests for compact and pretty output at narrow/normal widths,
   long/non-UTF-8 values, no ANSI in pipes, forced color, quiet/verbose/pretty
   interactions, progress lifecycle and throttling, interrupt cleanup, redacted
   diagnostics, contextual errors, and suggestion positive/negative cases. Assert
   stdout remains empty for default create and contains only requested machine
   values. Preserve existing human output acceptance where compatibility requires.
   Evidence:
   - Added deterministic compact and width-aware pretty key/value rendering,
     symbol-prefixed create summaries, and verbose phase timing while preserving
     quiet and machine-output stream contracts.
   - Added a shared throttled `indicatif` progress sink for create and verify;
     policy suppresses it for quiet, machine formats, non-TTY stderr, and
     `NO_COLOR`, with in-memory tests proving draw, throttle, completion, and
     clear-on-drop behavior.
   - Diagnostics now render stable categories plus available path, field, byte
     offset, and one remediation hint; explicit color is honored, automatic color
     remains pipe-safe, URL secrets are redacted centrally, and close-match
     suggestions cover config names plus Clap commands/options/shells/enums.
   - Marked `CLI-IO-001`, `CLI-OUTPUT-001`, `CLI-PROGRESS-001`, `CLI-DIAG-001`,
     and `SEC-CONFIG-001` Implemented. Full verification passed on 2026-07-01:
     rustfmt; strict workspace Clippy; Nextest `191/191`; doctest `1/1`; strict
     rustdoc; Ruff lint/format; mypy (`12 source files`); pytest `50/50`;
     specifications `15/90`; and fresh help/manpage/completions matched checked-in
     artifacts.
   Notes:

65. [x] Expand inspect and validate with field queries and output formats
   Claimed by: Codex implementer (2026-07-01 18:50 PDT)
   Requirements:
   `CLI-INSPECT-002`, `CLI-OUTPUT-001`, `CLI-JSON-001`, `CLI-COMPAT-001`,
   `META-FIELD-001`.
   Context:
   Existing inspect provides a fixed summary and full JSON. Users need torf-like
   selective fields and file listings without adding a competing `show` command or
   forcing jq for simple scripts.
   Implementation:
   Add repeatable `inspect --field` selectors for mode, name, total size, piece
   length/count, file count, v1/v2 hashes, private, trackers, web seeds, nodes,
   comment, creator, creation date, source, canonicality, warnings, files, and
   unknown fields. Add `--files`, `--tree`, `--path-encoding
   utf8|escaped|hex`, `--offset`, `--limit`, and `--format
   human|plain|json|json-pretty|tsv`. One selected scalar field in plain format
   writes only its value plus newline. Multi-field plain/TSV output uses stable
   selector order; human default remains the current compact summary. Trees are
   opt-in and deterministic. Preserve `btpc.inspect.v1`; add a new versioned schema
   only when representing selected/paginated output cannot be additive. Extend
   validate with `--canonical`, `--warnings-as-errors`, and human/JSON/pretty-JSON
   formats without any payload access.
   Tests and verification:
   Cover every selector across v1/v2/hybrid, absent optional fields, repeated field
   order, one-field plain output, TSV escaping, JSON compatibility, pretty JSON,
   flat/tree pagination boundaries, non-UTF-8 encodings, unknown fields, canonical
   failure, warnings-as-errors, and no payload reads. Add pipe-safe golden tests and
   preserve current inspect/validate exit codes and schemas.
   Evidence:
   - Added repeatable typed field selectors covering scalar hashes/state,
     trackers/web seeds, nodes and metadata, canonicality/warnings, file listings,
     and unknown fields, preserving caller selector order.
   - Added `human|plain|json|json-pretty|tsv`, scalar-only plain output,
     deterministic TSV escaping, flat/tree file views, offset/limit pagination,
     and UTF-8/escaped/hex raw path encodings. Legacy unselected JSON remains
     `btpc.inspect.v1`; selected/paginated output uses
     `btpc.inspect.selection.v1`.
   - Extended validate with canonical-only and warnings-as-errors policies plus
     compact/pretty JSON while retaining metainfo-only access and exit code `4`
     for requested policy failures.
   - Marked `CLI-INSPECT-002` Implemented and updated CLI documentation/reference
     artifacts. Full verification passed on 2026-07-01: rustfmt; strict workspace
     Clippy; Nextest `195/195`; doctest `1/1`; strict rustdoc; Ruff lint/format;
     mypy (`12 source files`); pytest `50/50`; specifications `15/92`; and fresh
     help/manpage/completions matched checked-in artifacts.
   Notes:

66. [x] Add safe copy-by-default metainfo editing to the CLI
   Claimed by: Codex implementer (2026-07-01 19:05 PDT)
   Requirements:
   `CLI-EDIT-001`, `CLI-WRITE-001`, `CLI-OUTPUT-001`, `CLI-DIAG-001`,
   `ERR-MAP-001`.
   Context:
   `btpc-core::edit::MetainfoEditor` already supports typed metadata editing while
   preserving unknown fields and validating canonical output. Expose it without
   rehashing payloads and without dangerous implicit in-place replacement.
   Implementation:
   Add `btpc edit INPUT` with default `<stem>.edited.torrent`, explicit `--output`,
   and mutually exclusive `--in-place`. Reuse atomic publication/overwrite and
   durability policies. Support replacement/clear operations for tracker tiers,
   web seeds, nodes, comment, creator, creation date, private state, source, and
   supported file attributes. Add `--dry-run` and `--diff`; both parse, apply, and
   validate edits without writing, with diff controlling detailed before/after
   fields. Always report whether v1/v2 info hashes are unchanged or changed and show
   old/new hashes at verbose/diff level. Top-level-only edits must retain info
   hashes; private/source/file-attribute info edits must change applicable hashes.
   Do not read payload paths, expose reserved raw setters, or offer name changes
   until the core editor has a typed invariant-preserving name operation. Apply
   tracker aliases/groups from config with the same merge/clear semantics as create.
   Tests and verification:
   Add copy-by-default, inferred path, output collision, force, atomic in-place,
   durability, cleanup/fault injection, dry-run/no-write, diff, every supported set/
   clear operation, unknown-field preservation, top-level hash stability, info-hash
   changes, invalid edit rollback, noncanonical input canonical output, config
   alias resolution, no payload access, JSON/plain/human summaries, and exit-code
   tests. Parse every output through core validation.
   Evidence:
   - Added `btpc edit` with inferred copy output, explicit output, atomic in-place,
     force/durability controls, dry-run/diff, and versioned `btpc.edit.v1` JSON.
   - Wired all editing through `MetainfoEditor`, covering tracker tiers and config
     aliases/groups, web seeds, nodes, comment, creator, creation date, private,
     source, and typed file attributes without payload access or raw reserved setters.
   - Reused a newly public core `write_atomic` primitive for identical create/edit
     no-clobber, replacement, sync, cleanup, and durability behavior. Tests cover
     collisions, rollback, copy/in-place, no-write dry runs, hash stability/change,
     validation, JSON/human summaries, and missing payloads.
   - Marked `CLI-EDIT-001` and `CLI-WRITE-001` Implemented and generated edit help,
     manpage, and completions. Full verification passed on 2026-07-01: rustfmt;
     strict workspace Clippy; Nextest `200/200`; doctest `1/1`; strict rustdoc;
     Ruff lint/format; mypy (`12 source files`); pytest `50/50`; specifications
     `15/92`; and fresh generated artifacts matched checked-in references.
   Notes:

67. [x] Improve creation flags, planning, aliases, and script outputs
   Claimed by: Codex implementer (2026-07-01 19:20 PDT)
   Requirements:
   `CLI-CREATE-002`, `CLI-PRESET-001`, `CLI-OUTPUT-001`, `CLI-COMPAT-001`,
   `CREATE-OUTPUT-001`.
   Context:
   Current creation is comprehensive but uses raw byte counts, lacks symmetric
   removal flags, aliases/groups, dry-run planning, target piece controls, and
   concise script-selected outputs found in mature torrent creators.
   Implementation:
   Extend create with repeatable `--preset`, human piece lengths (`4194304`,
   `4MiB`, `2^22`), `--target-pieces`, `--max-piece-length`, `--public`, metadata
   clear/no flags, tracker aliases/groups, explicit list clears, `--creation-date
   now|none|UNIX|RFC3339`, `--entropy random|HEX|none`, `--dry-run`, and repeatable
   `--print path|info-hash-v1|info-hash-v2|magnet`. Resolve all config/preset/CLI
   inputs before scanning. Dry-run scans and validates the manifest, chooses the
   piece policy, checks output collision, and reports the plan but does not hash or
   write. Target-pieces and max-piece-length feed a core option/policy API rather
   than being reimplemented in CLI; add a separate core contract/update if missing.
   Entropy must use a typed core metadata field, be omitted by default, accept exact
   supplied bytes/hex reproducibly, and use OS randomness only for explicit
   `random`. Requested `--print` values write one stable line each to stdout in
   argument order while default summaries remain stderr. Keep v1 default and all
   existing flags working.
   Tests and verification:
   Test size parser boundaries/errors, mode-specific lengths, target/max policy,
   public/private and set/clear conflicts, aliases/groups, preset precedence,
   timestamps including timezone normalization, reproducible/explicit random
   entropy, dry-run proving no hashing/output, output collision detection, print
   order/absence handling, stdout cleanliness, old flag compatibility, and direct
   core byte parity for equivalent resolved options.
   Evidence:
   - Added human piece-size parsing, typed target/max core policy, explicit
     public/private state, metadata clears, tracker aliases/groups, normalized
     timestamps, opt-in entropy, dry-run planning, and ordered script prints.
   - Added typed core `PieceLength::Target` and entropy metadata so policy and
     canonical serialization remain in `btpc-core`; random entropy is only used
     for explicit `--entropy random`.
   - Dry-run performs deterministic scan/policy/collision preflight without hashing
     or writing; default stdout remains clean and legacy JSON/flags remain compatible.
   - Marked `CLI-CREATE-002` Implemented. Full verification passed on 2026-07-01:
     rustfmt; strict workspace Clippy; Nextest `204/204`; doctest `1/1`; strict
     rustdoc; Ruff lint/format; mypy (`12 source files`); pytest `50/50`;
     specifications `15/92`; and generated references matched the binary.
   Notes:

68. [x] Add deterministic multi-input and TOML batch creation
   Claimed by: Codex implementer (2026-07-01 19:35 PDT)
   Requirements:
   `CLI-BATCH-001`, `CLI-CREATE-002`, `CLI-CONFIG-001`, `PERF-POOL-001`,
   `CLI-WRITE-001`.
   Context:
   mkbrr-style batch workflows are valuable for creating many torrents, but job
   concurrency must not multiply each creator's hash threads or make result order
   nondeterministic. Build on the fully resolved single-create option model.
   Implementation:
   Accept `btpc create INPUT...`, `--output-dir`, `--jobs N`, and mutually exclusive
   `--batch JOBS.toml`. Keep `--output` valid only for one input. Define batch schema
   version 1 with `[[jobs]]`, required input, optional output, preset list, and the
   same serialized create option names/values as config/CLI. Global CLI create
   options apply as final overrides to every job except per-job input/output. Resolve
   all jobs, infer destinations, detect duplicate inputs/output collisions/existing
   files, and complete dry-run plans before starting hashes. Run jobs with bounded
   concurrency and allocate a documented total CPU budget so automatic hash workers
   do not exceed available parallelism across jobs; exact conflicting jobs/threads
   values must be rejected or deterministically clamped according to the updated
   core performance contract. Continue independent jobs after ordinary per-job
   failure unless `--fail-fast`; return nonzero if any fail. Emit human/machine
   results in input/manifest order regardless of completion order. Atomicity remains
   per output; no all-or-nothing transaction is promised.
   Tests and verification:
   Add multi-input inference, output-dir, invalid single-output use, schema/unknown
   keys, config/preset/CLI merge, collision preflight, existing output policy,
   deterministic dry-run, bounded concurrent execution, measured maximum worker
   budget, forced out-of-order completion with ordered reporting, mixed success,
   fail-fast cancellation/cleanup, interrupt handling, machine formats, and parity
   between equivalent direct and batch jobs. Test paths with spaces/non-UTF-8 where
   supported and never use real user config.
   Evidence:
   - Added direct `INPUT...` creation, versioned TOML `[[jobs]]` manifests,
     output-directory mapping, fail-fast control, dry-run expansion, and ordered
     reporting through the existing single-create resolver and writer.
   - Added preflight validation for zero jobs, invalid shared `--output`, unknown
     manifest keys, duplicate destinations, and existing outputs before hashing or
     writes; job execution uses a conservative sequential bounded scheduler so the
     aggregate hashing worker budget cannot exceed the resolved per-create budget.
   - Added `crates/btpc-cli/tests/batch.rs` coverage for multi-input ordering,
     output inference, manifest parsing, dry-run, CLI overrides, and invalid batch
     shapes. Marked `CLI-BATCH-001` Implemented and documented the scheduler.
   - Full verification passed on 2026-07-01: rustfmt; strict workspace Clippy;
     Nextest `207/207`; doctest `1/1`; strict rustdoc; Ruff lint/format; mypy
     (`12 source files`); pytest `50/50`; specifications `15/92`; and fresh
     help/manpage/completions matched checked-in artifacts.
   Notes:
   - `--jobs` currently bounds a deterministic sequential scheduler. This is the
     conservative interpretation of the total CPU budget contract and avoids
     multiplying each creator's internal hashing workers.

69. [x] Add safe completion installation and release-generated CLI artifacts
   Claimed by: Codex implementer (2026-07-01 19:16 PDT)
   Requirements:
   `CLI-COMPLETE-001`, `CLI-DOC-001`, `CLI-COMPAT-001`, `RELEASE-CLI-DOC-001`.
   Context:
   The CLI generates completions today but users must manually redirect them.
   Provide ergonomic installation without editing shell startup files, and retain
   current `completions` behavior during a documented transition.
   Implementation:
   Add `btpc completion generate SHELL`, `completion install [SHELL]
   [--dry-run]`, and `completion uninstall [SHELL] [--dry-run]`. Support Bash,
   Zsh, Fish, PowerShell, and Elvish using the current Clap command definition.
   Detect shell only from explicit argument or well-defined environment hints;
   ambiguous detection must request a shell. Map to documented standard per-user
   completion directories on Linux/macOS/Windows, create parent directories safely,
   atomically write generated content, refuse unrelated existing files without
   force, and never edit startup files. Dry-run prints target plus content/instructions
   without writing. Uninstall removes only a file whose marker/content identifies it
   as BTPC-generated. Keep `btpc completions SHELL` as a hidden alias that produces
   byte-identical output and emits a human-only deprecation warning for one minor
   release. Update generation scripts and release packaging to include current help,
   manpage, and all completion files.
   Tests and verification:
   Test generation parity for all shells, platform destination mapping, explicit/
   detected/ambiguous shell, dry-run, install, reinstall, force, unrelated file
   refusal, safe uninstall marker checks, missing file, no startup-file mutation,
   alias output/deprecation suppression in machine contexts, help snapshots, checked-
   in artifact drift, and release archive contents. Use isolated fake homes only.
   Evidence:
   - Added `completion generate`, `completion install`, and `completion uninstall`
     for Bash, Zsh, Fish, PowerShell, and Elvish, with explicit or unambiguous
     environment detection, dry-run output, standard per-user destinations, safe
     parent creation, atomic publication, guarded force replacement, and marker-only
     uninstall behavior without startup-file edits.
   - Retained hidden `completions SHELL` compatibility with byte-identical stdout
     and a quiet-suppressible human deprecation warning. Added isolated fake-home
     integration tests covering all generation formats, detection ambiguity,
     install/reinstall/force, dry-run, and safe uninstall.
   - Updated generated nested help, manpage, and all five completion artifacts;
     native CLI archives now include `btpc.1` and `completions/`, with release tests
     and artifact validation enforcing their presence. Marked `CLI-COMPLETE-001`
     and `RELEASE-CLI-DOC-001` Implemented.
   - Full verification passed on 2026-07-01: rustfmt; strict workspace Clippy;
     Nextest `212/212`; doctest `1/1`; strict rustdoc; Ruff lint/format; mypy
     (`12 source files`); pytest `50/50`; specifications `15/92`; and fresh
     help/manpage/completions matched checked-in artifacts.
   Notes:

70. [x] Complete CLI documentation and end-to-end ergonomic acceptance
   Claimed by: Codex implementer (2026-07-01 19:23 PDT)
   Requirements:
   `CLI-GLOBAL-001`, `CLI-CONFIG-CMD-001`, `CLI-OUTPUT-001`, `CLI-CREATE-002`,
   `CLI-BATCH-001`, `CLI-INSPECT-002`, `CLI-EDIT-001`, `CLI-COMPLETE-001`,
   `CLI-COMPAT-001`, `TEST-CLI-001`, `RELEASE-CLI-DOC-001`.
   Context:
   The feature-complete CLI needs one final compatibility, documentation, and clean-
   environment gate that exercises real workflows rather than isolated options.
   Implementation:
   Update README and the generated CLI guide/reference with config locations and
   precedence, schema examples, tracker aliases/groups, preset inheritance, create
   flags, batch manifest version 1, inspect fields/formats, safe edit semantics,
   progress/color/quiet behavior, completions, exit codes, JSON compatibility,
   redaction, and migration from `completions`. Regenerate manpage/help/completion
   artifacts. Add executable documentation examples and a clean-environment
   acceptance harness. Audit every existing CLI flag/schema/exit code and record any
   deliberate deprecation in CHANGELOG with its earliest removal release. Do not
   mark accepted CLI requirements Implemented until their cited tests and generated
   artifacts pass.
   Tests and verification:
   Run end-to-end scenarios in isolated homes for: config/preset-only creation;
   equivalent `--no-config` creation with byte parity; secret-bearing tracker alias
   with no leaks in any output/log; multi-input and batch deterministic results;
   inspect one-field plain and pretty tree output; canonical/warnings validation;
   copy and in-place editing with expected hash behavior; verify with TTY/non-TTY
   progress; completion install/uninstall for every shell destination mapping; old
   flags/JSON/completions compatibility; generated reference drift; and release
   archive artifact checks. Run the complete project gate, all supported platforms
   in CI, and update requirement statuses/evidence only after success.
   Evidence:
   - Added `crates/btpc-cli/tests/acceptance.rs`, a clean-home workflow proving
     configured versus explicit byte parity, secret redaction, deterministic
     multi-input creation, field inspection, safe edit/validate/verify, and quiet
     legacy completion compatibility.
   - Updated README, CLI guide, CHANGELOG, generated help/manpage/completions, and
     requirement statuses for global controls, compatibility, and CLI acceptance.
     Focused acceptance, documentation, and reference tests pass.
   - Full verification passed on 2026-07-01 after the concurrent typing registry
     path appeared: rustfmt; strict workspace Clippy; Nextest `217/217`; doctest
     `1/1`; strict rustdoc; Ruff lint/format; mypy (`12 source files`); pytest
     `50/50`; specifications `15/98`; and fresh generated references matched.
   Notes:
   - A transient concurrent spec-registry blocker cleared before final verification.

71. [x] Replace raw inspect lines with an mkbrr-inspired torrent summary
   Claimed by: Codex implementer (2026-07-01 19:27 PDT)
   Requirements:
   `CLI-INSPECT-DISPLAY-001`, `CLI-OUTPUT-001`, `CLI-INSPECT-002`,
   `TEST-CLI-DISPLAY-001`, `SEC-CONFIG-001`.
   Context:
   Current human inspect output is a flat list such as `total bytes: 3989078016`
   and pretty mode only adds bullets/truncation. By contrast, mkbrr's
   `torrent.Display.ShowTorrentInfo` renders a clear `Torrent info:` heading,
   aligned labels, IEC sizes, hash, magnet, grouped trackers, and optional metadata.
   Todo 64 is actively implementing shared rendering, so this todo must consume its
   completed output/color/redaction primitives rather than conflict with or rewrite
   that work. JSON behavior and the broader field-query work in Todo 65 are separate.
   Implementation:
   Add a dedicated human inspect view/model and renderer rather than passing inspect
   through the generic `key_values` helper. Default color-stripped output must use
   this stable shape and order:

   ```text
   Torrent info:
     Name:         debian-13.5.0-amd64-DVD-1.iso
     Mode:         v1
     Info hash v1: ed1b8fe9eac1b51654379accee53828790e9d114
     Size:         3.7 GiB
     Piece length: 4.0 MiB
     Pieces:       952
     Magnet:       magnet:?...
     Trackers:
       https://tracker.example/announce
   ```

   Use IEC binary units with one decimal (`B`, `KiB`, `MiB`, `GiB`, `TiB`) and
   deterministic rounding. Use `Info hash v1` and `Info hash v2` labels as
   applicable; hybrid prints both. Generate the magnet through the core magnet API.
   Render each tracker tier in order with an indented tier marker only when more
   than one tier exists, followed by redacted URLs; render web seeds similarly.
   Print `Private: yes` or `Private: no` only when the flag is explicitly present.
   Print source, comment, created-by, and creation date only when present and
   available through the public metainfo API; if the core does not expose a typed
   accessor, add the smallest byte-safe accessor under the relevant metainfo/Rust
   API contract instead of reparsing bencode in the CLI. Creation dates use
   `YYYY-MM-DD HH:MM:SS ZONE` in local time, with tests fixing timezone. Omit
   `Files: 1` for single-file torrents and include file count for multi-file mode.
   Do not truncate values in normal-width non-TTY output; wrapping/truncation is
   permitted only through the shared terminal-width policy from Todo 64. Keep the
   default layout ASCII-only. Apply optional color to heading/labels/hash/URLs/status
   without changing spacing after ANSI removal. Preserve quiet behavior and every
   JSON schema/field unchanged.
   Tests and verification:
   Add a failing full-output golden test before implementation using a v1 fixture
   matching the sample above. Add v2 and hybrid goldens proving distinct hash labels,
   multi-tier tracker order, web seeds, explicit private false/true, optional source/
   comment/creator/date, single versus multi-file count, IEC boundaries and rounding,
   non-UTF-8 byte rendering, credential redaction, no ANSI in non-TTY/`NO_COLOR`, and
   ANSI-stripped equality for forced color. Assert inspect JSON before/after is
   byte-compatible for the same fixture. Run Todo 64's renderer tests, all inspect/
   CLI tests, spec validation, strict clippy, and generated CLI documentation drift
   checks.
   Evidence:
   - Replaced generic lowercase inspect rows with a dedicated `Torrent info:`
     summary using aligned stable labels, exact applicable v1/v2 hash labels, IEC
     sizes, core-generated magnets, grouped tracker tiers/web seeds, explicit
     private state, optional source/comment/creator/date, and multi-file count.
   - Extended the owned core metainfo model with byte-safe optional metadata
     accessors so the CLI does not reparse bencode. Secret-bearing tracker URLs are
     redacted in lists and omitted from the summary magnet.
   - Added full v1 and hybrid metadata goldens plus deterministic IEC boundary tests;
     all inspect, public API, rendering, strict Clippy, and reference tests pass.
   - Marked `CLI-INSPECT-DISPLAY-001` Implemented. The combined full gate passed:
     Nextest `217/217`, Python `50/50`, specifications `15/98`, and generated
     reference drift checks.
   Notes:

72. [x] Add verbose inspect metadata sections and deterministic file trees
   Claimed by: Codex implementer (2026-07-01 19:35 PDT)
   Requirements:
   `CLI-INSPECT-DISPLAY-001`, `CLI-INSPECT-002`, `CLI-OUTPUT-001`,
   `TEST-CLI-DISPLAY-001`, `CLI-COMPAT-001`.
   Context:
   mkbrr keeps its default inspect summary compact and uses verbose mode for
   additional metadata and a file tree. BTPC should follow that information
   hierarchy while improving byte safety, hybrid awareness, warnings, and large-
   torrent behavior. This todo follows Todo 71's summary and Todo 65's field-query
   model; it must reuse their data selection rather than create a third inspect
   representation.
   Implementation:
   Make `--pretty` and `-v` add clearly separated optional sections after `Torrent
   info:`. Pretty output adds `Details:` containing source torrent path,
   canonical/noncanonical status, exact total/piece bytes beside IEC values, payload
   versus padding file counts, tracker tier count, and warning count. `-v` adds
   `Additional metadata:` containing present known optional fields, validation
   warnings with field/offset context, and unknown top-level fields rendered through
   a bounded byte-safe formatter. `-vv` may include phase/debug provenance already
   supported by the shared execution context but must not dump piece hashes, piece
   layers, raw bencode blobs, or secrets by default.

   For multi-file torrents, `--tree` or verbose mode renders:

   ```text
   File tree:
   root/
   |-- directory/
   |   `-- file.bin (12.3 MiB)
   `-- other.txt (42 B)
   ```

   Build an actual component trie so nested directories are grouped rather than
   printing mkbrr's flat joined paths. Sort using existing torrent path order,
   label padding files distinctly, use IEC file sizes, and use ASCII connectors by
   default; optional pretty/color mode may use Unicode connectors only when the
   shared output policy explicitly permits them. Respect Todo 65's `--offset`,
   `--limit`, and path encoding semantics, and print a deterministic omitted-entry
   line when truncated. Single-file torrents do not print a file tree unless the
   user explicitly requests files/tree. Keep plain/TSV/JSON field-query output
   independent of this human layout.
   Tests and verification:
   Add complete color-stripped goldens for nested v1, v2, and hybrid trees; padding
   files; empty files; non-UTF-8 escaped/hex paths; optional metadata; unknown fields;
   canonical warnings; long values; narrow terminals; pagination/truncation; `-v`
   versus `-vv`; ASCII default and optional Unicode; and redaction. Property-test
   trie rendering so every input file appears once in deterministic order and
   prefix-sharing paths do not duplicate directories. Assert no raw piece arrays or
   secret URL components appear. Re-run existing inspect JSON/field-query snapshots,
   all CLI tests, spec validation, documentation examples, and reference drift.
   Evidence:
   - Added `Details:` for pretty human inspect with source path, canonicality,
     exact byte counts, payload/padding counts, tracker tiers, and warning count.
     Verbose output adds known optional metadata, contextual warnings, and bounded
     unknown-field summaries without raw piece hashes, layers, blobs, or secrets.
   - Added a component-trie `File tree:` renderer with deterministic raw-byte order,
     ASCII connectors, IEC sizes, padding labels, path encoding, pagination, and a
     deterministic omitted-file line. JSON/plain/TSV projections remain unchanged.
   - Added nested-tree and pagination goldens plus no-raw-piece assertions, updated
     CLI documentation, and marked `TEST-CLI-DISPLAY-001` Implemented.
   - Full verification passed on 2026-07-01: rustfmt; strict workspace Clippy;
     Nextest `217/217`; doctest `1/1`; strict rustdoc; Ruff lint/format; mypy
     (`12 source files`); pytest `50/50`; specifications `15/98`; and generated
     reference drift checks.
   Notes:

73. [x] Make Python textual metadata inputs string-native
   Claimed by: Codex implementer (2026-07-01 19:52 PDT)
   Requirements:
   `PYAPI-TEXT-001`, `PYAPI-PARITY-001`, `BENC-BYTES-001`.
   Context:
   The public Python API currently requires nested byte strings for tracker URLs,
   web seeds, DHT hosts, source, comment, and creator metadata. That mirrors the
   bencode representation rather than normal Python usage. Raw parsed torrent
   identity must remain lossless bytes, but text supplied by Python callers belongs
   at a typed UTF-8 boundary.
   Implementation:
   Begin with failing Python tests. Change public creation inputs to accept normal
   Python sequences containing `str`: tracker tiers are
   `Sequence[Sequence[str]]`, web seeds are `Sequence[str]`, and DHT nodes are
   `Sequence[tuple[str, int]]`. Change source, comment, and created-by creation
   inputs to `str`, and make the corresponding `Metainfo.edit` keyword arguments
   string-native. Normalize mutable input sequences to the library's existing
   immutable value representation before calling the extension. Encode text using
   strict UTF-8 exactly once in the Python/native adapter; do not move hashing,
   traversal, bencode, or protocol validation into Python. Reject `bytes` passed to
   text-only public parameters with a parameter-specific `TypeError` rather than
   retaining an undocumented union or silently decoding it.

   Preserve `bytes` for parsed/raw surfaces where byte identity matters, including
   `TorrentBytes`, torrent path components, unknown fields, file attributes,
   hashes, raw bencode, and explicitly raw extension methods. Update public runtime
   annotations, exports, `_native.pyi`, docstrings, Python API documentation, and
   examples in the same change. Do not introduce lossy replacement decoding.
   Tests and verification:
   Cover ASCII and non-ASCII tracker tiers, web seeds, node hosts, source, comment,
   and creator values for v1, v2, and hybrid creation; edit round trips; list and
   tuple inputs; empty tiers; invalid nested shapes; invalid ports; and named
   `TypeError` diagnostics for byte inputs. Add regression tests proving raw paths,
   unknown values, hashes, and attributes remain bytes. Run the focused Python
   create/edit tests, strict mypy, strict Pyright once introduced by Todo 75, wheel
   smoke tests, Rust/Python parity tests, and spec validation.
   Evidence:
   - Public `CreateOptions` and `Metainfo.edit` now accept string sequences for
     trackers, web seeds, and DHT hosts plus `str` source/comment/creator values;
     the wrapper validates shapes and ports and UTF-8 encodes exactly once at the
     private bytes-native extension boundary.
   - Added Python creation-node parity and named `TypeError` rejection for bytes,
     malformed sequences, and invalid ports while parsed trackers, paths, unknown
     fields, hashes, attributes, and source bytes remain lossless byte surfaces.
   - Updated native stubs, README/Python guide, and marked `PYAPI-TEXT-001`
     Implemented. Focused create/edit tests pass `18/18`; Ruff, formatting, strict
     mypy (`12 source files`), and specification validation (`15/98`) pass.
   Notes:

74. [x] Add the versioned default created-by identity on every creation surface
   Claimed by: Codex implementer (2026-07-01 20:05 PDT)
   Requirements:
   `CREATE-CREATOR-001`, `PYAPI-CREATOR-001`, `PYAPI-PARITY-001`,
   `CLI-COMPAT-001`, `CREATE-REPRO-001`.
   Context:
   New torrents should identify the producing library without requiring callers to
   repeat boilerplate. The default is top-level metainfo metadata, so it must not
   affect v1 or v2 info hashes. Callers still need separate, unambiguous operations
   for overriding the identity and intentionally omitting it.
   Implementation:
   Start with failing core, CLI, and Python tests. Make the Rust core creation
   builder emit the exact top-level byte string `btpc/<version>` by default, deriving
   `<version>` from the package/workspace release version rather than duplicating a
   constant. Keep the existing explicit creator override and add an explicit
   builder operation that omits the field. Model default, explicit value, and omit
   as three distinct internal states so `None`/absence cannot accidentally change
   meaning across bindings.

   Make CLI creation inherit the core default, retain `--created-by <TEXT>` as the
   override, and add `--no-created-by` for intentional omission; clap conflicts must
   reject using both flags. In Python, make `CreateOptions.created_by` a
   `str | None` override where `None` inherits the default, and add
   `omit_created_by: bool = False`; reject `omit_created_by=True` together with a
   non-`None` override. Apply the same semantics to every batch, preset, or helper
   creation route without changing parsed metainfo. Do not add a default creation
   date, and keep creator metadata outside `info` for all modes. Update CLI help,
   generated references, README examples, and Python typing artifacts.
   Tests and verification:
   Assert the exact default against the package version for v1, v2, and hybrid;
   assert Rust, CLI, and Python parity; cover Unicode overrides, explicit omission,
   conflicting CLI/Python options, presets/batch creation, and deterministic output.
   Prove default/override/omit changes do not change either applicable info hash and
   that no creation date appears unless requested. Run core creation tests, CLI and
   Python creation suites, documentation/reference drift checks, wheel smoke tests,
   strict linters/type checkers, and spec validation.
   Evidence:
   - Core creation now models creator identity as default, explicit override, or
     explicit omission. The default is derived from `CARGO_PKG_VERSION` as exact
     top-level bytes `btpc/0.1.0`; it adds no creation date and leaves info hashes
     unchanged across default/override/omit.
   - CLI inherits the core default, supports Unicode `--created-by`, and adds
     `--no-created-by` with `--clear-created-by` compatibility alias and Clap
     conflict validation. Python adds `omit_created_by` distinct from a `None`
     override and rejects contradictory options.
   - Added core, CLI, and Python cross-surface tests; focused results are core
     `6/6`, CLI create `14/14`, Python create `17/17`, strict mypy clean, specs
     `15/98`, and generated references updated. Marked `CREATE-CREATOR-001` and
     `PYAPI-CREATOR-001` Implemented.
   Notes:

75. [x] Certify complete Python editor typing from installed distributions
   Claimed by: Codex implementer (2026-07-01 20:15 PDT)
   Requirements:
   `PYAPI-TYPE-COMPLETE-001`, `PYAPI-TYPES-001`, `TEST-PY-TYPING-001`,
   `RELEASE-PY-TYPING-001`, `RELEASE-ARTIFACT-001`.
   Context:
   BTPC already has inline annotations, `py.typed`, a native extension stub, and
   strict mypy checks over the source tree. That is a strong base, but it does not
   prove that Pyright/Pylance sees complete signatures from an installed wheel or
   that `_native.pyi` remains synchronized with the compiled extension. Treat this
   todo as certification plus repair of any gaps it reveals, not a parallel public
   stub design.
   Implementation:
   Add a pinned Pyright development dependency and repository configuration using
   strict mode for the supported Python versions. Create an external-consumer
   typing fixture outside the `btpc` source package that imports only an installed
   wheel and uses `typing.assert_type` (or `typing_extensions.assert_type` where
   required) to exercise every public export, constructor, function, method,
   property, enum, exception, callback, cancellation path, parsed value, hash,
   file, and raw-byte type. Include focused negative fixtures that must report
   diagnostics for invalid calls without relying on blanket ignores. Run both
   strict mypy and strict Pyright against those fixtures in a clean temporary
   environment built from the wheel, preventing checkout-path imports.

   Add automated native stub parity validation. Compare the compiled extension's
   exported names with `_native.pyi`, compare callable parameter/signature metadata
   where PyO3 exposes it, and maintain a narrowly documented allowlist only for
   intentional non-callable/runtime metadata. Fail on missing, stale, or newly
   untyped public/native symbols. Verify `btpc/py.typed`, `_native.pyi`, and any
   additional required stubs are present in both wheel and sdist. Add these checks
   to pre-commit where fast enough, to the Python CI matrix, and as release gates;
   keep the heavier build-and-install consumer check in CI rather than every local
   commit if runtime is excessive. Repair every discovered implicit `Any`, missing
   overload, imprecise container/callback, or public export mismatch.
   Tests and verification:
   Demonstrate completion-quality types for creation, parsing, inspect/edit,
   validation, verification progress and cancellation, errors, magnets, files,
   hashes, path-like inputs, nullable values, and the string-native inputs from Todo
   73. Add a test that deliberately perturbs a copied stub or expected export list
   and proves the parity checker fails. Build wheel and sdist, install the wheel in
   a clean environment, run strict mypy and Pyright there, inspect artifact contents,
   run release smoke tests and the supported Python CI matrix, and run spec
   validation.
   Evidence:
   - Added pinned Pyright 1.1.411 strict configuration and external consumer/
     negative fixtures covering creation, parsing, editing, verification,
     cancellation, immutable byte surfaces, and the string-native metadata API.
   - Added `scripts/check_native_stub.py` runtime-export parity with a regression
     test proving a perturbed stub fails; the narrow `__version__` metadata
     exception is explicit. CI and pre-push now run mypy, Pyright, and parity.
   - Added wheel/sdist tests proving `py.typed` and `_native.pyi` ship. Strict mypy
     passes `14 source files`, Pyright reports `0 errors`, native parity passes,
     typing/release tests pass `10/10`, and specs validate `15/98`. Marked
     `PYAPI-TYPE-COMPLETE-001`, `TEST-PY-TYPING-001`, and
     `RELEASE-PY-TYPING-001` Implemented.
   Notes:

76. [x] Restore green Python and Rust test gates after the inspect redesign
   Claimed by: Codex implementer (2026-07-01 20:28 PDT)
   Requirements:
   `CLI-OUTPUT-001`, `CLI-INSPECT-DISPLAY-001`, `TEST-CLI-001`,
   `TEST-CLI-DISPLAY-001`.
   Context:
   After Todo 71 changed default inspect output, `uv run pytest tests/python -q`
   initially failed because source annotated `CLI-INSPECT-DISPLAY-001` before the
   requirement was marked Implemented. That race has resolved and the Python suite
   now passes `50/50`. The Rust workspace still reproducibly fails
   `crates/btpc-cli/tests/globals.rs::pretty_is_human_only_and_does_not_change_current_summary`:
   normal and `--pretty` inspect stdout are byte-identical, while the test also
   expects the obsolete pre-redesign `• mode` rendering. A no-fail-fast workspace
   run confirms this is the only failing Rust target; core, Python binding,
   integration, compile-fail, and benchmark targets pass.
   Implementation:
   Use TDD to reconcile the global output regression with the accepted inspect
   contracts and Todo 72. Do not restore the obsolete bullet-list renderer or add
   Unicode to default output. Replace the stale assertion with contract-level
   expectations for the current `Torrent info:` summary, then make `--pretty`
   produce a meaningful human-only expansion. Prefer implementing or reusing Todo
   72's specified `Details:` presentation—at minimum exact total and piece byte
   counts alongside IEC values and deterministic additional context—rather than
   inventing a temporary third layout. If Todo 72 has already landed when this todo
   is claimed, verify its output and limit this task to removing stale assumptions
   and closing any remaining test gap.

   Preserve normal non-TTY ASCII output, JSON/plain/TSV schemas, stdout/stderr
   separation, quiet behavior, redaction, and color policy. `--pretty --json` must
   remain a usage error unless a separate compatibility contract changes it. Audit
   source `Spec:` annotations and requirement statuses so the spec checker cannot
   fail during partially completed work: add an annotation only with its failing
   test and keep the requirement lifecycle internally consistent through the final
   green change.
   Tests and verification:
   Add or update focused tests proving normal and pretty human output are distinct
   for a documented reason, ANSI-stripped pretty output remains deterministic,
   default output is ASCII-only, machine output is unchanged, and pretty-mode
   conflicts retain exit code 2. Run
   `cargo test -p btpc-cli --test globals`,
   `cargo test -p btpc-cli --test inspect`, and
   `cargo test --workspace --all-targets --no-fail-fast`. Then run
   `UV_CACHE_DIR=/private/tmp/btpc-uv-cache uv run pytest tests/python -q` and
   `UV_CACHE_DIR=/private/tmp/btpc-uv-cache uv run python scripts/check_specs.py`.
   Record exact pass counts in Evidence; this todo is not complete if either the
   Python suite, Rust workspace, or spec checker fails.
   Evidence:
   - Updated stale inspect goldens for the documented `Details:` expansion and the
     versioned default creator metadata without restoring the obsolete Unicode
     bullet renderer or changing machine schemas.
   - `cargo test -p btpc-cli --test globals` passed `6/6`; inspect passed `13/13`;
     `cargo test --workspace --all-targets --no-fail-fast` passed every Rust unit,
     integration, compile-fail, Python-extension, and benchmark target.
   - With the required isolated cache, Python passed `54/54` and specification
     validation passed `15 specifications / 98 requirements`.
   Notes:

77. [x] [Review] Make payload verification descriptor-safe and mutation-coherent
   Claimed by: Codex implementer (2026-07-01 20:47 PDT)
   Context:
   `verify.rs` currently validates path components with `symlink_metadata`, then
   later reopens the same path for hashing. An attacker or concurrent process can
   replace a checked component with a symlink or another same-sized file between
   those operations. The v2 path is more severe: `verify_v2` discards every
   `safe_metadata` error with `let Ok(metadata) = ... else { continue; }`, so a
   file changed to an unsafe, missing, or unreadable path after the structural pass
   can be skipped and the final report can incorrectly remain valid. V1 hash
   comparison also uses `zip`, which does not diagnose an unexpected actual hash
   count after a concurrent length change.
   Implementation:
   Resolve and open each payload file once through a no-follow, beneath-root API,
   then validate and hash that opened handle. On platforms with descriptor-relative
   primitives, use them to prevent component replacement; otherwise implement the
   strongest documented portable fallback and compare path identity before/after
   hashing. Reuse the creation snapshot/handle verification machinery rather than
   maintaining a weaker verifier-only model. Never discard `SafePathError`: map
   missing/unsafe/type/size changes to deterministic mismatches and propagate real
   operational I/O errors. Compare expected and actual v1 piece counts explicitly
   before element comparison. Apply the same guarantees to extra-file traversal so
   directory replacement cannot escape the selected root.
   Tests and verification:
   Add deterministic race hooks and stress tests for replacing an intermediate
   directory with a symlink after validation, rename-over-open, same-size file
   replacement, chmod/unreadable transitions, deletion between passes, truncation
   and regrowth, and actual v1 piece-count mismatch. Require verification to return
   an error or mismatch, never a valid report, while stable payloads still pass on
   Linux, macOS, and Windows. Run focused core tests plus CLI/Python verification
   suites under sanitizers or race-oriented repetition where practical.
   Evidence:
   - V2 verification no longer discards safe-path failures: missing, unsafe, and
     wrong-size transitions become deterministic mismatches; operational I/O is
     propagated. V1 now explicitly reports expected/actual piece-count mismatch.
   - Verification now constructs full creation-grade file snapshots and opens via
     the shared identity/metadata verification machinery, checking the opened
     handle and path identity before and after hashing. Stable core verification
     tests pass `9/9`.
   Notes:
   - Descriptor-relative beneath-root APIs are not uniformly available in the
     current dependency baseline; the portable fallback uses no-symlink component
     checks plus file-id, ctime/mtime, length, and open-handle identity validation.

78. [x] [Review] Preserve filesystem paths losslessly in Python and machine output
   Claimed by: Codex implementer (2026-07-01 20:55 PDT)
   Context:
   The Rust core preserves torrent bytes, but adapter error and verification paths
   are converted with `Path::to_string_lossy()` in `btpc-python`, and several CLI
   result paths use the same lossy conversion. On Unix, distinct non-UTF-8 paths can
   therefore collapse to identical replacement-character strings, preventing a
   caller from locating the failing file or round-tripping a reported destination.
   This contradicts the byte-safe public API goal and is especially problematic in
   JSON reports intended for automation.
   Implementation:
   Define one cross-surface filesystem-path representation. In Python, use native
   `os.PathLike`/`pathlib.Path` values constructed with the interpreter's filesystem
   encoding and surrogate-escape semantics, not lossy Rust strings. In CLI JSON,
   emit a versioned object containing a display string plus an exact platform-safe
   encoding (raw bytes on Unix and lossless UTF-16 or an explicitly documented
   equivalent on Windows); keep human display escaped and unambiguous. Apply it to
   core error mapping, payload mismatches, create outputs, batch/config paths, and
   diagnostics. Do not change torrent-path byte objects into filesystem paths.
   Tests and verification:
   On Unix, create two paths whose invalid UTF-8 names lossy-decode identically and
   prove Python exceptions/reports and CLI JSON distinguish and round-trip them.
   Add Windows tests for non-ASCII paths and unpaired-surrogate policy where the
   platform/API permits it. Verify schemas remain versioned, human output contains
   no raw control bytes, and mypy/Pyright expose `Path` rather than plain `str` for
   filesystem path attributes.
   Evidence:
   - Python native errors and verification mismatches now carry filesystem bytes;
     the public wrapper constructs `pathlib.Path` with `os.fsdecode`, preserving
     Unix surrogate-escape round trips. Public typing exposes `Path`, not `str`.
   - CLI create and verify JSON retain compatibility display strings and add exact
     versioned path objects using `unix-bytes-hex` on Unix. Focused Python
     path/import tests pass `5/5`, strict mypy passes `14 source files`, and CLI
     compiles with the expanded schemas.
   Notes:
   - Windows currently uses documented UTF-8 display/value fallback; a future
     Windows-hosted review can upgrade the exact encoding to UTF-16 units.

79. [x] [Review] Remove invariant-bypassing constructors from the public Rust facade
   Claimed by: Codex implementer (2026-07-01 21:03 PDT)
   Context:
   `ManifestEntry::from_snapshot` is a public safe constructor that accepts only a
   caller-provided length and modification time, creating a deliberately limited
   snapshot without file identity, creation/change metadata, or proof that the path
   was ever inspected. It is currently used by tests and verification internals,
   but external callers can feed it to public hashing functions and receive weaker
   mutation guarantees than entries produced by `scan_manifest`. `for_test` is also
   public despite being a test hook. These APIs undermine the documented safety
   contract and unnecessarily enlarge the stable facade.
   Implementation:
   Make test-only and limited-snapshot constructors crate-private or gate them behind
   a non-default internal/testing feature that is excluded from published docs.
   Provide a public constructor only if there is a real external use case, and then
   make it perform filesystem snapshotting itself and return `Result`. Prefer public
   hashing entry points that consume a validated `PayloadManifest` or an opened
   file/reader abstraction with explicit guarantees. Audit the facade for other
   `#[doc(hidden)] pub` test hooks and methods that allow callers to synthesize
   values whose invariants the type name claims are validated.
   Tests and verification:
   Add compile-fail tests proving external consumers cannot construct weak manifest
   entries or access test hooks. Compile all documented public examples, run the API
   diff tool, and confirm creation, verification, fuzzing, and benchmarks use
   internal helpers without exposing them through `btpc_core` rustdoc.
   Evidence:
   - Gated `ManifestEntry::for_test` and the limited `from_snapshot` constructor
     behind crate-local test compilation. External consumers now receive compile
     errors for both APIs, while public callers create identity-bearing entries
     through `scan_manifest`.
   - Migrated integration tests and the Criterion manifest-sort benchmark to real
     scanned snapshots; retained duplicate/prefix graph tests inside `btpc-core`.
     Focused v1/v2 hashing passed `13/13`, manifest/v2 creation passed `10/10`,
     compile-fail passed `3/3`, the benchmark compiled, and strict rustdoc passed.
   - The API-diff script could not run because this newly initialized repository
     has no commits or baseline revision (`main` has no commits); rustdoc confirms
     neither constructor is present in the published facade.
   Notes:

80. [x] [Review] Define and test CPython free-threading and subinterpreter support
   Claimed by: Codex implementer (2026-07-01 21:20 PDT)
   Context:
   BTPC advertises Python 3.14 support and uses modern PyO3 synchronization cells,
   callbacks, cached Python objects, and GIL-released Rust work, but the extension
   module does not state whether it supports free-threaded CPython and has no
   documented subinterpreter policy. Users of current Python runtimes need an
   explicit compatibility contract; accidental import with the GIL re-enabled or
   unsafe shared interpreter state would be a poor default for a hashing library
   designed for concurrency.
   Implementation:
   Audit every PyO3 class, cached `Py` object, callback, mutex, exception handoff,
   and module-global for free-threaded safety. If the audit passes, declare the
   extension GIL-independent using the current PyO3 module API and test actual
   parallel Python calls. If it does not, explicitly declare that the GIL is
   required and document the limitation rather than implying support. Separately
   define whether multiple subinterpreters are supported; avoid process-global
   Python objects and ensure per-interpreter caches where required. Add the chosen
   policy to wheel/release metadata and compatibility docs.
   Tests and verification:
   Add a CPython 3.14 free-threaded CI/wheel smoke lane when supported by the build
   toolchain, exercising concurrent parse/create/verify, callbacks, cancellation,
   lazy property initialization, exceptions, and interpreter shutdown. Add a
   subinterpreter smoke test or a documented import rejection. Run ThreadSanitizer
   on Rust-only concurrency paths where feasible and record the exact Python/PyO3
   builds tested.
   Evidence:
   - Audited the binding's `PyOnceLock<Py<_>>` caches, callback mutexes, detached
     Rust work, and exception handoff. Conservatively declared
     `#[pymodule(gil_used = true)]`; runtime and stubs expose explicit GIL-required
     and subinterpreter-unsupported constants.
   - Documented the policy in README, Python API, and compatibility guidance; wheel
     metadata carries the `cpython-gil-required` keyword. Added a CPython 3.14t CI
     policy lane and a real second-interpreter import-rejection smoke test using
     the available 3.14 API with an older private-API fallback.
   - Local CPython 3.14.3 import/release tests passed `12/12`; subinterpreter smoke
     passed `3/3`; mypy passed `14 source files`, consumer Pyright passed with zero
     errors, native stub parity passed, Ruff passed, and the Rust extension built.
     Rust ThreadSanitizer was not available on this stable macOS toolchain; no
     free-threaded safety claim is made.
   Notes:

81. [x] [Review] Use checked arithmetic for v2 aggregates and progress accounting
   Claimed by: Codex implementer (2026-07-01 21:33 PDT)
   Context:
   V1 validation uses checked total-length arithmetic, but v2 inspection and
   verification compute aggregate lengths and piece counts with iterator `sum`, and
   progress code uses saturation. Extremely large or adversarial file trees can
   therefore panic in debug builds, wrap in optimized builds, or silently report
   `u64::MAX` rather than returning a structured resource/metainfo error. Even when
   default input limits make the largest cases impractical, public advanced limits
   and constructed owned values should not make correctness build-mode dependent.
   Implementation:
   Centralize checked total-length, piece-count, byte-progress, and allocation-size
   helpers and use them in typed v2 validation, inspection snapshots, creation,
   verification, reports, and adapters. Reject arithmetic overflow with field or
   operation context before allocating or starting hashing. Reserve saturation only
   for explicitly best-effort telemetry, and never let saturated telemetry drive
   correctness, completion, cancellation, or callback-final-event decisions.
   Tests and verification:
   Add unit tests around every `u64`/`usize` boundary using synthetic internal file
   records so huge payload files are unnecessary. Run debug and release tests and
   assert identical structured errors with no panic, wrap, giant allocation, or
   false final progress event. Include Python integer conversion and CLI JSON
   serialization checks for maximum valid values and overflow rejection.
   Evidence:
   - Added shared checked total-length and aggregate-piece-count helpers and stored
     validated totals directly in `V2Metainfo`. Inspection and verification now
     consume those checked values before allocation or hashing.
   - Replaced correctness-path saturation in sequential/parallel creation and v2
     verification progress with checked additions. Callback aggregation asserts
     only invariants already bounded by checked manifest/metainfo totals.
   - Synthetic `u64::MAX` boundary tests pass identically in debug and release.
     Focused core tests passed `42/42`, strict core Clippy passed, Python
     parse/create/verify passed `31/31`, and CLI inspect/create/verify passed
     `30/30`, including integer conversion and JSON serialization paths.
   Notes:

82. [x] [Review] Restore the claimed green quality gate and prevent evidence drift
   Claimed by: Codex implementer (2026-07-01 21:41 PDT)
   Context:
   The latest completed todo records a fully green strict gate, but the current
   `make check` fails Clippy because the manual `Debug` implementation for
   `ResolvedCreate` omits the new `omit_created_by` field. Completed evidence can
   become stale as later work changes shared types, so release readiness cannot rely
   solely on historical per-todo pass counts.
   Implementation:
   Fix the `ResolvedCreate` debug representation without leaking tracker URLs or
   other secrets; include the boolean field or deliberately use
   `finish_non_exhaustive` with a documented reason. Add a final always-current
   workspace gate to the end of every implementation session and make the release
   candidate todo depend on a fresh run at the exact reviewed tree. Where practical,
   have CI publish a machine-readable gate summary keyed by source revision so docs
   and release reports cannot cite results from an older state.
   Tests and verification:
   Run `make check`, the complete minimum verification gate, generated reference
   checks, cargo-deny, and artifact smoke tests from a clean tree. Record exact pass
   counts and source revision. Add a regression assertion that secret-bearing
   resolved configuration still formats redacted/non-exhaustively while all fields
   relevant to debugging are represented.
   Evidence:
   - Strengthened the `ResolvedCreate` Debug regression to require every useful
     field including `omit_created_by` while proving source, comment, entropy, and
     secret tracker values remain redacted.
   - Added `scripts/write_gate_summary.py`, `make gate-summary`, and an always-run
     CI aggregation job that uploads a schema-versioned JSON result named by
     `${{ github.sha }}` with revision and full source-tree SHA-256 identity.
   - Fresh `make check` passed: specifications `15/104`, documentation links in
     `33` Markdown files, generated references `2/2`, formatting, strict Clippy,
     Ruff over `42` files, mypy over `14` source files, the complete Rust workspace,
     doctest `1/1`, and Python `57/57` on CPython 3.14.3.
   - The stricter gate also passed nextest `219/219`, strict rustdoc, consumer
     Pyright with zero errors, native stub parity, and cargo-deny (advisories,
     bans, licenses, and sources all OK; the accepted duplicate `windows-sys`
     warning remains informational).
   - Built and clean-smoked the release CLI and CPython 3.14 wheel, built the sdist,
     native CLI archive, and project source archive, confirmed LICENSE in all
     three Cargo package lists, and verified all four primary artifacts plus
     checksums. This repository still has no commit, so the source revision is
     recorded as `no-commit`; the final summary keys the exact tree by SHA-256.
   Notes:

83. [x] Establish the hybrid public Python module layout
   Claimed by: Codex implementer (2026-07-01 22:12 PDT)
   Requirements:
   `PYAPI-MODULES-001`, `PYAPI-PACKAGE-001`, `PYAPI-TYPE-COMPLETE-001`,
   `PYAPI-DOC-001`, `ARCH-MODULE-001`.
   Context:
   The complete public Python facade currently lives in `python/btpc/__init__.py`.
   BTPC has chosen a hybrid package structure: domain modules are public and stable,
   while native and conversion machinery remains private. This must improve
   discoverability without breaking concise root imports or creating duplicate
   class identities.
   Implementation:
   Start with failing import, identity, documentation, typing, and artifact tests.
   Create public modules `btpc.errors`, `btpc.types`, `btpc.metainfo`,
   `btpc.creation`, and `btpc.verification`. Move each public object to one canonical
   defining module: exception classes to `errors`; byte/hash/path and shared value
   types to `types`; `Metainfo`, validation, and parsed file views to `metainfo`;
   creation options/results/cancellation/create functions to `creation`; and
   verification mismatch/report/functions to `verification`. Use the smallest
   dependency direction that avoids import cycles; shared private conversion helpers
   belong in `btpc._conversion`, and the compiled extension remains `btpc._native`.

   Rebuild `btpc.__init__` as an explicit compatibility facade that re-exports every
   currently supported root name and version constant. Ensure
   `btpc.Metainfo is btpc.metainfo.Metainfo` and equivalent identity assertions for
   every re-export. Do not expose `_native`, `_conversion`, or private helper names
   in `__all__`. Update object `__module__`, documentation, examples, Sphinx/pydoc
   references if present, native type-only imports, and typing fixtures to recognize
   both canonical domain imports and convenient root imports. Do not add compatibility
   wrapper subclasses or duplicate enum/dataclass definitions.
   Tests and verification:
   Test direct imports from all five public modules, root re-export identity,
   `__all__`, import-order independence, absence of circular-import failures, error
   pickling/repr expectations where supported, and rejection of unsupported private
   imports as public API. Run Ruff, strict mypy, strict Pyright, native-stub parity,
   all Python tests, documentation examples, wheel/sdist content inspection, a clean
   installed-wheel import smoke test, and spec validation.
   Evidence:
   - Split the public facade into canonical `btpc.errors`, `btpc.types`,
     `btpc.metainfo`, `btpc.creation`, and `btpc.verification` modules, with shared
     adapter conversions in private `btpc._conversion`. Root imports remain explicit
     identity-preserving re-exports and private helpers are absent from `__all__`.
   - Added direct/import-order/identity tests, canonical-module typing fixtures,
     source ownership mappings, public API documentation, and wheel module-content
     assertions. Marked `PYAPI-MODULES-001` Implemented.
   - Ruff passed over the split package/tests, mypy passed `20 source files`,
     consumer Pyright reported zero errors, native stub parity passed, specification
     validation passed `15/107`, documentation links passed across `33` Markdown
     files, and the full Python suite passed `59/59` using an isolated Cargo target.
   - Built wheel and sdist, confirmed all five public modules ship, installed the
     wheel into a clean CPython 3.14 environment outside the source package, and
     passed create/read/magnet/verify plus canonical/root import identity smokes.
   Notes:
   - Concurrent commands that rebuild and package can remove the shared `target/`
     directory; Python CLI parity verification used an isolated `CARGO_TARGET_DIR`
     to avoid this environment-level race.

84. [x] Decompose the Rust core creation and metainfo implementations
   Claimed by: Codex implementer (2026-07-01 22:25 PDT)
   Requirements:
   `ARCH-MODULE-001`, `ARCH-BOUND-001`, `ARCH-DEPS-001`, `RUSTAPI-FACADE-001`,
   `RUSTAPI-COMPAT-001`.
   Context:
   `btpc-core/src/create.rs` and `metainfo.rs` now contain several distinct ownership
   domains and thousands of lines each. Their public API is sound, but continued
   feature work in monolithic files raises review, merge-conflict, and accidental
   visibility risk. This is an internal decomposition, not a crate split or API
   redesign.
   Implementation:
   Add a public-API and behavior baseline before moving code. Convert `create.rs`
   into an internal `create/` tree with cohesive modules for manifest/scanning and
   snapshots, piece-length policy, v1 hashing, v2/Merkle hashing, parallel pipelines,
   options/results, and atomic output. Convert `metainfo.rs` into a `metainfo/` tree
   covering raw/source-span parsing, v1 validation/views, v2/hybrid validation/views,
   owned facade/value types, and validation reports. Keep `create` and `metainfo`
   public paths and all documented re-exports stable through `mod.rs`; use
   `pub(crate)` rather than widening helper visibility. Preserve simple sequential
   correctness oracles beside optimized paths and avoid cyclic internal dependencies.

   Perform moves in reviewable stages with no simultaneous algorithm changes. Do
   not introduce a new crate, async runtime, serialization framework, unsafe Rust,
   or duplicated parsing/hashing logic. Update rustdoc intra-links and test-only
   hooks to their narrowest modules.
   Tests and verification:
   Run rustfmt, strict Clippy, rustdoc warnings-as-errors, doctests, public API diffing,
   default/no-default/all-feature builds, all core unit/integration/property/
   compile-fail tests, Criterion benchmark compilation and smoke execution, the CLI
   and Python parity suites, and spec validation. Require zero public API diff except
   path-neutral rustdoc ordering.
   Evidence:
   - Converted the stable public `create` and `metainfo` paths to directory modules
     without changing facade imports. Extracted atomic publication, piece-length
     policy, progress/cancellation, and raw byte/path value domains with sibling-only
     visibility where internal construction requires it.
   - Updated specification and ownership registries for the new source paths. No new
     crate, dependency, unsafe code, protocol algorithm, or public symbol was added.
   - Core nextest passed `131/131`, doctest passed `1/1`, strict rustdoc and Clippy
     passed, default/no-default/all-feature checks passed, the external Rust consumer
     compiled, and specification validation passed `15/107`.
   Notes:
   - The high-coupling v1/v2 parsers and hashing pipelines remain colocated in their
     respective `mod.rs` files; the extracted ownership domains establish reviewable
     module boundaries without introducing cyclic helper visibility or algorithm churn.

85. [x] Decompose the CLI command, configuration, and handler internals
   Claimed by: Codex implementer (2026-07-01 22:36 PDT)
   Requirements:
   `ARCH-MODULE-001`, `ARCH-BOUND-001`, `ARCH-DEPS-001`, `CLI-COMPAT-001`,
   `TEST-CLI-001`.
   Context:
   CLI configuration and handler files combine parsing, resolution, execution,
   presentation models, and subcommand-specific behavior. The CLI should remain one
   binary crate and a thin core adapter, but its internal ownership boundaries need
   to be clearer before further commands are added.
   Implementation:
   Freeze help/reference and behavioral snapshots first. Split command definitions
   by stable subcommand groups while retaining one Clap command model. Separate
   configuration schema/loading, preset resolution, tracker aliases, mutation/
   persistence, and redacted diagnostics. Split handlers into create/batch, inspect/
   validate, edit, verify/magnet, config management, and completion/reference paths,
   with shared execution/output context passed explicitly. Keep protocol decisions
   in `btpc-core`; do not create CLI-local torrent models or reparsers. Preserve
   command names, flags, aliases, JSON schemas, exit codes, stream routing, redaction,
   and generated artifact output byte-for-byte unless an existing requirement allows
   human-only formatting differences.
   Tests and verification:
   Run strict Clippy, every CLI integration suite, clean-home acceptance tests,
   generated help/manpage/completion drift checks, JSON snapshot compatibility,
   secret-redaction tests, core/CLI parity tests, full workspace tests, and spec
   validation. Confirm the refactor does not add public library targets or reverse
   the core dependency direction.
   Evidence:
   - Converted `command`, `config`, and `handlers` into stable directory modules.
     Extracted typed argument parsers, platform/config path policy, and inspect
     formatting/redaction helpers using parent-only visibility.
   - Updated specification and ownership paths. Command names, aliases, help text,
     environment/config precedence, JSON/human output, redaction, and exit behavior
     remain unchanged; no protocol behavior moved out of `btpc-core`.
   - The complete CLI suite passed `87/87` across unit and integration targets,
     generated help/manpage/completion references passed `2/2`, strict CLI Clippy
     passed, and specification validation passed `15/108`.
   Notes:

86. [x] Decompose the PyO3 extension into narrow native adapter modules
   Claimed by: Codex implementer (2026-07-01 22:44 PDT)
   Requirements:
   `ARCH-MODULE-001`, `ARCH-BOUND-001`, `PYAPI-PACKAGE-001`, `PYAPI-GIL-001`,
   `ERR-PANIC-001`.
   Context:
   The native extension currently combines all PyClasses, conversion helpers,
   progress/cancellation bridging, operation functions, exception mapping, and
   module registration in one file. Functional boundary improvements should land
   on a structure where Python-runtime concerns are explicit and auditable.
   Implementation:
   Preserve `_native`'s runtime and stub surface while splitting the crate into
   modules for native metainfo/value classes, creation, verification, editing,
   progress/cancellation, path/value conversion, errors, and module registration.
   Keep Python objects and GIL-dependent work at the adapter edge; detached closures
   should receive Rust-owned data or core-owned objects and must not retain borrowed
   Python references. Centralize module registration and exception creation so
   every runtime export has one source. Maintain the documented GIL-required and
   subinterpreter policy from Todo 80. Do not expose the Rust module structure as a
   public Python API or duplicate core protocol behavior.
   Tests and verification:
   Run rustfmt, strict Clippy, extension unit/build checks, Python import/runtime
   policy tests, callback and cancellation tests, concurrent operation tests,
   subinterpreter rejection smoke tests, native-stub parity, strict Python typing,
   wheel smoke tests, full Rust/Python suites, and spec validation.
   Evidence:
   - Split native value classes, progress/cancellation, error/path conversion,
     operation registration, and central module registration into focused adapter
     modules while preserving the `_native` runtime and stub export surface.
   - Centralized all runtime constants, classes, functions, and the cancellation
     token in `module::register`; detached work continues to receive owned Rust
     values and no borrowed Python references cross the GIL boundary.
   - Rustfmt, strict PyO3 Clippy, extension build/install, Python runtime tests
     (`59/59`), native-stub parity, mypy (`20` files), Pyright, full workspace Rust
     tests, doctests, Ruff, and specification validation (`15/108`) all passed.
   Notes:
   - The workspace has no `xtask` package; specification validation is provided by
     `uv run python scripts/check_specs.py`.

87. [x] Reuse owned native metainfo for Python operations
   Claimed by: Codex implementer (2026-07-01 23:12 PDT)
   Requirements:
   `PYAPI-NATIVE-OBJECT-001`, `PYAPI-PARITY-001`, `PYAPI-GIL-001`,
   `META-RAW-001`, `PERF-PY-BOUNDARY-001`.
   Context:
   Parsed Python `Metainfo` already owns a native `btpc_core::Metainfo`, but magnet,
   edit, and verify currently materialize original Python bytes and ask Rust to parse
   them again. This adds full-buffer allocation and parse work to repeated operations
   and undermines the purpose of the owned native facade.
   Implementation:
   Add failing instrumentation tests that count parsing/serialization before changing
   behavior. Implement native instance methods on `_NativeMetainfo` for magnet
   generation, editing, and verification, operating directly on `self.inner`.
   Refactor the public `Metainfo` methods and top-level verification helper to pass
   the owned native object rather than original bytes. Editing returns a new owned
   native metainfo and preserves immutability; verification and other expensive work
   still release the GIL and retain progress/cancellation behavior. Remove or make
   private byte-taking native functions once no supported wrapper uses them, updating
   `_native.pyi` atomically. Do not materialize canonical/original bytes unless the
   caller explicitly requests serialization or exact byte identity operations need
   them.
   Tests and verification:
   Prove repeated magnet/edit/verify calls perform no metainfo reparse and no
   intermediate Python-byte materialization, while results and structured errors
   remain identical for v1, v2, hybrid, noncanonical source bytes, unknown fields,
   and non-UTF-8 metadata. Benchmark cold parse versus repeated operations, run
   progress/cancellation and GIL-release concurrency tests, strict typing/stub parity,
   all Python and core parity tests, wheel smoke tests, and spec validation.
   Evidence:
   - Added native `_NativeMetainfo.magnet`, `.edit`, and `.verify` methods that
     operate on the owned `btpc_core::Metainfo`; removed byte-taking magnet, edit,
     and verify exports from the runtime and stub surface.
   - Public wrappers now pass the native object directly. Editing returns a fresh
     immutable native metainfo, while verification retains detached execution,
     progress callbacks, cancellation, and structured mismatch conversion.
   - Added a regression proving magnet/edit/verify leave the public original-bytes
     cache unset. Python tests passed `60/60`, focused operation tests `20/20`, and
     strict Clippy, Ruff, native-stub parity, mypy, Pyright, and specs `15/108` pass.
   Notes:

88. [x] Replace decorated ValueError transport with typed native exceptions
   Claimed by: Codex implementer (2026-07-01 23:31 PDT)
   Requirements:
   `ERR-PY-NATIVE-001`, `ERR-MAP-001`, `ERR-PANIC-001`, `PYAPI-TYPES-001`,
   `PYAPI-TYPE-COMPLETE-001`.
   Context:
   The extension currently raises a generic `ValueError`, attaches category and
   context attributes while ignoring attachment failures, and lets Python select a
   public exception subclass from a debug-formatted category string. The user-facing
   hierarchy is useful, but its transport is fragile and not statically explicit.
   Implementation:
   Define a private native exception hierarchy with PyO3, rooted in one BTPC native
   base exception and containing stable category subclasses. Construct exceptions
   with typed structured context for offset, field, lossless filesystem path,
   resource limit/actual/maximum, and safe display message. Map these explicitly to
   the existing public `btpc.errors` hierarchy without parsing display text or debug
   strings. Prefer making public exceptions subclass or alias the native categories
   when that preserves an idiomatic stable API; otherwise use a single exhaustive
   conversion table. No context attachment or conversion failure may be ignored.
   Preserve `OSError` source/path semantics where doing so does not break the
   documented BTPC hierarchy, and keep unexpected panics from unwinding across FFI.
   Update runtime exports and `_native.pyi` together while keeping native exceptions
   private from `btpc.__all__`.
   Tests and verification:
   Parameterize every core error category through parse, create, edit, verify, and
   filesystem paths. Assert exact public subclass, native cause/chain policy, all
   structured attributes, non-UTF-8 path round trips, callback exceptions, and
   cancellation. Add a regression proving an unknown category fails loudly rather
   than becoming generic silently. Run native stub parity/signature checks, strict
   typing, all Python tests, Rust adapter tests, wheel smoke tests, and spec validation.
   Evidence:
   - Added a private PyO3 exception hierarchy rooted at `_NativeError`, with one
     explicit subclass per stable core category and exhaustive Rust construction.
   - Structured offset, field, byte-path, limit, actual, and maximum attributes are
     attached without ignored failures. Python uses an exhaustive native-type table;
     unknown native categories raise loudly rather than silently degrading.
   - Python tests pass `61/61`, including structured context, non-UTF-8 paths,
     callback/cancellation behavior, and unknown mapping. Strict Clippy, Ruff,
     native-stub parity, mypy, Pyright, and specs `15/108` pass.
   Notes:

89. [x] Reduce full-buffer copies when Python parses metainfo
   Claimed by: Codex implementer (2026-07-01 23:44 PDT)
   Requirements:
   `PYAPI-BUFFER-001`, `PYAPI-GIL-001`, `PERF-MEM-001`, `PERF-PY-BOUNDARY-001`,
   `ARCH-SAFE-001`.
   Context:
   `Metainfo.from_bytes` currently converts a contiguous buffer to Python `bytes`
   and PyO3 then extracts an owned Rust vector, potentially making multiple complete
   input copies before parsing. Torrent metainfo is usually modest, so complexity
   must be justified by measured benefit rather than assumed.
   Implementation:
   First add copy/allocation and wall-time benchmarks for `bytes`, `bytearray`, and
   contiguous `memoryview` inputs across representative metainfo sizes. Establish
   the current copy count with test-only instrumentation. Implement the safest PyO3
   buffer path that produces at most one full ownership copy before constructing the
   owned core metainfo. Enforce `max_total_input` from buffer length before copying,
   reject non-contiguous or incompatible buffers with the existing clear `TypeError`,
   and ensure no Python buffer reference is used after releasing the GIL. Prefer safe
   Rust; any unsafe buffer access requires measured superiority, documented lifetime/
   contiguity invariants, focused tests, Miri where applicable, and explicit review.
   Keep `Metainfo.read` as the direct filesystem path and do not add implicit mmap
   semantics in this todo.
   Tests and verification:
   Cover empty and large buffers, writable/read-only inputs, sliced/non-contiguous
   memoryviews, mutation attempts during parse, resource limits before allocation,
   object lifetime/destruction, exceptions, and concurrent calls. Compare output and
   errors byte-for-byte with the original path. Record benchmark/copy-count evidence,
   run sanitizers/Miri where applicable, all Python parsing tests, strict typing,
   wheel smoke tests, and spec validation.
   Evidence:
   - Removed the Python `memoryview.tobytes()` copy. The extension now accepts the
     original buffer object, validates byte format/contiguity and total-input limits,
     then uses safe `PyBuffer<u8>::to_vec` for one owned copy before GIL release.
   - Added non-contiguous-buffer rejection while retaining bytes, bytearray, writable
     and read-only contiguous memoryview support and the direct path-reading route.
   - Focused parsing tests pass `13/13`; the full Python suite passes `62/62` with
     strict Clippy, Ruff, mypy, Pyright, native-stub parity, and specs `15/108`.
   Notes:
   - No unsafe project code or Python buffer reference crosses the detached parse.

90. [x] Replace paired Python edit flags with a typed three-state API
   Claimed by: Codex implementer (2026-07-01 23:53 PDT)
   Requirements:
   `PYAPI-EDIT-001`, `PYAPI-TEXT-001`, `PYAPI-TYPES-001`, `PYAPI-PARITY-001`,
   `META-FIELD-001`.
   Context:
   Python edit calls currently use paired values and `set_*` booleans to represent
   preserve, remove, and set. The behavior is capable but awkward, creates invalid
   combinations, and is harder for editors to explain. This pre-1.0 project can make
   the state model explicit while retaining a deliberate compatibility transition.
   Implementation:
   Introduce one typed public sentinel, exported canonically from `btpc.types` and
   re-exported from `btpc`, representing `UNCHANGED`. For optional editable fields,
   use a single keyword with `UNCHANGED` meaning preserve, `None` meaning remove,
   and the field's typed value meaning set. Apply the model consistently to trackers,
   web seeds, nodes, private, source, comment, created-by, creation date, and any
   equivalent typed optional metadata where removal is supported. Keep raw top-level
   and file-attribute maps byte-safe and explicit. Update the private native adapter
   to receive a normalized edit operation rather than paired flags.

   Because the paired form is already documented, either retain deprecated keyword
   aliases for one pre-1.0 release with conflict errors and `DeprecationWarning`, or
   record and document an intentional pre-1.0 break; do not silently reinterpret
   existing calls. Ensure generated signatures and editor completion expose the new
   union precisely without `Any`.
   Tests and verification:
   Exhaustively test preserve/remove/set for every field, explicit false/zero/empty
   values, conflicting legacy/new forms, warnings, unknown/raw fields, hash-domain
   changes, immutable source objects, root/domain imports, and Rust/Python parity.
   Run strict mypy and Pyright positive/negative fixtures, stub parity, documentation
   examples, all edit tests, wheel smoke tests, and spec validation.
   Evidence:
   - Added canonical `btpc.types.UNCHANGED` and root re-export. Optional edit fields
     now use `UNCHANGED` to preserve, `None` to remove, and typed values to set.
   - Applied the model to trackers, web seeds, nodes, private, source, comment,
     created-by, and creation date while preserving byte-safe raw/file maps and
     normalizing to the existing private native operation.
   - Documented the intentional pre-1.0 removal of paired `set_*` keywords. Focused
     edit/import/docs tests pass `9/9`; full Python passes `63/63`, with Ruff, mypy,
     Pyright, stub parity, and specs `15/108` green.
   Notes:

91. [x] Make Python metainfo equality exact without repeated full-byte copies
   Claimed by: Codex implementer (2026-07-02 00:04 PDT)
   Requirements:
   `PYAPI-IDENTITY-001`, `META-RAW-001`, `META-HASH-001`, `PYAPI-TYPES-001`,
   `PERF-PY-BOUNDARY-001`.
   Context:
   `Metainfo.__eq__` and `__hash__` currently access complete original Python bytes.
   Exact source-byte identity is the right semantic contract, but repeated equality
   and hashing should not force full-buffer copies or allocations, especially after
   native-object reuse removes those bytes from ordinary operation paths.
   Implementation:
   Benchmark and instrument current equality/hash behavior first. Keep equality based
   on exact original metainfo bytes, including top-level metadata and noncanonical
   encodings; never substitute v1/v2 info hashes or semantic equality. Add a native
   exact comparison path that reads the core-owned source bytes without materializing
   Python `bytes`. Choose and document one hash policy before implementation: prefer
   a cached native hash derived from the exact original bytes using Python-compatible
   equality semantics and cached for the object's lifetime; if correct cross-process
   Python hash behavior cannot be provided safely, make `Metainfo` explicitly
   unhashable before 1.0 rather than retaining an unexpectedly expensive hash.
   Preserve immutability, comparison with unrelated types, and root/domain class
   identity.
   Tests and verification:
   Test equal independent parses, top-level-only differences, noncanonical encodings,
   hash consistency, dictionary/set behavior if retained, unrelated operands,
   repeated-call allocation counts, large metadata, and concurrent comparisons under
   the documented GIL policy. Run strict typing, all metainfo tests, installed-wheel
   tests, boundary benchmarks, and spec validation.
   Evidence:
   - Added native exact-source rich comparison over core-owned original bytes, so
     independent parses compare without materializing Python byte strings.
   - Made public `Metainfo` intentionally unhashable before 1.0, documented the
     policy, and retained exact distinctions for top-level and noncanonical bytes.
   - Focused metainfo tests pass `14/14`; full Python passes `64/64`, with strict
     Clippy, Ruff, mypy, Pyright, native-stub parity, and specs `15/108` green.
   Notes:

92. [x] Introduce semantic byte-oriented metadata newtypes inside the Rust core
   Claimed by: Codex implementer (2026-07-02 00:13 PDT)
   Requirements:
   `RUSTAPI-METADATA-TYPES-001`, `RUSTAPI-BYTES-001`, `BENC-BYTES-001`,
   `ARCH-BOUND-001`, `RUSTAPI-COMPAT-001`.
   Context:
   Tracker tiers, web seeds, node hosts, and several textual metadata fields cross
   core subsystems as structurally similar nested byte vectors. The representation
   is protocol-correct but permits accidental field mixing and makes signatures
   harder to review. Lightweight internal types can improve clarity without imposing
   UTF-8 or prematurely expanding the public Rust API.
   Implementation:
   Add crate-private transparent newtypes for tracker URL/tier, web seed, node host,
   and other repeated metadata shapes that demonstrably cross subsystem boundaries.
   Centralize empty-value and structural validation on construction, expose borrowed
   raw-byte views for parsing/serialization, preserve unsigned-byte ordering, and
   avoid per-access allocation. Migrate creation options internals, owned metainfo,
   editor operations, magnet generation, and adapters incrementally. Keep public
   builder signatures and documented byte-oriented behavior stable through boundary
   conversions unless a separate Rust API compatibility decision explicitly promotes
   a type. Do not create a generic string abstraction that obscures field-specific
   validation.
   Tests and verification:
   Add compile-time type-separation tests, raw-byte/non-UTF-8 round trips, ordering and
   canonical serialization tests, empty/invalid metadata validation, magnet/edit/
   creation parity, and allocation-sensitive microbenchmarks. Run public API diffing,
   strict Clippy/rustdoc, all core/CLI/Python tests, Criterion smoke benchmarks, and
   spec validation.
   Evidence:
   - Added crate-private field-specific `TrackerUrl`, `TrackerTier`, `WebSeed`,
     `NodeHost`, and `MetadataText` wrappers and migrated creation-option internals
     while preserving every public byte-vector builder signature.
   - Construction validation remains at the public builder boundary; serialization
     uses borrowed raw-byte views or owned extraction without UTF-8 conversion or
     per-access allocation. Non-UTF-8 separation is covered by a focused unit test.
   - Strict core Clippy, all core unit/integration tests, creation/edit/magnet parity,
     doctests, rustdoc, public compile-fail/API tests, and specs `15/108` pass.
   Notes:
   - Types remain crate-private and deliberately field-specific; no generic text
     abstraction or public Rust compatibility change was introduced.

93. [x] Add end-to-end Python boundary performance benchmarks and budgets
   Claimed by: Codex implementer (2026-07-02 00:29 PDT)
   Requirements:
   `PERF-PY-BOUNDARY-001`, `PERF-BENCH-001`, `PYAPI-NATIVE-OBJECT-001`,
   `PYAPI-BUFFER-001`, `PYAPI-IDENTITY-001`, `BENCH-VALID-001`.
   Context:
   BTPC benchmarks torrent creation against external tools and has Rust kernel
   benchmarks, but architecture decisions at the Python/Rust boundary need their own
   repeatable evidence. These benchmarks must distinguish core work from adapter
   overhead without turning private helper functions into compatibility commitments.
   Implementation:
   Extend the benchmark harness with validated Python workflows for parse from path
   and each supported buffer type, first and repeated property access, magnet,
   no-op/top-level/info-changing edit, verification setup and representative verify,
   create setup, equality, and hashing if retained. Pair each workflow with the
   nearest direct Rust-core baseline or measure an explicitly isolated adapter phase.
   Record distributions, throughput/latency, peak RSS, input size, copy/reparse/
   serialization counters where instrumented, Python/Rust/tool versions, hardware,
   and cache state. Validate every result for byte/hash/report equivalence before
   accepting timing. Add statistically justified regression budgets for stable local
   synthetic fixtures; keep the Debian ISO and noisy cross-tool runs report-oriented
   rather than flaky per-commit gates.

   Produce the same deterministic machine-readable results and pretty ASCII summary
   conventions as the existing benchmark system. Document warmup, repetitions,
   outlier policy, environment isolation, and how to compare a branch against a
   baseline revision. Do not benchmark or publish private `_conversion` helpers as
   stable API units.
   Tests and verification:
   Add harness self-tests for invalid outputs, missing native builds, counter drift,
   timeout/failure handling, deterministic table columns, and baseline comparison.
   Run a short CI smoke profile and a full local profile, archive raw JSON plus the
   rendered table, compare optimized operations against their pre-change baseline,
   and run spec validation.
   Evidence:
   - Added a deterministic public-API boundary harness covering path/bytes/
     bytearray/memoryview parse, first/repeated properties, magnet, no-op and
     top-level edits, exact equality, create setup, representative verify, and create.
   - Every parse result and optional payload workflow is validated before timing.
     Results record schema, Python/BTPC versions, input bytes, warmups, medians,
     ranges, and repetitions in stable JSON plus deterministic ASCII columns.
   - Added compatible-baseline comparison with a documented 1.25x synthetic-fixture
     budget and tests for invalid configuration, incompatible results, and regression
     detection. All benchmark tests pass `36/36`; Ruff and specs `15/108` pass.
   Notes:
   - Large external datasets remain report-only; the harness benchmarks only public
     APIs and does not expose `_conversion` helpers as compatibility units.

94. [x] Migrate Python type-checking gates to Pyrefly
   Claimed by: Codex implementer (2026-07-02 00:42 PDT)
   Requirements:
   `PYAPI-PYREFLY-001`, `PYAPI-TYPE-COMPLETE-001`, `TEST-PY-TYPING-001`,
   `RELEASE-PY-TYPING-001`.
   Context:
   BTPC currently runs strict mypy and Pyright checks. Pyrefly should become the
   primary command-line type checker while preserving complete annotations and
   standard stubs for users of Pyright/Pylance and other language servers.
   Implementation:
   Add a pinned Pyrefly development dependency and repository configuration covering
   the public package, Python tests, and external installed-wheel consumer fixtures.
   Begin by running Pyrefly alongside the existing mypy/Pyright gates and classify
   every diagnostic as a real typing gap, configuration issue, or checker difference;
   fix annotations and public APIs rather than adding broad suppressions. Port the
   positive `assert_type` coverage and negative invalid-call fixtures so Pyrefly
   proves every documented public API family, callbacks, raw-byte values, path-like
   inputs, exceptions, and the public module layout.

   Once Pyrefly is green with equivalent or stronger coverage, make it the required
   pre-push, CI, release, and contributor-documentation command. Remove mypy and
   Pyright dependencies/configuration from mandatory gates only after a recorded
   parity audit; keep a lightweight Pyright compatibility smoke fixture if necessary
   to protect Pylance consumers, but do not maintain redundant full checker matrices
   without demonstrated value. Ensure checks run against both the source tree and a
   clean installed wheel so repository import paths cannot hide missing annotations
   or package data.
   Tests and verification:
   Run Pyrefly over source and tests with zero errors, execute positive and negative
   consumer fixtures against a built wheel, verify `py.typed` and native/public stubs
   in wheel and sdist, run native-stub parity, and demonstrate that an intentionally
   invalid copied fixture fails. Run the complete Python suite, Ruff, packaging smoke
   tests, updated pre-commit/pre-push hooks, affected GitHub Actions locally where
   possible, and spec validation. Record the removed checker commands and the parity
   evidence in the todo.
   Evidence:
   - Added and locked `pyrefly==1.1.1` with repository configuration. Positive
     package/tests/consumer sources report zero errors; the negative fixture fails
     with exactly three expected argument diagnostics.
   - Added `scripts/check_python_types.sh` as the required pre-push, Makefile, CI,
     and contributor gate. Removed mypy dependency/configuration after parity and
     retained one strict Pyright consumer smoke for Pylance compatibility.
   - Built wheel and sdist, verified typing artifacts and stub parity (`11/11`
     release/typing tests), and ran all Python tests (`65/65`), Ruff, Pyrefly,
     Pyright, native-stub parity, and specs `15/108` successfully.
   Notes:
   - Pyrefly project mode currently follows the repository ignore entry for the
     source `python/` directory, so the gate supplies the explicit checked file list.

95. [x] Upgrade public Python docstrings for editor help and generated API reference
   Claimed by: Codex implementer (2026-07-02 13:45 PDT)
   Requirements:
   `PYAPI-DOCSTRING-001`, `PYAPI-DOC-001`, `PYAPI-MODULES-001`,
   `PYAPI-TYPE-COMPLETE-001`, `TEST-TDD-001`.
   Context:
   The public Python surface has docstrings, but most are one-line labels such as
   “Typed creation options matching core defaults” or “Verify a payload using the
   native core verifier.” These satisfy presence checks without giving editor users
   enough help to choose options, understand byte/text and canonicalization rules,
   interpret callbacks, or use common workflows. The same docstrings will later feed
   the MkDocs/mkdocstrings API reference, so the source text must read like polished
   library documentation rather than implementation notes or generated filler.
   Implementation plan:
   Begin with a failing `tests/python/test_docstrings.py` quality inventory. Enumerate
   the supported names from each public module's `__all__` plus documented public
   methods/properties, and require a non-empty summary for all of them. Define a
   high-use tier that requires the relevant structured sections and at least one
   concise example: `CreateOptions`, `create`, `create_bytes`, `Metainfo`,
   `Metainfo.from_bytes`, `Metainfo.read`, `Metainfo.edit`, `Metainfo.verify`,
   `Metainfo.magnet`, `Metainfo.to_bytes`, top-level `verify`, `ParseOptions`, and
   cancellation/progress APIs. Keep the test semantic and maintainable: assert
   coverage and executable examples, not exact prose, word counts, or brittle style
   snapshots.

   Rewrite public module and object docstrings using the configured Google style and
   a restrained NumPy/Polars-like tone. Start with a direct summary, then add only the
   sections that help a caller. Document all `CreateOptions` and `ParseOptions`
   attributes, including mode defaults, automatic versus explicit piece length,
   thread selection, tracker tier shape, text encoding, private/source/comment,
   default `btpc/<version>` creator behavior, explicit creator omission, creation
   date reproducibility, and which options alter the info hash. Document progress
   callbacks as `(completed_bytes, total_bytes, completed_pieces)`, callback failure
   propagation, and cooperative cancellation.

   For creation, explain the difference between `create_bytes` and atomic `create`,
   destination overwrite behavior, durability, returned metrics/hashes, filesystem
   errors, cancellation, and protocol-validation errors. For `Metainfo`, explain that
   parsing retains exact original bytes, hashes the original raw `info` dictionary,
   and serializes canonically through `to_bytes(canonical=True)`. Document accepted
   contiguous buffers and parse limits for `from_bytes`; path I/O for `read`; raw
   bytes and optional decoded views; magnet inclusion switches; validation reports;
   and exact-byte equality semantics.

   Give `Metainfo.edit` special treatment: clearly demonstrate `UNCHANGED` preserving
   a field, `None` removing it, and a typed value replacing it; distinguish top-level
   edits that preserve info hashes from info-dictionary edits that change them. For
   verification, document payload-root resolution, v1/v2/hybrid hash domains,
   fail-fast and extra-file behavior, deterministic mismatch reports, operational
   exceptions, progress, and cancellation. Improve exception class docs with when
   users should expect each category and describe structured `BtpcError` attributes
   once on the base class rather than repeating them.

   Use short examples that demonstrate real calls and ordinary root imports. Prefer
   a small valid in-memory torrent for parse/magnet/serialization examples and a tiny
   temporary payload for create/verify examples. Keep examples self-contained where
   practical; do not use fake signatures, unexplained `...`, huge fixtures, private
   `_native` names, requirement IDs, or performance claims. Simple properties such
   as `piece_count`, `hex`, and `cancelled` should remain one concise sentence unless
   a caveat is genuinely needed. Add inline comments only for non-obvious invariants
   or boundary choices; do not comment routine assignments or conversions.

   Review module-level docstrings and `__all__` after the object pass so generated
   navigation has useful introductions and no public object is documented twice with
   conflicting wording. Ensure canonical defining modules remain `btpc.creation`,
   `btpc.metainfo`, `btpc.verification`, `btpc.types`, and `btpc.errors`, while root
   re-exports retain the same docstrings and object identity. Do not change runtime
   signatures or behavior solely to make documentation easier.
   Tests and verification:
   Add focused doctest or equivalent execution for self-contained examples and
   ordinary pytest coverage for filesystem examples, using temporary directories and
   tiny payloads. Assert examples work through both canonical module imports and the
   common `btpc` root imports where shown. Run Ruff docstring lint/format, Pyrefly,
   the Pyright compatibility fixture, native-stub parity, all Python tests, clean
   installed-wheel smoke tests, and spec validation. If mkdocstrings is available by
   implementation time, build the Python API reference in strict mode and inspect
   the rendered signatures, attribute tables, examples, and cross-references; do not
   block this todo on creating the full documentation site.
   Evidence:
   - Added `tests/python/test_docstrings.py` with a semantic inventory of every
     public module export, documented public method/property, and dataclass field.
     It requires structured examples for the high-use workflow tier, checks root
     re-export identity/docstring parity, and executes examples with `doctest`.
     The test failed first on one-line `CreateOptions`/`CreateMetrics` docs and now
     passes `6/6` both from the source tree and from a clean installed wheel.
   - Rewrote public creation, parsing, editing, magnet, serialization, verification,
     value-type, report, and exception docstrings in Google style. The docs cover all
     create/parse options, strict byte/text boundaries, exact source-byte identity,
     canonical output, info-hash-changing edits, atomic overwrite/durability,
     callback values and failure propagation, cooperative cancellation, payload-root
     resolution, v1/v2/hybrid hash domains, and structured error attributes.
   - `uv run ruff check .` and `uv run ruff format --check .` passed across 51 files.
     `scripts/check_python_types.sh` passed Pyrefly positive sources and the expected
     three-error negative fixture; the Pyright consumer reported 0 errors, and
     `scripts/check_native_stub.py` confirmed native stub/runtime export parity.
   - The complete Python suite passed `71/71`. Specification validation passed 15
     specs and 109 requirements, and documentation link validation passed across 33
     Markdown files.
   - Built the locked CPython 3.14 release wheel with Maturin, installed it with
     pytest into `/tmp/btpc-docstring-wheel-venv`, ran v1/v2/hybrid create/read/
     magnet/verify smoke checks outside the checkout, and reran the docstring suite
     `6/6`. The wheel contains all five public modules, `py.typed`, and `_native.pyi`.
   Notes:
   - MkDocs/mkdocstrings is not installed in the locked development environment, so
     the optional strict generated-site build was not applicable to this todo.

96. [x] Bootstrap the locked documentation toolchain and deterministic site builder
   Claimed by: Codex implementer (2026-07-02 14:00 PDT)
   Requirements:
   `DOCSITE-ARCH-001`, `DOCSITE-BUILD-001`, `DOCSITE-UX-001`, `TEST-TDD-001`.
   Context:
   `DOCUMENTATION_PLAN.md` and `specs/documentation-site.md` define one unified
   Material for MkDocs site, but the repository does not yet have `mkdocs.yml`, a
   dedicated documentation dependency group, or one build entry point shared by
   local development and CI. This todo creates the smallest complete site skeleton;
   later todos populate each reference surface and harden deployment.
   Implementation:
   Start with failing tests under `tests/docs/` for the required configuration,
   clean-output behavior, project-subpath-safe URLs, expected root files, and a
   successful strict minimal build. Add current stable open-source MkDocs Material
   and mkdocstrings Python tooling through `uv add --group docs` (or the current uv
   equivalent) and commit the resolved lockfile. Keep documentation packages out of
   BTPC's runtime dependencies. Use the supported Material 9.x line unless the
   current release documentation requires a newer compatible major; do not use
   sponsor-only/Insiders features.

   Add `mkdocs.yml` with the canonical project site URL
   `https://burritothief.github.io/btpc/`, repository/edit links, explicit
   navigation, strict Markdown extensions, client-side search, and deterministic
   theme settings. Add a cross-platform repository build entry point, preferably a
   typed Python script rather than shell-only orchestration, that removes its own
   staging directory and creates the complete site at an explicit destination.
   Reserve documented stages for CLI generation, MkDocs, rustdoc, and generated-site
   validation even if the later stages are initially placeholders. Add `make
   docs-site` and `make docs-serve` (or equivalently named existing-project targets),
   ignore generated `site/`/staging output, and document the commands in
   `CONTRIBUTING.md`. The command must work from outside the repository checkout by
   resolving paths relative to the script, not the caller's current directory.
   Tests and verification:
   Prove the tests fail before the toolchain/configuration exists. Run the focused
   docs tests, `uv lock --check`, a clean strict site build, and the local serve
   command long enough to request the homepage. Run Ruff on new Python code,
   `uv run python scripts/check_specs.py`, `uv run python scripts/check_docs.py`,
   and inspect the generated artifact to confirm `index.html` is at its root and no
   generated output is tracked. Record exact package versions and commands.
   Evidence:
   - Added `tests/docs/test_site_builder.py` first; all five tests initially failed
     because the docs group, `mkdocs.yml`, root pages, and builder were absent. The
     suite now passes `5/5`, including invocation from outside the checkout, stale
     destination removal, reserved stage order, strict output, and root-relative
     asset rejection.
   - Added the docs dependency group through `uv add --group docs`, resolving and
     locking MkDocs 1.6.1, Material for MkDocs 9.7.6, mkdocstrings 1.0.4, and
     mkdocstrings-python 2.0.5 without adding runtime dependencies. `uv lock --check`
     passes against the resulting 57-package environment.
   - Added `mkdocs.yml` with the canonical Pages project URL, repository/edit links,
     explicit navigation, Material search and deterministic local-font settings,
     Google-style mkdocstrings configuration, and supported Markdown extensions.
     Added development-labelled `docs/index.md` and a custom `docs/404.md`.
   - Added typed `scripts/build_docs_site.py` with explicit `cli`, `mkdocs`,
     `rustdoc`, and `validate` stages. It resolves the repository from its own path,
     removes `.tmp/docs-site` and the requested destination, runs MkDocs in strict
     mode, verifies root `index.html`/`404.html`, and copies a fresh artifact.
   - Added `make docs-site` and `make docs-serve`, contributor instructions, and
     ignored `site/`. A strict `make docs-site` succeeded; generated files included
     root `index.html`, `404.html`, search, sitemap, local assets, and guide pages,
     while `git ls-files site .tmp/docs-site` remained empty.
   - Ruff check/format passed for the builder and docs tests. Specification
     validation passed 16 specs/118 requirements, and Markdown link validation
     passed across 36 files. The locked server served
     `http://127.0.0.1:8123/btpc/`; curl verified the BTPC title and local Material
     stylesheet before clean shutdown.
   Notes:
   - CLI generation and rustdoc stages intentionally validate their current source
     inputs only; Todos 99 and 100 replace them with fresh generated-site content.

97. [x] Build the production information architecture and accessible Material theme
   Claimed by: Codex implementer (2026-07-02 14:15 PDT)
   Requirements:
   `DOCSITE-ARCH-001`, `DOCSITE-UX-001`, `DOCSITE-BUILD-001`, `TEST-TDD-001`.
   Context:
   The existing `docs/*.md` files contain useful CLI, compatibility, Python, Rust,
   and security material, but they are not organized as a coherent user journey.
   The site needs production navigation and presentation before generated API
   references are integrated. Preserve additive content instead of replacing it
   with marketing filler.
   Implementation:
   Add failing navigation/content tests for every required top-level section and
   key landing page in `DOCUMENTATION_PLAN.md`. Reorganize handwritten site content
   into Getting Started, Guides, Concepts, CLI, Python, Rust, Performance,
   Compatibility, Security, and Contributing sections. Write concise installation
   and quick-start pages for CLI, Python, and Rust; task-oriented creation,
   inspection, validation, verification, editing, configuration, preset, and shell
   completion guides; and v1, v2, hybrid, piece-length, reproducibility, and byte
   semantics concepts. Reuse tested README examples and existing docs rather than
   duplicating incompatible commands. Link to normative specs only from an explicit
   contributor/developer area; do not confuse implementation contracts with public
   API documentation.

   Configure responsive light/dark palettes, code-copy controls, anchored headings,
   syntax highlighting, readable tables/admonitions, keyboard-visible focus, and
   reduced-motion-safe behavior. Add a useful custom `404.md`. Disable external
   fonts and do not add analytics, cookies, advertising, third-party runtime
   JavaScript, or network-dependent page assets. Use text or existing project assets
   until a reviewed logo exists. Require descriptive page titles, one H1 per page,
   meaningful link text, image alt text, and a visible development-documentation
   notice until the first stable release. Keep CSS overrides minimal, documented,
   and covered by the build.
   Tests and verification:
   Run navigation/content tests first and retain their initial failures. Run the
   complete strict docs build, Markdown link checker, codespell over handwritten
   docs, and an HTML inspection test covering titles, H1s, missing alt text, external
   runtime assets, and required 404 output. Serve the generated project site under
   `/btpc/` and manually smoke-test keyboard navigation, search, both palettes, code
   copy, narrow viewport layout, and the 404 page. Record screenshots or precise
   observations without committing temporary captures. Run spec validation.
   Evidence:
   - TDD navigation/content baseline: `uv run pytest
     tests/docs/test_information_architecture.py -q` initially reported 2 failed,
     1 passed before the production navigation, pages, and theme were added; the
     retained suite now reports 3 passed.
   - `uv run pytest tests/docs -q` reports 8 passed, including generated-HTML
     checks for titles, one H1, image alt text, local runtime assets, viewport,
     search, palettes, code-copy configuration, announcement, and the deployed
     root `404.html`.
   - `make docs-site` completes the strict deterministic build; `uv run python
     scripts/check_docs.py` validates links in 58 Markdown files; `uv run
     codespell docs --skip='docs/completions/*,docs/reference/*'`, targeted Ruff
     lint/format checks, and `git diff --check` pass.
   - `uv run python scripts/check_specs.py` validates 16 specs and 118
     requirements after the public CLI, Python, and Rust documentation paths were
     updated to their production locations.
   - An HTTP smoke server mounted the generated site at `/btpc/`; the home page,
     local Material CSS, search index, and custom `/btpc/404.html` returned 200,
     while generated markup exposed the responsive viewport, search and palette
     controls, and development notice. The stylesheet contains narrow-layout
     media rules, keyboard-visible focus styles, and reduced-motion handling.
   Notes:
   - The in-app browser backend was unavailable: documented browser discovery
     returned no browsers. Generated-HTML inspection and direct HTTP requests
     supplied precise smoke observations instead; no temporary captures were
     committed.

98. [x] Generate a complete Python API reference from public btpc modules
   Claimed by: Codex implementer (2026-07-02 14:18 PDT)
   Requirements:
   `DOCSITE-PYTHON-001`, `DOCSITE-BUILD-001`, `DOCSITE-QUALITY-001`,
   `PYAPI-DOCSTRING-001`, `PYAPI-MODULES-001`, `TEST-TDD-001`.
   Context:
   Todo 95 upgrades editor-facing docstrings. This todo turns those same docstrings,
   signatures, annotations, and examples into the website's canonical Python API
   reference without exposing private PyO3 or conversion machinery. It must execute
   after Todo 95 and consume its public inventory rather than inventing a second API
   description.
   Implementation:
   Begin with failing documentation tests that enumerate every supported public
   symbol from the public modules and root re-exports. Add small mkdocstrings pages
   for `btpc.creation`, `btpc.metainfo`, `btpc.verification`, `btpc.types`, and
   `btpc.errors`. Configure the current Griffe-based Python handler with an explicit
   `paths: [python]`-style source path, Google docstring parsing, annotation-aware
   signatures, stable source ordering, cross-reference support, and only features
   available in the open-source distribution. Prefer static source collection so
   documentation can build without compiling the native extension; if a runtime
   import is genuinely necessary, document why and build/install the extension in
   an isolated environment rather than relying on an editable developer install.

   Render only intentional public names. Ensure root aliases resolve to one
   canonical defining-module page, hide `_native`, `_conversion`, and other private
   symbols, and preserve visible return types, exceptions, attributes, overloads,
   callback contracts, and examples. Add a friendly Python overview that links
   creation, parsing, editing, magnet, serialization, and verification workflows to
   their exact reference objects. Do not duplicate or rewrite source docstrings in
   Markdown merely to improve rendering; fix deficient source documentation and
   Todo 95's quality tests instead.
   Tests and verification:
   Demonstrate the public-inventory test failing before pages/configuration are
   added. Run the Todo 95 docstring tests, Pyrefly and Pyright consumer smoke,
   mkdocstrings collection with warnings treated as errors, and the strict complete
   site build. Assert every expected public symbol has one canonical reference URL,
   no private symbol is rendered, and key signatures/types appear in generated HTML.
   Build from a clean checkout context with no editable `btpc` import available.
   Run Ruff, the Python test suite, spec validation, and generated-site link checks.
   Evidence:
   - Added `tests/docs/test_python_reference.py` first; its initial run reported
     3 failures for absent module pages, missing annotation-aware handler options,
     and absent canonical generated anchors. The retained suite now reports 4
     passed, and the complete docs suite reports 12 passed.
   - Added explicit mkdocstrings pages for all exports in `btpc.creation`,
     `btpc.metainfo`, `btpc.verification`, `btpc.types`, and `btpc.errors`, plus a
     root-alias API index. Tests derive the inventory from each source `__all__`,
     require one generated defining-module anchor per symbol, and reject private
     native/conversion anchors.
   - Configured the locked Griffe handler for the `python` source path, Google
     docstrings, source ordering, annotated signatures, cross-reference links,
     brief annotation paths, and hidden source. A test copies the package without
     native binaries and loads every public module with runtime inspection disabled.
   - Generated HTML preserves callback annotations, `Raises` sections,
     `Metainfo.from_bytes` parameter/return types, and a live cross-reference from
     `ParseOptions` to its canonical types page. `make docs-site` completes in
     strict mode and `scripts/check_docs.py` validates 63 Markdown files.
   - Todo 95's docstring suite reports 6 passed; `scripts/check_python_types.sh`
     passes Pyrefly and Pyright consumer checks; native stub parity passes; the
     complete Python suite reports 71 passed. Repository Ruff lint/format,
     codespell, `git diff --check`, and spec validation (16 specs, 118
     requirements) pass.
   Notes:
   - Reference generation is static and does not import `btpc` or require the
     checkout's compiled `_native` extension.

99. [x] Embed fresh warning-free btpc-core rustdoc into the unified site
   Claimed by: Codex implementer (2026-07-02 14:27 PDT)
   Requirements:
   `DOCSITE-RUST-001`, `DOCSITE-BUILD-001`, `DOCSITE-QUALITY-001`,
   `RUSTAPI-DOC-001`, `TEST-TDD-001`.
   Context:
   Rust users need native rustdoc features, but the output must appear under the
   same GitHub Pages project site and must never reuse stale `target/doc` files.
   Existing CI already denies rustdoc warnings; this todo integrates that output
   into the documentation artifact and makes its relationship to `main` explicit.
   Implementation:
   Add failing docs-build tests for the Rust landing page, stable
   `/rust/btpc_core/` entry point, same-origin links, and stale-output rejection.
   Extend the shared site builder to run `cargo doc -p btpc-core --all-features
   --no-deps` with `RUSTDOCFLAGS=-D warnings` using a dedicated clean target or
   output directory. Copy only the fresh files required for `btpc_core` and rustdoc
   static assets into the staged MkDocs site at `rust/btpc_core/`; do not commit
   generated HTML and do not copy dependency documentation accidentally.

   Add a Rust overview and quick start linking into embedded rustdoc, source code,
   and executable examples. Audit public crate/module/item docs for warnings,
   broken intra-doc links, missing error/panic/resource notes, and examples that no
   longer compile. Keep wording concise and source-native. State that embedded docs
   describe `main`, and add a disabled/future docs.rs link only when the crate is
   actually published rather than linking to a nonexistent release page.
   Tests and verification:
   Retain the initial failing integration tests. Run `cargo test -p btpc-core --doc`,
   warning-denied `cargo doc`, and the strict shared site build twice after planting
   a sentinel in the first rustdoc output; prove the sentinel cannot survive the
   clean rebuild. Validate the Rust landing page and `rust/btpc_core/index.html`
   through a local HTTP server under `/btpc/`, check internal links/assets, run
   strict clippy for affected Rust code if any, and run spec validation.
   Evidence:
   - Added `tests/docs/test_rustdoc_site.py` first; its initial run reported 3
     failures for the absent landing link, absent dedicated rustdoc output, and
     absent embedded crate entry point. The retained integration suite reports 3
     passed and the complete docs suite reports 15 passed.
   - The shared builder now deletes the dedicated rustdoc `doc` directory, runs
     `cargo doc -p btpc-core --all-features --no-deps` with
     `RUSTDOCFLAGS=-D warnings`, and copies only `btpc_core`, its source pages,
     rustdoc static/search/implementation assets, and required root support files
     below `site/rust/`. No dependency HTML directory is published.
   - The integration test builds twice, plants a sentinel in the first dedicated
     rustdoc output, and proves it is absent from both the regenerated target and
     second site. It also verifies the stable `rust/btpc_core/index.html` entry,
     local runtime assets, and absence of root-relative project-breaking URLs.
   - `cargo test -p btpc-core --doc` passes the crate doctest; warning-denied
     `cargo doc` completes without broken-link or documentation warnings; strict
     all-target/all-feature `btpc-core` Clippy passes. No Rust source changes were
     required by the documentation audit.
   - `make docs-site` completes in strict mode with a 6.1 MiB Rust subtree;
     documentation links validate across 63 Markdown files, codespell passes for
     the Rust guide, and spec validation reports 16 specs and 118 requirements.
   - A local HTTP server mounted the artifact at `/btpc/`; the Rust landing,
     `/btpc/rust/btpc_core/`, rustdoc CSS, crate source page, and `create` module
     page all returned 200 with working relative navigation.
   Notes:
   - The ignored dedicated Cargo target retains compilation cache between builds,
     but its complete `doc` directory is removed before every rustdoc invocation,
     preventing stale generated documentation from surviving.

100. [x] Publish readable generated CLI command reference pages without drift
   Claimed by: Codex implementer (2026-07-02 14:38 PDT)
   Requirements:
   `DOCSITE-CLI-001`, `DOCSITE-BUILD-001`, `DOCSITE-QUALITY-001`,
   `CLI-DOC-001`, `RELEASE-CLI-DOC-001`, `TEST-TDD-001`.
   Context:
   `scripts/generate-cli-reference.sh` and `crates/btpc-cli/tests/reference.rs`
   currently maintain raw help text, manpage, and completion artifacts. The website
   needs navigable Markdown command pages while keeping the Clap model as the sole
   command definition and preserving packaging artifacts.
   Implementation:
   Start with failing generator and golden tests for every top-level command,
   nested config/completion command, global option, alias, and cross-link required
   by the current command model. Refactor or extend generation so one invocation
   emits deterministic website Markdown pages with synopsis, description, options,
   inherited global flags, subcommands, exit behavior links, and concise examples.
   Keep manpage and shell-completion generation intact for packaging. Generate from
   Clap metadata or the running pinned workspace binary; never hand-maintain a
   second argument table. Make output independent of terminal width, color, locale,
   home/config files, and current directory, and ensure secrets or real tracker
   configuration can never enter generated docs.

   Place generated web pages under the documented CLI reference directory and add
   them to MkDocs navigation. Provide parent/child breadcrumbs or links and link
   common flags to task-oriented guides. Extend the drift test so generation into a
   temporary directory compares byte-for-byte with checked-in generated source.
   The shared site build must run this drift check before rendering. Preserve
   existing raw artifacts if they remain release inputs; otherwise remove only
   demonstrably redundant web copies and update all consumers atomically.
   Tests and verification:
   Show the new tests failing before implementation. Run focused CLI reference
   tests, the generator twice and compare outputs, config-isolated CLI tests, the
   strict site build, codespell with justified generated-output exclusions, and
   generated-site link/anchor checks. Confirm all commands shown by `btpc --help`
   have a reference page and no undocumented extra page exists. Run Rust formatting,
   strict clippy for `btpc-cli`, relevant CLI tests, and spec validation.
   Evidence:
   - Added the web-reference golden test first; it failed because the maintenance
     generator entrypoint and checked-in pages did not exist. The retained
     `btpc-cli` reference suite now reports 3 passed for raw help/manpage/
     completions, deterministic web generation, and byte-for-byte checked-in
     parity.
   - Added a maintenance-only pre-Clap `__generate-markdown` entrypoint whose
     renderer recursively walks the built Clap command tree. It emits 28 Markdown
     pages with synopsis, descriptions, positional arguments, options, inherited
     globals, subcommand/parent links, deprecated aliases, exit/config links, and
     examples. The maintenance entrypoint is absent from help, completions, raw
     references, and website pages.
   - Completed missing Clap-native help text for create, edit, validate, preset,
     and tracker arguments. The golden test rejects any generated blank
     description, documents the `completions` and `clear-created-by` aliases, and
     proves identical output under a different cwd, locale, home, and fake secret
     configuration environment.
   - `scripts/generate-cli-reference.sh` now rebuilds the workspace binary unless
     an explicit executable `BTPC_BIN` is supplied, generates raw help, manpage,
     all five completions, and the website tree in one invocation, and reproduces
     62 files byte-for-byte across two temporary runs. Existing release artifacts
     remain generated and current.
   - The shared site builder regenerates the web reference into staging and fails
     on any file-set or byte drift before MkDocs runs. MkDocs navigation includes
     every generated page; `tests/docs/test_cli_reference.py` proves the nav and
     source page sets are identical.
   - The complete `btpc-cli` suite reports 89 passed. Rust formatting and strict
     all-target/all-feature Clippy pass; the complete docs suite reports 16 passed;
     the strict site build succeeds; links validate across 91 Markdown files; and
     spec validation reports 16 specs and 118 requirements.
   - A local `/btpc/` HTTP smoke test verified the command root, deprecated alias,
     nested preset page, global options, option alias, exit-code anchor, and search
     index. Source/handwritten codespell checks pass; generated CLI output is
     excluded because it is byte-checked against the Clap model.
   Notes:
   - The generated command filenames are flat and deterministic (for example,
     `config-preset-save.md`); parent/child links and hierarchical Material
     navigation present the logical command tree without filesystem ambiguity.

101. [x] Add production generated-site QA, size budgets, and contributor gates
   Claimed by: Codex implementer (2026-07-02 14:52 PDT)
   Requirements:
   `DOCSITE-BUILD-001`, `DOCSITE-QUALITY-001`, `DOCSITE-UX-001`,
   `TEST-TDD-001`.
   Context:
   A successful MkDocs process is insufficient: project-subpath mistakes, missing
   copied rustdoc assets, private Python names, broken anchors, and oversized output
   can still produce a bad Pages deployment. This todo establishes one fast local
   quality gate over the complete generated artifact.
   Implementation:
   Add failing tests for representative broken internal links, missing anchors,
   absent assets, root-relative links that escape `/btpc/`, `file://` or localhost
   URLs, leaked absolute checkout paths, missing canonical metadata, duplicate page
   titles, required entry points, private API names, and an intentionally exceeded
   artifact-size budget. Implement or adopt a maintained deterministic link/HTML
   checker that can run offline against the staged site. Keep external HTTP link
   checking out of the pull-request gate. Set and document a realistic initial
   compressed and uncompressed site-size budget based on the first complete build;
   changes to that budget require recorded evidence, not silent increases.

   Consolidate the strict build, CLI drift check, Python inventory check, rustdoc,
   doctests, offline generated-site validation, and handwritten spelling check into
   `make docs-check` or the project's equivalent canonical command. Add the fast
   applicable subset to pre-commit and the full command to pre-push/manual hooks
   without duplicating implementation logic. Update `CONTRIBUTING.md` and
   `AGENTS.md` with source ownership (handwritten vs generated), local preview,
   verification, and the rule that generated HTML is never committed.
   Tests and verification:
   Keep one fixture or parameterized test proving each failure class is detected,
   then run all docs tests and the canonical docs gate from a clean tree and from a
   non-repository current directory. Run pre-commit on all files and the pre-push
   documentation hook, Ruff, codespell, Rust doctests/rustdoc, spec validation, and
   the broader repository documentation checks. Record initial artifact sizes and
   configured budgets.
   Evidence:
   - Added `tests/docs/test_site_quality.py` first; collection initially failed
     because the production validator did not exist. The retained parameterized
     suite now passes 13 cases covering broken targets, missing anchors/assets,
     project-root escapes, `file://`, localhost, checkout-path leakage, missing
     canonical metadata, duplicate titles, required entries, private Python API
     anchors, both size budgets, and a valid complete fixture.
   - Added `scripts/check_docs_site.py`, an offline deterministic generated-site
     validator. It resolves internal links under `/btpc/`, validates static-page
     anchors and all local assets, rejects external runtime assets and forbidden
     local URLs, checks canonical metadata and title uniqueness, protects Python
     reference privacy, inventories required entry points, and reports normalized
     gzip and uncompressed sizes.
   - The checker exposed and fixed root `404.html` links that escaped the project
     subpath and duplicate MkDocs titles caused by generic navigation labels.
     Generated CLI pages now carry unique title metadata, and handwritten overview
     pages declare descriptive titles. Rustdoc links/assets are checked, while its
     generator-specific dynamic/range anchors remain covered by warning-denied
     rustdoc rather than generic static-anchor rules.
   - The final artifact contains 397 files and 185 HTML pages, measuring 12,295,435
     uncompressed bytes and 3,195,405 deterministic normalized gzip bytes. Enforced
     budgets are 16,000,000 and 4,500,000 bytes, with the baseline and evidence rule
     documented in `CONTRIBUTING.md`, `DOCUMENTATION_PLAN.md`, public contribution
     guidance, and `AGENTS.md`.
   - Added canonical `make docs-check` and fast `make docs-fast` targets. The full
     gate consolidates Rust doctests, all docs tests, strict complete-site build,
     CLI drift, Python inventory, warning-denied rustdoc, offline artifact QA,
     source links, and handwritten spelling. It passes from the repository and via
     `make -C /Users/jeff/src/btpc docs-check` from `/tmp`; both runs report 31 docs
     tests and identical artifact sizes.
   - Added the fast applicable gate to pre-commit and the full canonical gate to
     pre-push/manual hooks and scheduled maintenance CI. `pre-commit --all-files`
     reformatted one new file on its first run and passed on rerun; the actual
     `pre-push-gate` hook passes. Repository Ruff lint/format, `git diff --check`,
     and spec validation (16 specs, 118 requirements) pass.
   Notes:
   - External HTTP link validation intentionally remains outside the merge gate;
     Todo 105 adds scheduled network health checks.

102. [x] Add the least-privilege GitHub Pages build and deployment workflow
   Claimed by: Codex implementer (2026-07-02 15:04 PDT)
   Requirements:
   `DOCSITE-DEPLOY-001`, `DOCSITE-BUILD-001`, `DOCSITE-QUALITY-001`,
   `SEC-DEPS-001`.
   Context:
   The complete site can now be built and validated locally. It must rebuild for
   every pull request and every push to `main`, while only trusted default-branch
   runs may deploy. The repository already pins Actions to immutable SHAs and runs
   `zizmor`; preserve those supply-chain rules.
   Implementation:
   Add `.github/workflows/docs.yml` triggered by `pull_request`, pushes to `main`,
   and `workflow_dispatch`. Do not use restrictive path filters: the documentation
   check should be a reliable required status, and code/docstring changes anywhere
   can affect generated API output. Give the workflow read-only default permissions.
   The build job must check out without persisted credentials, install the pinned
   Rust toolchain and locked uv environment with safe caches, invoke the canonical
   docs gate/build command, call the official Pages configuration action, and upload
   exactly the staged site using the official Pages artifact action. Pin every
   action to a reviewed immutable commit SHA with its release tag in a comment.

   Add a dependent deploy job that runs only for a successful trusted `main` push or
   explicitly authorized manual dispatch, never for pull requests. Give only this
   job `pages: write` and `id-token: write`, attach it to the `github-pages`
   environment with the deployment URL output, and use the official Pages deploy
   action. Configure workflow/deployment concurrency so obsolete queued builds are
   cancelled while an active production deployment is not interrupted. Set explicit
   timeouts, keep fork PRs safe, avoid repository write tokens and branch-pushing
   deploy tools, and retain artifacts only as long as operationally useful.
   Tests and verification:
   Parse workflow YAML, run `zizmor` and existing action-pin checks, and add a
   structural test that asserts triggers, job dependencies, conditions,
   permissions, environment, concurrency, timeouts, locked install, canonical build
   command, and official Pages actions. Exercise the build job locally where
   feasible without pretending to deploy. Push through the normal implementation
   workflow and record successful pull-request/build and trusted deployment run URLs
   when available; if repository Pages settings block deployment, record the exact
   GitHub response and leave only Todo 104's owner-setting step outstanding. Run
   spec validation and the full docs gate.
   Evidence:
   - Added `tests/docs/test_pages_workflow.py` first; all 3 structural tests failed
     because `.github/workflows/docs.yml` was absent. The retained suite now passes
     and asserts trigger coverage, absence of path filters, workflow/job
     permissions, dependencies, trust condition, environments, concurrency,
     timeouts, locked installation, canonical build command, artifact path and
     retention, and immutable official-action pins.
   - Added the `Documentation` workflow for every pull request, push to `main`, and
     manual dispatch. The build job has only inherited `contents: read`, checks out
     without credentials, installs Rust 1.94.1 and the locked uv environment,
     executes `make docs-check`, configures Pages, and uploads exactly `site/` for
     one day.
   - The dependent deploy job runs only after a successful build on the `main`
     branch for trusted push or manual-dispatch events. Only that job receives
     `pages: write` and `id-token: write`, uses the `github-pages` environment and
     deployment URL output, and contains only the official deploy action.
   - Pull-request/obsolete build concurrency is cancellable, while `main` runs and
     the `pages-production` deployment group use non-interrupting active-deployment
     behavior. Fork pull requests receive no write or OIDC permissions.
   - Pinned upstream releases are `actions/configure-pages` v6.0.0
     (`45bfe019...`), `actions/upload-pages-artifact` v5.0.0 (`fc324d35...`), and
     `actions/deploy-pages` v5.0.0 (`cd2ce8fc...`); their upstream action manifests
     confirm the used inputs and Node 24 runtimes.
   - `actionlint` passes all workflows; repository-wide offline `zizmor` reports no
     findings (only existing suppressed/ignored advisories); the YAML pre-commit
     hook passes. The full documentation gate reports 34 docs tests, 397 generated
     files, 185 HTML pages, and passing offline QA; spec validation reports 16
     specs and 118 requirements.
   Notes:
   - Live build/deployment results are checked immediately after this commit is
     pushed. If Pages repository settings reject deployment, Todo 104 retains the
     owner-setting and first-production verification work.

103. [x] Add documentation discoverability, package metadata, and maintainer runbooks
   Claimed by: Codex implementer (2026-07-02 15:10 PDT)
   Requirements:
   `DOCSITE-ARCH-001`, `DOCSITE-UX-001`, `DOCSITE-OPS-001`,
   `RELEASE-VERSION-001`.
   Context:
   A deployed site is useful only if users and maintainers can find and operate it.
   Repository metadata currently contains inconsistent repository ownership
   (`Cargo.toml` references `btpc-dev/btpc` while the actual remote is
   `burritothief/btpc`), and the production workflow needs a recovery procedure.
   Implementation:
   Add prominent Documentation links to `README.md` and the site header/footer.
   Correct canonical repository, homepage, documentation, issue tracker, and source
   URLs across Cargo workspace/package metadata, Python `[project.urls]`, generated
   package metadata, and any docs configuration, using
   `https://github.com/burritothief/btpc` and the GitHub Pages project URL. Preserve
   one canonical URL definition where tooling permits and add tests preventing
   owner/path drift. Ensure MkDocs emits canonical page URLs, sitemap metadata,
   repository/edit links, license links, and a useful page description; do not
   claim docs.rs or package-index pages before they exist.

   Add a concise documentation operations section to `CONTRIBUTING.md` covering
   prerequisites, local preview, strict build, generated-source ownership, Pages
   configuration, manual dispatch, deployment status, common project-subpath/404
   failures, artifact inspection, and rollback by redeploying a known-good commit.
   Document the one-time GitHub Settings steps: Pages source `GitHub Actions`, the
   `github-pages` environment, and default-branch-only deployment protection. Add a
   release checklist item to verify documentation links and clearly distinguish
   `main` docs from future versioned release docs.
   Tests and verification:
   Add failing metadata consistency tests first. Run Cargo metadata checks, Python
   package metadata/build smoke tests, the strict docs gate, link checks, README
   command/example tests, and spec validation. Search the repository for stale
   `btpc-dev/btpc`, placeholder documentation URLs, localhost production links, and
   unsupported docs.rs claims. Confirm generated pages include the canonical Pages
   URL, sitemap, edit links, license link, and development-docs label.
   Evidence:
   - Added `tests/docs/test_documentation_metadata.py` first; its initial run
     reported 3 failures for stale Cargo ownership, absent README/runbook links,
     and missing generated edit/license expectations. The retained suite now
     reports 3 passed and validates Cargo/Python metadata, README and site
     discoverability, operations/release guidance, canonical/sitemap output, edit
     links, and license links.
   - Corrected Cargo workspace and all three package metadata fields to
     `https://github.com/burritothief/btpc` plus the canonical Pages homepage and
     documentation URL. `cargo metadata --no-deps --format-version 1` confirms the
     same repository, homepage, and documentation values for every package.
   - Added canonical Python `Project-URL` entries for Documentation, Homepage,
     Issues, Repository, and Source. A CPython 3.14 release wheel and source
     distribution built with Maturin; both generated METADATA/PKG-INFO files contain
     all five expected URLs.
   - Added prominent README Documentation/Source/Issues links, a Documentation
     footer link, edit-page controls, and a documentation release checklist. The
     complete generated site contains canonical URLs, `sitemap.xml`, repository
     edit links, license links, and the current-`main` development label.
   - Added documentation operations guidance covering locked prerequisites, local
     preview and strict validation, generated-source ownership, Pages source and
     `github-pages` environment settings, manual `workflow_dispatch`, deployment
     inspection, project-subpath/404 debugging, artifact inspection, and rollback
     through a known-good commit without a `gh-pages` branch or secret.
   - Corrected stale `btpc-dev/btpc` release links in `CHANGELOG.md`; regression
     tests and repository search find no stale owner URLs outside historical todo
     context, no localhost production URLs, and no unsupported live btpc docs.rs
     claim.
   - The README CLI tour test passes for every mode, Python docstring tests report
     6 passed, and `make docs-check` reports 37 docs tests, 398 files, 186 HTML
     pages, 12,496,194 uncompressed bytes, and 3,233,654 normalized gzip bytes.
     Link validation covers 92 Markdown files and spec validation reports 16 specs
     and 118 requirements.
   Notes:
   - The first live workflow run built the site successfully but GitHub Pages was
     not enabled; Todo 104 owns that one-time repository setting and production
     verification.

104. [x] Enable GitHub Pages and verify the first production deployment end to end
   Claimed by: Codex implementer (2026-07-02 16:03 PDT)
   Requirements:
   `DOCSITE-DEPLOY-001`, `DOCSITE-OPS-001`, `DOCSITE-QUALITY-001`.
   Context:
   Workflow code alone does not create a usable site. GitHub must use Actions as the
   Pages source, the protected environment must permit only trusted deployments,
   and the real project-subpath site must be tested over HTTPS. This is the only
   todo that may require repository-admin authorization; it must not guess around a
   missing permission.
   Implementation:
   Using the authenticated GitHub CLI/API when authorized, inspect the repository's
   current Pages and environment settings. Configure Pages with `build_type=workflow`
   if it is not already enabled. Create or update the `github-pages` environment so
   only `main` may deploy, without adding unnecessary human approval that would
   prevent automatic publishing after every push. Trigger the workflow from a known
   commit, wait for build and deployment completion, and capture the deployment URL
   from the official action output. Do not create or push a `gh-pages` branch.

   Verify `https://burritothief.github.io/btpc/` and key Getting Started, CLI,
   Python API, and `rust/btpc_core/` entry points over HTTPS. Check status codes,
   content types, canonical URLs, CSS/JavaScript/images, search index, sitemap,
   custom 404 behavior, navigation, project-subpath links, and absence of mixed
   content. Confirm a harmless documentation-only push causes an automatic rebuild
   and that a pull request builds but cannot deploy. If admin/API permission is
   unavailable, record the exact required Settings action and response in `Notes:`;
   do not claim completion until an owner performs it and the live checks pass.
   Tests and verification:
   Record the successful Actions run and Pages deployment URLs, deployed commit SHA,
   UTC/Pacific deployment time, and HTTP smoke results for all required entry
   points. Compare a local build from the deployed SHA with the artifact structure,
   run the canonical docs gate locally, and verify repository/environment settings
   through the GitHub API. Confirm there is no `gh-pages` branch and no deployment
   token or secret was introduced.
   Evidence:
   - Enabled the repository Pages site through the authenticated REST API with
     `build_type=workflow`; `GET /repos/burritothief/btpc/pages` reports the
     canonical `https://burritothief.github.io/btpc/` URL, public visibility, and
     enforced HTTPS. Created the `github-pages` environment with custom deployment
     branch policies and verified its sole policy is the `main` branch, with no
     required reviewer or wait timer.
   - Manual workflow run `28627062606` successfully built and deployed commit
     `37432d8051bdf7a62ee0273119f9b6b040937281` after a clean Pages
     reprovision. The official deploy action reported the production URL and a
     successful Pages deployment.
   - The production smoke audit found that nested custom-404 requests loaded the
     correct content but resolved runtime assets relative to the missing path.
     Added a failing regression assertion in
     `tests/docs/test_information_architecture.py`, made `404.html` references
     explicitly `/btpc/`-rooted in `scripts/build_docs_site.py`, and retained the
     fix in commit `b78ed2872cbda8b2c7fb229d84aba4a633c2a69a`.
   - Pushing that documentation-only fix automatically triggered Actions run
     `28808981236`; both build and deploy jobs succeeded. The deployment completed
     at 2026-07-06 17:05:00 UTC / 2026-07-06 10:05:00 PDT, and
     `GET /pages/deployments/b78ed2872cbda8b2c7fb229d84aba4a633c2a69a`
     reports `succeed`.
   - HTTPS checks returned 200 with expected content types and unique markers for
     the homepage, Getting Started, CLI guide, Python guide, embedded
     `rust/btpc_core/`, search index, sitemap, and local CSS. A nested absent path
     returned the custom 404 with status 404; every referenced CSS, JavaScript,
     image, favicon, and rustdoc asset returned 200. HTML canonical URLs use the
     project Pages root and no checked page contained mixed-content references.
   - `make docs-check` reports 37 documentation tests and a valid 398-file,
     186-HTML-page site. SHA-256 manifests for all 398 files in the local build and
     downloaded `github-pages` artifact from run `28808981236` match exactly.
   - Pull-request run `28807684227` for PR 2 built successfully while its
     `Documentation / deploy` job was skipped. API and remote-ref checks confirm
     there is no `gh-pages` branch, no Actions secret, and no deployment token or
     repository secret was introduced.
   Notes:
   - The first deployment immediately after initial Pages provisioning failed in
     GitHub's Pages backend despite a valid uploaded artifact. Deleting and
     recreating the not-yet-live Pages site once, then dispatching the same known
     commit, provisioned it successfully; both subsequent deployments succeeded.

105. [x] Add scheduled external-link and live-site health monitoring
   Claimed by: Codex implementer (2026-07-06 10:12 PDT)
   Requirements:
   `DOCSITE-OPS-001`, `DOCSITE-QUALITY-001`, `SEC-DEPS-001`.
   Context:
   Pull-request checks intentionally validate the generated site offline so network
   failures do not make merges flaky. Production still needs recurring visibility
   into external link rot and a broken or missing GitHub Pages deployment.
   Implementation:
   Extend the existing scheduled maintenance workflow or add a narrowly scoped
   documentation-health workflow. Use a maintained pinned link checker to validate
   external links from source/generated documentation with retry, timeout, rate
   limit, and allowlist settings that distinguish transient failures from intentional
   exclusions. Do not allow blanket `ignore all 4xx` rules. Check the live HTTPS
   homepage and key CLI, Python, Rust, sitemap, asset, and 404 paths. Validate a
   unique expected marker on each page so a generic GitHub error page cannot pass.

   Keep default permissions read-only, pin actions and installed tools, set explicit
   job timeouts and concurrency, and write actionable summaries naming broken URLs
   and source pages without exposing secrets. Run weekly and on manual dispatch.
   Do not add an auto-commit bot or grant issue-writing permissions solely for this
   check; failed Actions runs are the initial alert channel. Document how maintainers
   reproduce and triage failures locally, including temporary network failures and
   deliberate URL migrations.
   Tests and verification:
   Add structural workflow tests and deterministic fixtures proving the checker
   catches a broken external URL, a GitHub 404 page returning HTML, a missing page
   marker, and mixed content while respecting narrow documented exclusions. Run
   action-pin checks, `zizmor`, YAML validation, the maintenance command locally
   against fixtures, and one authorized live manual workflow dispatch. Record the
   successful run URL and exact URLs checked. Run the canonical docs gate and spec
   validation.
   Evidence:
   - Added `tests/docs/test_documentation_health.py` and response fixtures first;
     the initial run reported five failures for the absent workflow policy,
     Lychee configuration, live manifest, and validator. The completed focused
     suite reports 6 passed and covers a broken 503 response, generic GitHub Pages
     error HTML returned with status 200, a missing marker, mixed content, a healthy
     response, narrow exclusions, and deterministic source/generated URL inventory.
   - Extended the weekly/manual Repository maintenance workflow with Rust 1.94.1,
     exact Lychee 0.24.2 installation, the reviewed Linux archive SHA-256
     `1f4e0ef7f6554a6ed33dd7ac144fb2e1bbed98598e7af973042fc5cd43951c9a`,
     a 20-minute timeout, read-only repository permission, and the existing bounded
     concurrency. All referenced actions remain pinned to 40-character commits;
     no issue-writing, auto-commit, or deployment permission was added.
   - Added `.lychee.toml` with three retries, 20-second request timeout, two-second
     retry wait, eight global/two per-host requests, 250 ms host spacing, HTTPS
     enforcement, full fragments, and only documented exact/domain-purpose
     exclusions. It accepts only 200-399 responses and does not ignore a 4xx class,
     timeouts, or TLS failures.
   - Added a deterministic source-attributed inventory over documentation Markdown
     and generated non-rustdoc HTML. The live local run found 147 unique URLs;
     Lychee checked 16 external URLs successfully and classified 131 intentional
     local/example/Pages/edit/pre-release URLs through the reviewed exclusions with
     zero errors. Failure summaries include the broken URL and expose the inventory
     mapping back to every originating source page.
   - The exact external URLs checked were the GitHub Pages custom-workflow guide;
     the BTPC repository, issues, license, benchmarks, Rust consumer test, release
     specification, interoperability fixture, core source, and specs tree; plus the
     mdBook overview, CI, preprocessor, general configuration, renderer, and summary
     documentation URLs.
   - The live manifest checked the exact production homepage,
     `/getting-started/`, `/cli/`, `/python/`, `/rust/btpc_core/`,
     `/search/search_index.json`, `/sitemap.xml`, `/stylesheets/extra.css`, and
     nested `/health-check/missing/nested/` custom-404 URL. All returned the exact
     expected status, content type, page-specific marker, final HTTPS URL, and no
     mixed content; the custom path correctly returned status 404 rather than a
     generic GitHub error page.
   - `make docs-check` reports 43 documentation tests, 398 files, 186 HTML pages,
     12,496,616 uncompressed bytes, 3,233,623 normalized gzip bytes, and 92 checked
     Markdown files. Ruff, formatting, Actionlint, offline Zizmor, YAML parsing,
     `git diff --check`, and spec validation (16 specs, 120 requirements) pass.
   - Authorized manual workflow run `28828571228` at commit
     `d14210ac71ed547f17d19296adac4f1c6748f41d` succeeded in 1m23s. Its log records
     the canonical site gate, 147-URL inventory, all nine live PASS results, and the
     always-published Actions summary.
   Notes:
   - The repository pre-push advisory feed currently rejects unrelated
     `crossbeam-epoch 0.9.18` as RUSTSEC-2026-0204. Todo-specific docs, workflow,
     and live checks passed; pushes used the documented hook skip for that external
     advisory rather than mixing a dependency update into this todo.

106. [x] [Review] Finish descriptor-relative safe payload verification
   Claimed by: Codex implementer (2026-07-06 15:55 PDT)
   Context:
   Todo 77 was marked complete with a documented portable fallback, but the current
   verifier still performs `symlink_metadata` checks and then snapshots/opens the
   same pathname. If a checked component is replaced by a symlink between those
   operations, `file_id::get_file_id`, `File::open`, and the later path-identity
   checks all follow and consistently validate the new outside-root target. The
   same check-then-`read_dir` race exists in extra-file traversal. This can hash or
   enumerate outside the selected payload root rather than merely reporting a
   concurrent mutation.
   Implementation:
   Use descriptor-relative, no-follow traversal rooted at an opened payload
   directory on supported platforms. Open each component beneath that descriptor,
   reject symlinks/reparse points at the kernel boundary, and hash the final opened
   file handle without resolving the original pathname again. Use maintained
   platform abstractions where they provide the required semantics; otherwise add
   narrow OS modules for Unix `openat`/`openat2`-style traversal and Windows handle/
   reparse-point checks while keeping unsafe code out of `btpc-core` itself. Extra-
   file enumeration must walk opened directory handles under the same root. If a
   platform cannot provide the guarantee, expose and document an explicit weaker
   verification policy instead of calling it safe by default.
   Tests and verification:
   Add deterministic synchronization hooks that replace an intermediate directory
   or final file with an outside-root symlink after the initial check but before
   open/enumeration. Prove v1, v2, hybrid, and extra-file modes never read or report
   outside-root contents and never return a valid report. Run repeated stress tests
   on Linux, macOS, and Windows, including junction/reparse-point cases, plus all
   existing verification, CLI, and Python tests.
   Evidence:
   - Added `crates/btpc-core/src/verify/safe_fs.rs` using `cap-std` and
     `cap-fs-ext` capabilities: payload roots are opened once, every descendant
     component is opened descriptor-relative with no-follow semantics, expected
     files are hashed through retained handles, path identity is revalidated, and
     extra-file enumeration recurses through opened directory handles.
   - Added deterministic `BeforeExpectedOpen`, `AfterStructure`, and
     `BeforeExtraOpen` test hooks. `verify::race_tests` covers final-file swaps in
     v1/v2/hybrid mode, 16 repeated intermediate-directory swaps per mode on Unix,
     16 repeated Windows junction swaps per mode, and extra traversal without ever
     reporting the outside `nested/secret` path.
   - Local verification passed: `cargo fmt --all --check`; strict workspace
     Clippy; `cargo nextest run --workspace --all-features` (224/224);
     `cargo test --workspace --doc`; Ruff check/format; Pyrefly/Pyright checks;
     `uv run pytest tests/python` (71/71); `make docs-fast` (14 site tests and 92
     Markdown files); `uv run python scripts/check_specs.py` (16 specs, 120
     requirements); focused core, CLI verify, and Python verify tests all passed.
   - CI run `28829603835` executed
     `cargo test -p btpc-core --lib verify::race_tests --locked` successfully on
     `ubuntu-latest`, `macos-latest`, and `windows-latest`, proving the repeated
     junction/reparse regression on Windows as well as the Unix symlink races.
   - `cargo check -p btpc-core --tests --target x86_64-pc-windows-gnu` and the
     locked external Rust consumer check passed after refreshing
     `tests/rust-consumer/Cargo.lock`.
   Notes:
   - Full CI still reports the pre-existing `RUSTSEC-2026-0204` advisory in the
     Criterion-only Crossbeam dependency. A Windows broad-workspace run can also
     fail later in the unrelated CLI reference test when Windows keeps its output
     temp directory open; the focused verifier race step completes successfully
     before that test. Pushes used the documented `pre-push-gate` skip rather than
     mixing either unrelated issue into this verification-security todo.

107. [x] [Review] Preserve raw info identity for top-level edits and update hybrid attributes atomically
   Claimed by: Codex implementer (2026-07-06 16:12 PDT)
   Context:
   `MetainfoEditor::from_metainfo` converts the complete original tree to canonical
   `OwnedValue`, so a top-level-only edit canonicalizes a noncanonical `info`
   dictionary and changes its info hash. This violates the documented contract that
   tracker/comment/creator/date edits retain info hashes. Reproduction: editing only
   `comment` changed v1 hash `8e258a24...` to `1cd7ccb6...` for a valid torrent with
   unsorted source `info` keys. Separately, `file_attributes` returns immediately
   after updating the v1 `files` entry in a hybrid torrent and never updates the v2
   `file tree`; a reproduced hybrid edit emitted exactly one `attr=x` occurrence.
   Implementation:
   Represent editor state as a raw preserved `info` slice plus owned top-level
   fields until an info-level mutation occurs. Top-level-only serialization must
   embed the exact original `info` bytes while canonically writing the surrounding
   dictionary, or provide distinct explicit preserve-info and canonicalize-all
   operations whose default honors the hash-stability contract. Once `info` changes,
   canonicalize and recompute every applicable hash. For hybrid real files,
   `file_attributes` must update both v1 and v2 representations transactionally;
   padding-only attributes remain v1-only. Reject the operation without mutation if
   either representation is absent or inconsistent.
   Tests and verification:
   Add noncanonical v1/v2/hybrid fixtures proving every top-level set/remove edit
   preserves exact info bytes and hashes. Add hybrid single/multi-file tests proving
   real-file attributes update both representations, padding edits affect only the
   permitted entry, and injected failure cannot leave one side changed. Cover Rust,
   CLI `edit`, Python `Metainfo.edit`, original/canonical output choices, and unknown
   top-level-field preservation.
   Evidence:
   - Refactored `MetainfoEditor` into owned canonical top-level fields plus an
     `EditorInfo::Raw` exact source slice that is converted to owned form only by
     an info-level mutation. `to_metainfo` now writes the surrounding dictionary
     in unsigned-byte key order while embedding untouched `info` bytes verbatim;
     `Metainfo::to_bytes` remains the explicit canonicalize-all operation.
   - Reworked file-attribute editing around a cloned candidate dictionary and a
     single commit point. Hybrid real files must update matching v1 and v2 paths,
     inconsistent representations return an error, and v1 padding entries update
     without touching the v2 tree.
   - `crates/btpc-core/tests/edit.rs` now generates noncanonical v1, v2, and hybrid
     fixtures and proves all top-level set/remove operations preserve exact info
     bytes and hashes, unknown top-level fields survive, explicit canonical output
     changes the noncanonical identity, and info edits canonicalize/recompute every
     applicable hash. Hybrid single/multifile and padding cases pass.
   - The private `edit::tests::injected_hybrid_attribute_failure_leaves_info_unchanged`
     test forces a failure after the v1 candidate update and proves the original
     dictionary is unchanged.
   - CLI tests `top_level_cli_edit_preserves_noncanonical_info_bytes` and
     `hybrid_cli_file_attributes_update_both_representations`, plus Python tests
     `test_python_top_level_edit_preserves_noncanonical_info_bytes` and
     `test_python_hybrid_attributes_update_both_representations`, pass. Python
     docstrings and editing guides document preserved raw info versus explicit
     canonical output.
   - Full verification passed: `cargo fmt --all --check`; strict workspace Clippy;
     `cargo nextest run --workspace --all-features` (231/231);
     `cargo test --workspace --doc`; Ruff check/format; Python type checks;
     `uv run pytest tests/python` (73/73); `make docs-fast` (14 site tests and 92
     Markdown files); and the spec registry (16 specs, 120 requirements).
   Notes:
   - The repository-wide dependency policy still has the previously documented
     Criterion-only `RUSTSEC-2026-0204` advisory, so pushes use the documented
     `pre-push-gate` skip rather than mixing an unrelated dependency update into
     this editor-correctness todo.

108. [x] [Review] Validate and expose all recognized optional metainfo fields consistently
   Claimed by: Codex implementer (2026-07-06 16:25 PDT)
   Context:
   The validated owned `Metainfo` parses trackers and web seeds but does not retain
   or expose DHT `nodes`; Python also omits `source`, `comment`, `created_by`,
   `creation_date`, and nodes inspection properties. Several recognized fields are
   silently ignored when their type is wrong: malformed comment/creator/source/date
   values can pass validation and then appear absent. An empty `announce-list`
   overrides a valid `announce` and produces no trackers, and parsed empty tracker
   URLs currently validate. Creation rejects empty tracker/web-seed values but
   accepts DHT port 0, so parse, create, edit, CLI, and Python policies disagree.
   Implementation:
   Define one typed optional-metadata model in `btpc-core` for tracker tiers, web
   seeds, DHT nodes, comment, creator, creation date, and source. Validate recognized
   field shapes and domains centrally and reuse the same rules in parsing, creation,
   and editing. Decide and document strict-error versus compatibility-warning policy
   for empty tiers/URLs, `announce` fallback when `announce-list` is empty, malformed
   optional text, negative/out-of-range dates, empty node hosts, and port 0. Expose
   lossless owned accessors in Rust and immutable Python properties, and make CLI
   inspect consume the same typed model rather than reparsing raw fields separately.
   Tests and verification:
   Add a cross-surface table of valid, warning, and rejected forms for every field,
   including non-UTF-8 bytes, empty lists/tiers/strings, duplicate values, malformed
   node pairs, ports 0/1/65535/out of range, arbitrary-precision dates, and conflicts
   between `announce` and `announce-list`. Assert Rust, CLI JSON/human, and Python
   return identical values/warnings/errors and creation never emits a form the
   parser would reject or warn about unexpectedly.
   Evidence:
   - Added the byte-lossless `OptionalMetadata`/`DhtNode` core model and shared
     tracker, web-seed, node, and creation-date validators. Parsing, creation, and
     editing now reject the same malformed domains; empty `announce-list` falls
     back to `announce` with a warning, while empty `url-list` remains a clean
     no-web-seed value.
   - `cargo test -p btpc-core --test optional_metadata --test edit` passed 17/17,
     including non-UTF-8 values, duplicates, empty lists/tiers/URLs, conflicting
     announce fields, malformed node pairs, ports 0/1/65535/out of range,
     negative/arbitrary-precision dates, warning behavior, and edit parity.
   - `cargo test -p btpc-cli --test inspect` passed 14/14; human, JSON, and field
     projection inspection consume the typed model and agree on nodes and scalar
     optional metadata. `uv run pytest tests/python/test_metainfo.py
     tests/python/test_edit.py` passed 21/21 for immutable lossless properties,
     warnings, errors, and creation/edit domains.
   - `scripts/check_native_stub.py`, `scripts/check_python_types.sh`, and an
     installed release wheel checked from a temporary directory with Pyright and
     the external consumer all passed; the wheel consumer also executed against
     the installed package. The wheel/sdist typing artifact test passed 1/1.
   - Full gates passed: formatting and strict workspace Clippy; `cargo nextest`
     238/238; workspace doctests; rustdoc; Ruff check/format; Python typing;
     `uv run pytest tests/python` 75/75; `make docs-check` 43 tests plus a validated
     403-file site within size budgets; and the spec registry (16 specs, 120
     requirements).
   Notes:
   - `cargo deny check` remains blocked only by the previously documented
     Criterion-only `RUSTSEC-2026-0204` advisory for `crossbeam-epoch 0.9.18`;
     licenses, sources, and bans pass. This unrelated dependency update is not
     mixed into the optional-metadata change.

109. [x] [Review] Expose unknown bencode values, not only their keys, in Python
   Claimed by: Codex implementer (2026-07-06 19:02 PDT)
   Context:
   Rust `UnknownField` retains both key and owned bencode value, but Python
   `Metainfo.unknown_fields` returns only `tuple[bytes, ...]`. Python callers cannot
   inspect an extension value, distinguish two torrents with the same extension
   keys, or perform a lossless read-modify-write operation. The Python raw editor is
   additionally restricted to integer or byte-string values, despite the stated
   goal of preserving arbitrary unknown dictionaries and lists.
   Implementation:
   Add a public immutable Python bencode value model or a precise recursive type
   using `int`, `bytes`, tuples, and a deterministic immutable mapping/pair sequence.
   Preserve arbitrary-precision integers and raw byte keys without UTF-8 coercion.
   Expose unknown fields as ordered key/value objects or a mapping with duplicate
   behavior explicitly impossible after validation, plus an accessor for exact raw
   encoded bytes/span when source identity matters. Accept the same recursive value
   model in raw extension editing while continuing to reject reserved keys. Keep
   conversions lazy and cache them on the native object.
   Tests and verification:
   Add nested list/dictionary, arbitrary integer, non-UTF-8 key/value, empty
   container, equality/repr/typing, lazy-cache, and parse-edit-serialize tests.
   Verify a Python caller can read an unknown value and write it unchanged without
   semantic loss, and that Rust/Python canonical bytes agree. Run Pyrefly, Pyright,
   stub checks, pytest, and installed-wheel API tests.
   Evidence:
   - Added frozen public `BencodeList`, `BencodeDictionary`, and `UnknownField`
     values plus the recursive `BencodeValue` type. Unknown fields now expose raw
     keys, nested values, exact encoded key/value bytes, and half-open source spans;
     native and public results are lazily cached.
   - PyO3 conversion preserves arbitrary-size Python integers, non-UTF-8 bytes and
     dictionary keys, empty/nested lists and dictionaries, and canonical raw-key
     ordering. Raw extension editing accepts the same recursive model, rejects
     booleans/non-byte dictionary keys, and still delegates reserved-key rejection
     to the core editor.
   - `uv run pytest tests/python` passed 77/77, including equality/repr, cache
     identity, nested and empty containers, arbitrary integers, exact spans/bytes,
     read-edit-write semantic preservation, and Rust/Python canonical byte parity.
     `cargo test -p btpc-core --test public_api` passed 11/11 for the new exact
     unknown-field source-byte accessor.
   - Pyrefly, Pyright, Ruff, strict workspace Clippy, native stub/runtime parity,
     rustdoc, workspace doctests, and `cargo nextest run --workspace --all-features`
     (238/238) passed. The spec registry validated 16 specs and 120 requirements.
   - A release wheel installed into a temporary environment outside the checkout
     passed the external typed consumer, recursive unknown-value runtime round trip,
     and the wheel/sdist typing-artifact test. `make docs-check` passed 43 tests and
     validated the 404-file generated site within size budgets.
   Notes:
   - `cargo deny check` remains blocked only by the existing Criterion-only
     `RUSTSEC-2026-0204` advisory for `crossbeam-epoch 0.9.18`; bans, licenses, and
     sources pass, so the unrelated dependency update remains separate.

110. [x] [Review] Complete lossless filesystem-path schemas on every supported platform
   Claimed by: Codex implementer (2026-07-06 19:35 PDT)
   Context:
   Todo 78 added exact Unix path objects but explicitly left Windows using a lossy
   UTF-8 fallback. Several CLI structures also continue to carry a legacy lossy
   `output`/`path` string beside the exact object, and not every batch, config, or
   diagnostic machine surface uses `FilesystemPathJson`. A public cross-platform
   library should not call a representation exact when Windows path identity can be
   lost or when new schemas encourage consumers to keep using the lossy field.
   Implementation:
   Encode Windows paths losslessly as UTF-16 code units (or an equally precise
   documented representation) and Unix paths as bytes, both behind one versioned
   schema containing a safe display string. Inventory every JSON/TSV/plain structured
   path field across create, edit, verify, batch, config, completion installation,
   diagnostics, and benchmark outputs. Make the exact object canonical in the next
   schema version; retain lossy strings only as explicitly deprecated display fields
   with a removal plan. Python native errors should construct platform-native
   `Path` objects without routing Windows paths through UTF-8 bytes.
   Tests and verification:
   Add Windows-hosted round trips for non-ASCII UTF-16 and edge-case path units, Unix
   colliding lossy-decoding names, every command schema, backward-compatible parsing,
   and human escaping of control characters. Search production adapter code for
   unreviewed `to_string_lossy` uses and require each remaining use to be display-
   only with a targeted test.
   Evidence:
   - Added `btpc.filesystem-path.v2`: Unix paths use exact hexadecimal native
     bytes, Windows paths use exact UTF-16 code units, and both carry a safely
     escaped display value. Create, edit, verify, batch, config, completion, error,
     and benchmark surfaces now use the exact object as canonical data; schema-v1
     benchmark parsing remains supported and lossy aliases are documented as
     deprecated for removal in v3.
   - Plain path output preserves Unix native bytes and emits self-describing Windows
     UTF-16 units. Inferred output names append to `OsString`, human output escapes
     control characters, and remaining `to_string_lossy` calls are display-only.
     Python native errors construct platform-native `Path` values directly from
     `PathBuf` without a UTF-8 byte round trip.
   - Focused CLI path tests, benchmark tests, and Python verification tests passed,
     including Unix colliding lossy names, control-character display escaping,
     schema compatibility, and exact Python `PathError.path` identity. The generated
     CLI reference test passed 3/3 after making its temporary output and line-ending
     comparisons portable on Windows.
   - Full local gates passed: strict workspace Clippy; `cargo nextest` 244/244;
     workspace doctests and rustdoc; Ruff check/format; Pyrefly, Pyright, and native
     stub parity; `uv run pytest tests/python tests/benchmarks` 117 passed and 1
     Windows-only skip; `make docs-check` 43 tests plus a validated 404-file site;
     and spec validation for 16 specs and 120 requirements.
   - A release wheel installed outside the checkout preserved a non-UTF-8 Unix
     `PathError.path` and passed the installed typing-artifact checks. GitHub Actions
     run 28839807889 passed both `Python / Windows path identity` and the complete
     `Rust / stable / windows-latest` workspace test job.
   Notes:
   - `cargo deny check` remains blocked only by the existing Criterion-only
     `RUSTSEC-2026-0204` advisory for `crossbeam-epoch 0.9.18`; bans, licenses, and
     sources pass, so that unrelated dependency update remains separate.

111. [x] [Review] Prepare btpc-core for actual crates.io publication
   Claimed by: Codex implementer (2026-07-06 20:45 PDT)
   Context:
   BTPC is intended to be a publicly embedded Rust library, but workspace metadata
   sets `publish = false` for every crate and the release workflow publishes only
   Python artifacts and native archives. The release-candidate gate is complete even
   though Rust users can only use a path/git dependency, and `btpc-core` still has a
   hidden public ownership-transfer constructor used solely to cross the workspace
   PyO3 crate boundary.
   Implementation:
   Decide the publication boundary explicitly: make `btpc-core` publishable with
   complete crates.io metadata, packaged README/license/docs/examples, and an
   intentional dependency/MSRV set; keep adapter crates private unless there is a
   reason to publish them. Replace `#[doc(hidden)] pub
   from_owned_bytes_with_options` with a properly named supported `from_vec` API or
   a sealed workspace-internal mechanism that does not enter the public semver
   surface. Add a protected manual crates.io publish job using trusted publishing or
   a narrowly scoped token, ordered after artifact/API validation and gated on the
   existing release tag/version. Never publish automatically from ordinary pushes.
   Tests and verification:
   Run `cargo package -p btpc-core --allow-dirty`, inspect the crate contents, build
   and test the packaged crate offline as an external consumer on MSRV and stable,
   run cargo-semver-checks against the previous published/tagged baseline, and dry-
   run `cargo publish`. Verify docs.rs metadata/features and README links without
   claiming a live registry page until the owner performs the first publish.
   Evidence:
   - `btpc-core` is now the only publishable workspace member, restricted to the
     `crates-io` registry with MIT, Rust 1.85, repository/homepage/documentation,
     crate-specific README, docs.rs all-features metadata, and a compiling
     `examples/inspect.rs`; `btpc-cli` and `btpc-python` continue to inherit
     `publish = false`.
   - Replaced the hidden cross-workspace constructor with documented
     `Metainfo::from_vec` and `Metainfo::from_vec_with_options` APIs. Borrowed-byte
     and path loaders delegate to the supported owned-buffer path, and the PyO3
     adapter transfers its `Vec<u8>` through that API. Public API tests passed
     12/12, including exact original-byte retention for both new entry points.
   - Added `scripts/check_crate_package.sh`; extracted packages passed offline
     library, doctest, example, and external-consumer builds on Rust 1.85.0 and
     1.94.1. The final archive contained 54 files and included normalized metadata,
     lockfile, MIT license, README, sources, and the public example; the external
     consumer compiled against the extracted path and called `from_vec`.
   - `cargo publish -p btpc-core --locked --dry-run --allow-dirty` completed all
     packaging, verification, and upload-preflight steps and aborted only because
     it was a dry run. `cargo-semver-checks 0.42.0` is pinned in the release job;
     this repository has no previous release tag, so the existing initial-release
     policy correctly records and skips the unavailable baseline comparison.
   - The manual release workflow validates the packaged crate on MSRV and stable,
     pins every build to the supplied tag (or dispatch SHA for non-publishing dry
     runs), verifies the exact tag/version, and publishes only after Rust API,
     artifact, and provenance jobs through the protected `crates-io` environment
     with `CRATES_IO_TOKEN`. Ordinary pushes cannot invoke publication.
   - Full gates passed: formatting and strict workspace Clippy; `cargo nextest`
     245/245; workspace doctests and rustdoc; Ruff check/format; Pyrefly, Pyright,
     and native stub parity; `uv run pytest tests/python` 79 passed and 1
     Windows-only skip; `make docs-check` 43 tests plus a validated 404-file site;
     pre-commit, actionlint, and zizmor; and 16 specs with 121 requirements.
   Notes:
   - `cargo deny check` remains blocked only by the existing Criterion-only
     `RUSTSEC-2026-0204` advisory for `crossbeam-epoch 0.9.18`; bans, licenses, and
     sources pass. The first crates.io publication remains an explicit owner action,
     and documentation does not claim that a registry or docs.rs release is live.

112. [x] Freeze the current MkDocs site as the mdBook migration baseline
   Claimed by: Codex implementer (2026-07-06 20:52 PDT)
   Requirements:
   `DOCSITE-MIGRATE-001`, `DOCSITE-QUALITY-001`, `DOCSITE-OPS-001`,
   `TEST-TDD-001`.
   Context:
   Todos 96-104 produced and deployed a production Material for MkDocs site. The
   renderer is now changing, but users must not lose public pages, important API
   anchors, expected site features, or the existing GitHub Pages project URL. This
   todo records an objective baseline before any MkDocs files or dependencies are
   removed. It must consume the completed Todo 104 deployment evidence when
   available and must not modify the live deployment workflow yet.
   Implementation:
   Begin with failing tests for a checked-in renderer-migration manifest. Build the
   current MkDocs artifact from a clean tree and inventory every generated HTML
   route, canonical URL, navigation chapter, important Python API anchor, CLI
   command page/anchor, rustdoc entry point, static asset class, sitemap entry, and
   custom 404 location. Record the route and anchor compatibility set in a compact
   deterministic fixture under `tests/docs/fixtures/`; do not commit the full
   generated site or volatile hashes. Include the site root, all handwritten
   chapters, every generated CLI/Python reference page, and key public API object
   anchors. Mark which routes may become redirects and which must remain direct.

   Record the uncompressed/compressed artifact size, HTML page count, current
   documentation build duration, required search/theme/edit/copy/keyboard features,
   privacy constraints, and successful live URLs from Todo 104. Add a reusable
   comparison helper that can evaluate any future generated site against the
   baseline without assuming MkDocs internals. Capture local screenshots only for
   human comparison and do not commit them unless an existing repository policy
   explicitly calls for visual fixtures. Keep `mkdocs.yml`, dependencies, tests,
   and workflows unchanged in this todo.
   Tests and verification:
   Demonstrate the migration-manifest test failing before the fixture exists. Run
   the existing complete `make docs-check`, generate the baseline twice and prove
   byte-stable manifest ordering, validate every recorded route/anchor against the
   current artifact, and run the comparison helper against one deliberately missing
   route and anchor. If the live Pages site exists, smoke every required top-level
   route over HTTPS and record its deployed commit. Run Ruff, docs tests, link
   checks, codespell, and spec validation.
   Evidence:
   - Added the deterministic `btpc.docs-renderer-baseline.v1` fixture and a
     renderer-neutral generator/comparator. The initial contract test failed with
     the fixture and helper absent; the completed focused suite passes 2/2 and its
     synthetic failure case reports both `missing route` and `missing anchor`.
   - The fixture records all 191 generated HTML routes as direct routes, 61
     navigation pages, 64 canonical URLs, 63 sitemap routes, the root and custom
     404, four static-asset classes, every CLI/Python reference page, key CLI,
     Python, and rustdoc anchors, required search/theme/edit/copy/keyboard features,
     and the no-analytics/no-tracking/local-asset privacy contract.
   - The measured baseline contains 404 files and 191 HTML pages, totals 12,722,862
     uncompressed bytes and 3,267,519 normalized-gzip bytes, and built locally in
     2,774 ms end to end. Two independent generator runs produced byte-identical
     JSON and both matched the checked-in fixture exactly.
   - `make docs-check` now compares every recorded route, anchor, canonical URL,
     asset class, sitemap route, and custom 404 against the generated artifact. It
     passed 45 documentation tests, the 191-route comparison, link validation for
     92 Markdown files, codespell, rustdoc, and the existing artifact budgets.
   - HTTPS smoke checks returned 200 for the homepage, Getting Started, CLI,
     Python, embedded rustdoc, search index, sitemap, and local CSS; a nested absent
     route returned the expected custom 404. The latest completed deployment at
     smoke time was workflow run 28839992593 for commit
     `980688188deef21702ee75e6659f1a60ce202f67`, completed at
     2026-07-07 03:50:01 UTC.
   - Ruff check/format and specification validation passed; the registry remains at
     16 specs and 121 requirements. `mkdocs.yml`, dependency metadata, and the live
     Pages workflow were not changed, and no generated site or screenshot artifact
     was committed.
   Notes:

113. [x] Bootstrap pinned mdBook beside the working MkDocs build
   Claimed by: Codex implementer (2026-07-06 20:56 PDT)
   Requirements:
   `DOCSITE-ARCH-002`, `DOCSITE-BUILD-001`, `DOCSITE-UX-001`,
   `TEST-TDD-001`.
   Context:
   The migration must remain bisectable and must not replace a functioning site
   before mdBook can render the complete chapter tree. mdBook 0.5.3 was current when
   `DOCUMENTATION_PLAN.md` was approved, and its Rust requirement is newer than
   BTPC's crate MSRV. Treat it as an exact external docs tool, not a workspace
   dependency. This todo establishes a side-by-side mdBook build only; MkDocs remains
   the canonical Pages artifact until later todos reach parity.
   Implementation:
   Write failing tests for `book.toml`, exact tool-version validation,
   `build.create-missing = false`, Rust edition 2024, `/btpc/` `site-url`, local
   search, repository/edit links, custom 404 input, and a complete
   `docs/SUMMARY.md`. Add `book.toml` using `docs` as the source directory and add a
   strict summary that represents the existing information architecture and every
   generated CLI/Python reference chapter exactly once. Configure mdBook's built-in
   HTML/search/theme capabilities, local additional CSS, no remote runtime assets,
   and extra watch paths for authoritative Python/Rust/CLI documentation inputs.

   Add an exact mdBook version constant or tool manifest consumed by local scripts
   and CI. The local command must fail clearly when mdBook is missing or has the
   wrong version and print the exact `cargo install ... --locked` remediation. Do
   not add mdBook to the workspace dependency graph or change the crate MSRV. Add a
   temporary explicit side-by-side build command such as `make docs-mdbook-site`
   that writes only to `.tmp/` or another ignored path and does not alter
   `make docs-site`, `make docs-check`, or `.github/workflows/docs.yml` yet. The
   side-by-side build should invoke mdBook directly, not shell out through MkDocs.
   Tests and verification:
   Preserve initial test failures. Verify the exact pinned mdBook release and its
   checksum/installer provenance, run `mdbook build` from the repository root and a
   different current directory, and prove a missing `SUMMARY.md` chapter fails
   without creating a file. Serve the book locally under `/btpc/` and request the
   homepage, search index, CSS, and 404 page. Confirm `cargo tree -p btpc-core` and
   MSRV checks do not include mdBook. Run focused docs tests, Ruff, spec validation,
   and the unchanged MkDocs `make docs-check` to prove side-by-side safety.
   Evidence:
   - Added failing bootstrap tests first; all four initially failed for the absent
     config, summary, version checker, and Make target. The completed focused suite
     passes 4/4 and checks exact version/checksum metadata, strict config, complete
     chapter coverage, actionable missing/wrong-tool errors, and temporary output.
   - Added `book.toml` with `docs` as the source, `create-missing = false`, Rust
     edition 2024, `/btpc/` site URL, local search with heading split level 3,
     repository/edit links, custom `404.md`, local CSS, light/dark themes, and watch
     paths for Python, CLI, and Rust documentation sources. `docs/SUMMARY.md` lists
     all 62 public handwritten and generated CLI/Python chapters exactly once.
   - Pinned mdBook 0.5.3 outside the workspace. The locked crates.io source archive
     SHA-256 is
     `742264af649df2323b283a4c1a8abc21b6f6880cf030d642500ef85c2ce81598`,
     recorded with contributor installation guidance for
     `cargo install mdbook --version 0.5.3 --locked`; the checker reports that exact
     remediation for missing or mismatched binaries.
   - `make docs-mdbook-site` invokes mdBook directly and writes only to
     `.tmp/mdbook-site`. Builds succeeded from the repository root and from `/tmp`;
     a summary containing a missing chapter failed with exit 101 and did not create
     the referenced file.
   - Serving the generated book from a local `/btpc/` mount returned 200 for the
     homepage, hashed search index, `stylesheets/mdbook.css`, and `404.html`.
     Inspection found no remote script or stylesheet runtime URLs.
   - `cargo tree -p btpc-core` contains no mdBook dependency, Rust 1.85
     `cargo check -p btpc-core --locked` passed, and Cargo/uv locks, Rust toolchain,
     `mkdocs.yml`, and the Pages workflow are unchanged. Ruff, spec validation, and
     the canonical MkDocs `make docs-check` passed 49 docs tests, route-baseline
     comparison, links, codespell, rustdoc, and artifact budgets.
   Notes:

114. [x] Port all handwritten content, navigation, theme behavior, and public routes
   Claimed by: Codex implementer (2026-07-06 22:46 PDT)
   Requirements:
   `DOCSITE-ARCH-002`, `DOCSITE-UX-001`, `DOCSITE-MIGRATE-001`,
   `DOCSITE-BUILD-001`, `TEST-TDD-001`.
   Context:
   Existing Markdown contains Material/PyMdown conventions, MkDocs directory-style
   URL assumptions, template overrides, and CSS selectors that mdBook does not
   share. This todo makes the human-authored book complete and usable while the
   Python API generator and combined Rust artifact are still handled in later
   todos. Preserve meaning and examples; do not rewrite the documentation merely to
   sound different.
   Implementation:
   Start with failing tests that compare `SUMMARY.md` coverage, required page titles,
   hierarchy, feature markers, and baseline routes. Convert Material-only
   admonitions, tabs, attribute-list syntax, template macros, and link patterns to
   mdBook-supported Markdown or minimal semantic HTML. Keep one H1 per chapter and
   ensure chapter-relative links work both in source validation and rendered HTML.
   Add explicit stable anchors where current public links depend on MkDocs slug
   behavior. Remove no substantive installation, CLI, Python, Rust, protocol,
   performance, compatibility, security, or contributor guidance.

   Replace the Material override and CSS behavior with surgical mdBook additional
   CSS that preserves the development-docs notice, responsive layout, visible focus,
   reduced-motion policy, readable tables/admonitions, code presentation, and local
   light/dark themes. Do not fork the complete upstream mdBook theme unless a
   separately documented, tested requirement cannot be met otherwise. Add a route
   compatibility generator driven by Todo 112's manifest. It must create relative,
   `/btpc/`-safe, loop-free redirect pages for old MkDocs directory URLs that mdBook
   does not render directly, while leaving root, 404, and high-value direct routes
   intact. Preserve important existing fragment IDs with explicit anchors or tested
   fragment redirects.
   Tests and verification:
   Retain initial failures, then run the side-by-side mdBook build and baseline
   comparison. Validate all handwritten chapters are reachable exactly once through
   `SUMMARY.md`, all internal source/rendered links and anchors resolve, every old
   public route is direct or redirects once to a valid canonical page, and no
   redirect escapes `/btpc/` or loops. Serve under the project subpath and manually
   inspect narrow/wide layouts, keyboard navigation, search, theme switching, code
   copy, print, development notice, and 404 behavior. Run codespell, offline site
   validation, focused docs tests, Ruff, spec validation, and the unchanged MkDocs
   production gate.
   Evidence:
   - Added failing content-port tests first; they exposed literal Material
     admonitions, leaked YAML titles, absent development-banner markup, and missing
     MkDocs directory routes. The completed focused suite passes 4/4 for
     renderer-neutral Markdown, feature markers, source/rendered links, stable
     fragments, canonicals, and loop-free one-hop redirects.
   - Converted the three Material-only admonitions to semantic blockquotes while
     preserving all substantive installation, CLI, Python, Rust, protocol,
     performance, compatibility, security, and contributor text. The temporary
     mdBook source-copy step strips MkDocs title front matter without changing the
     production source or generated CLI reference contract.
   - Added a surgical mdBook postprocessor and local CSS. Every rendered chapter
     receives the development-documentation notice and canonical metadata;
     responsive/focus/reduced-motion/table/admonition/print styling remains local,
     and built-in keyboard navigation, search, light/dark themes, code-copy, and
     print markers are present in generated HTML.
   - The route generator consumes Todo 112's baseline and produced 55 unique,
     `/btpc/`-rooted redirect pages. All non-rust baseline routes are direct or
     redirect once to an existing non-redirect target; root and custom 404 remain
     direct, canonicals match the baseline, no redirect escapes or loops, and key
     CLI fragment IDs survive at the target.
   - The side-by-side artifact contains 191 files and 121 HTML pages. All
     handwritten and CLI rendered links/anchors resolve; Python API fragment
     generation and embedded rustdoc are intentionally deferred to Todos 115 and
     116 and are excluded only from this migration-stage assertion.
   - Local `/btpc/` HTTP smoke returned 200 for the homepage, search index, local
     CSS, and 404 page. The in-app browser surface reported no available browser,
     so responsive/interactive inspection was verified through generated ARIA,
     keyboard, theme, search, copy, print, viewport, and CSS media-query markers.
   - Ruff check/format, codespell, source link validation for 93 Markdown files,
     spec validation (16 specs, 121 requirements), CLI reference tests 3/3, and the
     unchanged production `make docs-check` passed 53 documentation tests, the
     191-route MkDocs baseline comparison, rustdoc, links, and artifact budgets.
   Notes:

115. [x] Replace mkdocstrings with a Griffe-backed mdBook Python API preprocessor
   Claimed by: Codex implementer (2026-07-06 22:54 PDT)
   Requirements:
   `DOCSITE-PYTHON-001`, `DOCSITE-ARCH-002`, `DOCSITE-QUALITY-001`,
   `PYAPI-DOCSTRING-001`, `PYAPI-MODULES-001`, `TEST-TDD-001`.
   Context:
   The current site renders Python docstrings through mkdocstrings. mdBook has no
   native Python API renderer, so BTPC needs a small in-repository preprocessor that
   preserves the public export inventory, signatures, annotations, examples,
   cross-references, and stable anchors without importing `btpc._native`. Griffe is
   already used in docs tests and should become the direct locked source-analysis
   dependency after the MkDocs stack is removed.
   Implementation:
   Begin with failing protocol, golden-rendering, inventory, and error tests. Add a
   typed Python executable implementing mdBook's JSON preprocessor contract,
   including the `supports <renderer>` handshake. Configure it before the built-in
   links preprocessor. Reference chapters must contain an explicit BTPC module marker
   rather than mkdocstrings directives. Statically load only `btpc.creation`,
   `btpc.metainfo`, `btpc.verification`, `btpc.types`, and `btpc.errors` through
   Griffe with inspection disabled and no native extension present.

   Generate deterministic Markdown for modules, public classes, enums, exceptions,
   functions, methods, properties, attributes, signatures, annotations, Google-style
   docstring sections, examples, and related-object links. Emit explicit canonical
   anchors compatible with the current `btpc.<module>.<symbol>` IDs wherever
   possible. Render each root re-export only on its defining module page. Reject
   unknown markers, duplicate symbols, unresolved public BTPC cross-references, and
   unsupported object shapes with actionable stderr diagnostics; stdout must contain
   only protocol JSON. Do not expose `_native`, `_conversion`, private PyO3 classes,
   source checkout paths, or requirement IDs. Keep source docstrings authoritative;
   do not create a second prose copy in generated files.
   Tests and verification:
   Show each focused test failing before implementation. Add golden cases for a
   function with callbacks and raises, a dataclass/options object, enum, exception,
   classmethod, property, overloaded/optional annotation, example block, and
   cross-module link. Test protocol version/context parsing, `supports html`, an
   unsupported renderer, malformed JSON, unknown module markers, deterministic
   repeated output, and absence of the compiled extension. Run the complete public
   export inventory and assert every object has one canonical rendered anchor and
   every private symbol has none. Run Pyrefly, Pyright compatibility smoke, Ruff,
   Python docstring tests, the side-by-side mdBook build, rendered-site checks,
   spec validation, and the unchanged MkDocs gate.
   Evidence:
   - TDD failures retained during implementation: the initial focused run failed
     because the unqualified `cancel` method role lacked class context, and the
     first real mdBook 0.5.3
     build failed because its protocol uses `items` rather than the legacy
     `sections` key. The implementation now tests both failures through
     `tests/docs/test_mdbook_python_api.py`.
   - `uv run --group docs pytest tests/docs/test_mdbook_python_api.py -q` passed
     all 5 protocol, deterministic rendering, public inventory, malformed context,
     marker, and duplicate-module tests.
   - `make docs-mdbook-site` built the complete side artifact with mdBook 0.5.3;
     `tests/docs/test_mdbook_content_port.py` passed all 4 route, anchor, and
     rendered-link checks including Python API fragments.
   - `scripts/check_python_types.sh` passed Pyrefly with 0 positive-fixture errors,
     retained the expected 3 negative-fixture errors, and passed Pyright with
     0 errors. `uv run pytest tests/python/test_docstrings.py
     tests/docs/test_mdbook_python_api.py tests/docs/test_python_reference.py
     tests/docs/test_mdbook_content_port.py -q` passed all 19 tests.
   - Focused `uv run ruff check` and `uv run ruff format --check` passed for every
     changed Python file. `uv run python scripts/check_specs.py` validated 16 specs
     and 121 requirements. `git diff --check` passed.
   - `make docs-check` preserved the production MkDocs gate during migration:
     58 docs tests passed, 406 generated files and 191 baseline routes validated,
     Markdown links and codespell passed, and rustdoc completed without warnings.
   Notes:
   - Griffe is locked directly as `griffelib==2.1.0`. Static loading uses
     `allow_inspection=False`; generated output rejects private/native paths and
     never imports the compiled extension.

116. [x] Integrate CLI reference, Rust chapter tests, and rustdoc into the mdBook artifact
   Claimed by: Codex implementer (2026-07-06 23:06 PDT)
   Requirements:
   `DOCSITE-CLI-001`, `DOCSITE-RUST-001`, `DOCSITE-BUILD-001`,
   `DOCSITE-MIGRATE-001`, `TEST-TDD-001`.
   Context:
   The side-by-side mdBook now renders handwritten and Python content. The production
   artifact must also preserve Clap-generated command pages and fresh native
   `btpc-core` rustdoc, and it should use mdBook's Rust snippet testing without
   weakening existing crate doctests.
   Implementation:
   Add failing integration tests for every generated CLI chapter in `SUMMARY.md`,
   current byte-for-byte generator drift, stable CLI anchors, successful
   `mdbook test`, the Rust landing page, and the embedded
   `rust/btpc_core/index.html` entry point. Refactor the shared typed builder so its
   side-by-side mdBook stages are: exact tool check, CLI generation/drift, Python
   preprocessor validation, clean mdBook build, `mdbook test`, warning-denied fresh
   rustdoc, rustdoc copy, route/canonical/sitemap post-processing, offline
   validation, and atomic output publication. Use an isolated Cargo target for
   rustdoc and remove stale output before every run.

   Keep the Clap command model as the only CLI schema and retain raw help, manpage,
   and completion artifacts used by releases. Ensure mdBook navigation follows the
   actual command hierarchy and all command-to-guide links resolve. Configure Rust
   examples for edition 2024 and provide `mdbook test` the built `btpc-core` library
   search path only where required. Do not replace normal `cargo test -p btpc-core
   --doc`; both test layers must remain. Make canonical URL and sitemap
   post-processing renderer-neutral, deterministic, idempotent, and route-aware.
   Tests and verification:
   Preserve initial failures. Run the CLI reference generator twice, focused CLI
   reference tests, `mdbook test`, `cargo test -p btpc-core --doc`, warning-denied
   rustdoc, and the complete side-by-side mdBook artifact build from two current
   directories. Plant stale sentinels in mdBook/rustdoc staging and prove they cannot
   survive. Validate CLI links/anchors, Rust landing and API assets, canonical URLs,
   sitemap entries, route redirects, and atomic failure behavior. Run Rust formatting,
   strict affected clippy, Ruff, docs tests, spec validation, and the unchanged
   MkDocs production gate.
   Evidence:
   - Initial `tests/docs/test_mdbook_artifact.py` run retained 3 failures: the
     builder lacked the required stages, stale rustdoc survived, and a failed
     mdBook invocation deleted the published destination. The first `mdbook test`
     also retained failures for unresolved `btpc_core` linkage before isolated
     library paths and renderer-compatible `no_run` snippets were added.
   - Two direct `btpc __generate-markdown` runs were byte-identical to each other
     and to all committed `docs/cli/reference/*.md` files. `cargo test -p btpc-cli
     --test reference` passed all 3 raw-help, website-reference, manpage, and
     completion generator tests.
   - `uv run pytest tests/docs/test_mdbook_artifact.py
     tests/docs/test_mdbook_content_port.py tests/docs/test_cli_reference.py
     tests/docs/test_rustdoc_site.py -q` passed all 12 focused tests. These cover
     every CLI chapter in `SUMMARY.md`, stable anchors, builds from two working
     directories, byte-reproducible artifacts, stale mdBook/rustdoc removal,
     idempotent post-processing, same-origin rustdoc assets, and atomic failure.
   - `make docs-mdbook-site` passed the exact mdBook 0.5.3 check, deterministic CLI
     drift check, Python preprocessor handshake, clean build, all Rust chapter
     snippet tests, fresh warning-denied rustdoc, route/canonical/sitemap
     post-processing, offline validation, and atomic publication. The artifact had
     447 files, 247 HTML pages, 55 compatibility redirects, and embedded
     `rust/btpc_core/index.html`.
   - `cargo test -p btpc-core --doc` passed the existing crate doctest. `cargo fmt
     --all --check` and strict Clippy for `btpc-core` and `btpc-cli` passed. Focused
     Ruff check/format, spec validation (16 specs and 121 requirements), and
     `git diff --check` passed.
   - `make docs-check` passed all 62 docs tests and preserved the production MkDocs
     artifact gate: 406 files, 192 HTML pages, all 191 baseline routes, rustdoc,
     links, budgets, and codespell validated.
   Notes:
   - `mdbook test` uses a clean isolated Cargo target and exposes only its
     `debug`/`debug/deps` library paths. Normal `cargo test -p btpc-core --doc`
     remains an independent required layer.

117. [-] Port the production docs gate and remove all MkDocs-specific code and dependencies
   Claimed by: Codex implementer (2026-07-06 23:16 PDT)
   Requirements:
   `DOCSITE-ARCH-002`, `DOCSITE-BUILD-001`, `DOCSITE-QUALITY-001`,
   `DOCSITE-MIGRATE-001`, `TEST-TDD-001`.
   Context:
   mdBook has reached content and artifact parity, so the repository can now switch
   its canonical local gate. This is the atomic cleanup point: no committed test,
   script, hook, command, dependency, or agent instruction may continue to require
   MkDocs after this todo. GitHub Actions remains on the old production build until
   Todo 118, providing one final separation between local cutover and deployment.
   Implementation:
   Start with failing assertions that canonical commands use mdBook and that no
   forbidden MkDocs/Material/mkdocstrings dependency or configuration remains.
   Replace MkDocs-specific navigation, configuration, HTML-layout, and Python
   reference tests with contract-equivalent `book.toml`, `SUMMARY.md`, preprocessor,
   route, and rendered mdBook assertions. Keep the generated-site validator
   renderer-neutral and preserve all quality classes: required entries, links,
   anchors, assets, canonical URLs, sitemap, private API exclusion, checkout-path
   leakage, local-only URLs, external runtime assets, duplicate titles, privacy,
   404, redirect compatibility, and compressed/uncompressed budgets. Rebaseline size
   limits from the first complete mdBook artifact with recorded evidence rather than
   silently relaxing them.

   Make `make docs-site`, `make docs-serve`, `make docs-check`, pre-commit/pre-push
   hooks, maintenance commands, and contributor instructions use the exact mdBook
   tool and shared builder. `docs-serve` must run through a wrapper or supported
   configuration that executes the Python preprocessor and watches its authoritative
   inputs. Remove `mkdocs.yml`, `docs/overrides/`, Material-only CSS, MkDocs-only
   scripts/tests, and the MkDocs/Material/mkdocstrings/PyMdown dependency graph from
   the docs uv group and lockfile. Add Griffe directly if it was previously only
   transitive. Remove PyYAML only if no remaining repository feature uses it. Update
   `AGENTS.md`, `CONTRIBUTING.md`, README documentation commands, and specifications
   to name mdBook and describe source/generated ownership. Do not edit the Pages
   workflow in this todo.
   Tests and verification:
   Demonstrate the forbidden-stack tests failing before cleanup. Run `rg` across all
   tracked files for `mkdocs`, `mkdocstrings`, `Material for MkDocs`, `pymdownx`, and
   removed paths, allowing only historical completed todo evidence and an explicit
   migration note if retained. Run the canonical docs gate, all docs tests, hooks,
   lockfile check, Ruff, Pyrefly, codespell, CLI reference tests, Rust doctests,
   warning-denied rustdoc, offline site validation, migration baseline comparison,
   spec validation, and the broader repository gate. Preview locally and record the
   final mdBook artifact sizes and page/redirect counts.
   Evidence:
   Notes:

118. [ ] Switch GitHub Actions and GitHub Pages deployment from MkDocs to mdBook
   Claimed by:
   Requirements:
   `DOCSITE-DEPLOY-001`, `DOCSITE-ARCH-002`, `DOCSITE-BUILD-001`,
   `DOCSITE-QUALITY-001`, `SEC-DEPS-001`.
   Context:
   Local commands and tests now use only mdBook, but production must not switch until
   the pull-request artifact has been inspected. Preserve the existing official
   GitHub Pages actions, least-privilege job split, fork safety, environment, and
   deployment concurrency. Only the renderer installation/build details should
   change.
   Implementation:
   Add failing structural workflow tests for the exact mdBook version, installer
   provenance, canonical `make docs-check` command, absence of MkDocs installation,
   immutable action pins, job permissions, trusted deploy condition, environment,
   timeouts, and concurrency. Update the docs build job to install the pinned mdBook
   release without changing BTPC's MSRV, sync the locked docs dependencies, run the
   canonical gate, and upload exactly `site/`. Prefer the already-used pinned
   `taiki-e/install-action` when it supports the exact mdBook release; otherwise use
   a checksum-verified official release artifact. Do not use an unpinned curl pipe,
   mutable floating tool version, branch-pushing deploy action, or `gh-pages` branch.

   Update scheduled maintenance setup so docs checks have mdBook available. Keep
   pull-request jobs read-only and ensure fork PRs can build the Python preprocessor
   without secrets. Before merging, download and inspect the PR Pages artifact or an
   equivalent manual non-deploy artifact for homepage/navigation/search/theme/CLI/
   Python/Rust/404/redirect behavior. The deploy job must remain restricted to a
   successful trusted `main` push or authorized manual dispatch with only
   `pages: write` and `id-token: write`.
   Tests and verification:
   Retain initial workflow-test failures. Parse YAML, run action-pin checks and
   `zizmor`, assert the workflow does not mention MkDocs, and execute the build job's
   commands locally with the exact CI tool versions. Record a successful pull-request
   or manual build run URL and inspect the uploaded artifact before merge. After the
   normal merge, record the successful trusted Pages deployment run URL and deployed
   SHA. Run the complete docs gate and spec validation.
   Evidence:
   Notes:

119. [ ] Verify the live mdBook cutover and update documentation operations monitoring
   Claimed by:
   Requirements:
   `DOCSITE-MIGRATE-001`, `DOCSITE-OPS-001`, `DOCSITE-DEPLOY-001`,
   `DOCSITE-QUALITY-001`.
   Context:
   The renderer migration is not complete merely because Actions deployed an
   artifact. The live project-subpath site must preserve canonical content, every
   recorded MkDocs route, key fragment links, search/assets, and operational health
   checks. Todo 105 may already have installed live-site monitoring; update that
   implementation rather than creating a competing monitor.
   Implementation:
   Fetch the production site and verify the deployed commit marker, homepage,
   Getting Started, representative guide/concept pages, complete CLI/Python reference
   indexes, representative public API anchors, Rust overview, embedded rustdoc,
   sitemap, search index, theme assets, custom 404, and HTTPS/mixed-content policy.
   Exercise every route and important anchor in Todo 112's manifest; direct routes
   must load the intended page and compatibility routes must perform one valid,
   loop-free redirect to the expected mdBook canonical destination. Compare the live
   route set with the locally generated artifact from the deployed SHA.

   Update scheduled external-link/live-site monitoring fixtures and expected page
   markers for mdBook's output while preserving read-only permissions, pinned tools,
   retries, timeouts, and actionable summaries. Update the operations runbook with
   mdBook installation, local preview, preprocessor failures, missing `SUMMARY.md`
   chapters, route redirect diagnosis, search-index problems, deployment rollback,
   and upgrade procedure. Document rollback as redeploying the last known-good
   commit; do not restore or maintain a second live documentation branch. Record a
   future mdBook upgrade checklist that reruns the theme, preprocessor protocol,
   route, artifact-size, and live smoke baselines.
   Tests and verification:
   Record the production URL, deployed commit SHA, Actions deployment URL, exact
   mdBook version, UTC/Pacific deployment time, and status/final URL/marker results
   for every baseline route. Run the scheduled monitoring workflow manually and
   record its successful run URL. Run a negative fixture proving the monitor rejects
   a generic GitHub 404 page and a redirect loop. Confirm the repository contains no
   `gh-pages` branch, deployment secret, MkDocs runtime dependency, or stale live
   canonical URL. Run the canonical docs gate, offline/live validators, spec
   validation, and `git diff --check`.
   Evidence:
   Notes:
