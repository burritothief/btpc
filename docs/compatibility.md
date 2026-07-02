# Compatibility and Release Guide

## Supported Development Matrix

- Rust: current stable and MSRV 1.85.
- Operating systems: Linux, macOS, and Windows.
- Python: CPython 3.11, 3.12, 3.13, and 3.14.
- Torrent modes: v1, v2 (BEP 52), and hybrid.

The matrix describes source and CI support. BTPC is pre-1.0 and no public package
registry release has been made. Final wheels and native archives must pass clean
artifact smoke tests before publication.

### Python Runtime Model

BTPC currently requires the CPython global interpreter lock. The extension
explicitly declares `gil_used = true` to PyO3, so a free-threaded CPython build
may import it only with the GIL enabled. Long-running Rust work still releases
the GIL, allowing independent BTPC operations to execute concurrently.

Subinterpreters are not supported. PyO3 0.29 rejects initialization from a
second interpreter because extension-module state and cached Python objects are
not yet per-interpreter. Use one main interpreter with threads or separate
processes. Runtime callers can inspect `btpc._native.__gil_required__` and
`btpc._native.__subinterpreters_supported__`.

## Interoperability

The committed fixture corpus covers mktorrent 1.1, mkbrr 1.23.0, torf-cli 5.2.1,
and torrenttools 0.6.2 in every mode each tool supports. It also records malformed,
noncanonical, and prior-fuzz cases with explicit accept/preserve/reject behavior.
See the [interoperability fixture documentation][interop-fixtures] in the repository.

## Stability

Before 1.0, Rust/Python APIs and CLI JSON schemas may evolve, but intentional
compatibility changes must update specs, tests, documentation, and release notes.
CLI exit categories and currently named JSON schema identifiers are tested.
Unknown protocol byte strings remain byte-safe; text convenience views are
best-effort only.

## Artifact Expectations

Release wheels and archives must be produced from one verified version, install in
clean environments, run all three torrent modes, inspect and magnetize output, and
verify payloads. Publication should use least privilege and provenance. Normative
requirements live in the [release specification][release-spec].

The Cargo workspace package version is the single source. Start a release bump
with `make version VERSION=X.Y.Z`, update `CHANGELOG.md`, then validate the tag
with `uv run python scripts/check_version.py --tag vX.Y.Z`.

## Performance Claims

No universal performance claim is valid without the canonical benchmark preflight,
pinned competitor versions, repeated randomized measurements, raw result files,
and machine/tool metadata. Reports must name limitations. See
the [benchmark documentation][benchmark-docs].

[benchmark-docs]: https://github.com/burritothief/btpc/blob/main/benches/README.md
[interop-fixtures]: https://github.com/burritothief/btpc/blob/main/tests/fixtures/interoperability/README.md
[release-spec]: https://github.com/burritothief/btpc/blob/main/specs/release.md
