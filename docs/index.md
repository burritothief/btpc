# BTPC Documentation

!!! warning "Development documentation"

    This site documents the current `main` branch before the first stable release.

BTPC is a high-performance BitTorrent metainfo toolkit with a Rust core, a native
`btpc` command-line interface, and typed Python bindings. It reads, validates,
creates, inspects, edits, and verifies v1, v2, and hybrid torrents while preserving
raw protocol bytes where identity matters.

## Start here

- Follow the [CLI guide](cli/index.md) to create and inspect torrents from a terminal.
- Use the [Python API guide](python/index.md) for typed creation, parsing, editing,
  magnet generation, and verification.
- Read the [Rust API guide](rust/index.md) to embed `btpc-core`.
- Review [compatibility](compatibility.md) and [security](security.md) policies before
  publishing or integrating BTPC.
