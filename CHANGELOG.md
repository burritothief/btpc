# Changelog

All notable changes are documented here. BTPC follows Semantic Versioning once
public compatibility is declared stable; pre-1.0 releases may make intentional
breaking changes documented in their release notes.

## [Unreleased]

- Release automation remains manual and non-publishing until package ownership,
  trusted publishers, and the GitHub release environment are configured.
- Added `btpc completion generate|install|uninstall`. The hidden
  `btpc completions SHELL` compatibility alias remains available through the
  0.1.x release line and may be removed no earlier than 0.2.0.

## [0.1.0] - 2026-07-01

- Added byte-safe v1, v2, and hybrid metainfo parsing and canonical serialization.
- Added deterministic streaming creation, payload verification, editing, magnets,
  native CLI, typed Python bindings, interoperability fixtures, fuzzing, and
  reproducible benchmark infrastructure.

[Unreleased]: https://github.com/btpc-dev/btpc/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/btpc-dev/btpc/releases/tag/v0.1.0
