# Changelog

All notable changes are documented here. BTPC follows Semantic Versioning once
public compatibility is declared stable; pre-1.0 releases may make intentional
breaking changes documented in their release notes.

## [Unreleased]

- Added identity-preserving atomic `Metainfo.write()` and cached
  `CreateResult.metainfo` for Python workflows. Canonical output remains explicit
  through `write(canonical=True)` or the existing `to_bytes()` default.
- Added strict UTF-8 metadata views and a cached `Metainfo.validation` property.
  Existing raw-byte properties and `validate()` remain supported.
- Creation options now own immutable metadata sequences.
- Bencode values support sequence and dictionary access while preserving their
  existing tuple attributes.

## [0.1.2] - 2026-10-03

- Updated the Rust API checker to support Rust 1.94.1. Pull requests now compare
  against their base commit, and API compatibility gates assembled artifacts.
- Release API comparisons use a previous version tag reachable from the candidate.

## [0.1.1] - 2026-10-03

- Fixed packaged Rust crate checks with an empty Cargo cache. Locked test
  dependencies are downloaded before offline validation.
- Made the creation mutation test independent of filesystem timestamp timing.
  Documented the limits of file metadata checks during creation.
- Removed fixed release versions from creator metadata tests.

## [0.1.0] - 2026-10-03

- Added byte-safe v1, v2, and hybrid metainfo parsing and canonical serialization.
- Added deterministic streaming creation, payload verification, editing, magnets,
  native CLI, typed Python bindings, interoperability fixtures, fuzzing, and
  reproducible benchmark infrastructure.
- Release automation remains manual. Publication requires a matching release tag,
  a configured Trusted Publisher, and approval through the GitHub environment.
- Added `btpc completion generate|install|uninstall`. The hidden
  `btpc completions SHELL` compatibility alias remains available through the
  0.1.x release line and may be removed no earlier than 0.2.0.
- Fixed verification of bare relative paths and imported torrents with large piece
  lengths. v1 verification uses a fixed read buffer. v2 verification accepts BEP 52
  piece lengths above the creation policy limit.
- Bounded verification file descriptors and added final file-state checks. Payload
  names must match actual directory entries, including on case-insensitive filesystems.
- Indexed v2 piece layers and validate a shared Merkle layer once per parsing pass.
- Added installed-wheel behavior, native signature, and external typing checks.
  Release gates now check one resolved commit and support a TestPyPI rehearsal.
  Rust crate publication is opt-in for Python releases.

[Unreleased]: https://github.com/burritothief/btpc/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/burritothief/btpc/tree/v0.1.2
[0.1.1]: https://github.com/burritothief/btpc/tree/v0.1.1
[0.1.0]: https://github.com/burritothief/btpc/tree/v0.1.0
