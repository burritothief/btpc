---
title: Rust API guide
---

# Rust API Guide

`btpc-core` is the protocol source of truth. It forbids unsafe Rust, preserves raw
bytes, exposes structured errors, and uses per-operation configurable concurrency
rather than an ambient global thread pool. The workspace MSRV is Rust 1.85.

> **Development documentation:** The <a href="btpc_core/index.html">embedded
> <code>btpc-core</code> rustdoc</a> is generated from the current `main` branch
> and may change before a stable release. The crate is prepared and validated for
> crates.io, but the owner has not performed the first publish, so no live docs.rs
> release page is claimed yet.

Use the [crate source](https://github.com/burritothief/btpc/tree/main/crates/btpc-core)
for implementation context and the
[executable consumer examples](https://github.com/burritothief/btpc/blob/main/crates/btpc-core/tests/documentation.rs)
for independently compiled facade usage.

## Quick Start

## Parse and Inspect

```rust,no_run
# extern crate btpc_core;
use btpc_core::{Metainfo, TorrentMode};

let bytes = b"d4:infod6:lengthi0e4:name5:empty12:piece lengthi16e6:pieces0:ee";
let torrent = Metainfo::from_bytes(bytes)?;
assert_eq!(torrent.mode(), TorrentMode::V1);
assert_eq!(torrent.original_bytes(), bytes);
assert!(torrent.validate().is_valid());
# Ok::<(), btpc_core::Error>(())
```

`Metainfo` is owned and immutable. `original_bytes()` and `write_original()` retain
source identity; `to_bytes()` and `write_canonical()` emit deterministic canonical
bencode. Canonical bytes are materialized and cached only when one of those
canonical-output methods is called, so ordinary inspection does not serialize the
owned bencode tree. Source info hashes always use the exact raw `info` slice.

Multi-file path graphs reject duplicate and file/directory-prefix collisions.
Hybrid inspection includes validated alignment padding in `files()` and exposes it
through `TorrentFile::is_padding()`; callers that need only payload files should
filter that flag explicitly.

Default loading applies conservative input, depth, item, byte-string, parser-tree,
and owned-snapshot limits. Advanced callers can pass `ParseOptions` to
`Metainfo::from_bytes_with_options` or `Metainfo::from_path_with_options`. Path
loading preflights regular-file length and also caps the actual read, so growth
races and non-regular streams cannot bypass `max_total_input`.

Callers that already own the input buffer can use `Metainfo::from_vec` or
`Metainfo::from_vec_with_options` to transfer it into the parsed object without an
additional input copy. The object retains those exact source bytes for original
serialization and source info hashes.

The low-level `bencode::Integer` borrows the complete signed decimal bytes and
supports fallible `to_i64()`/`to_u64()` conversion. `OwnedValue::integer_bytes`
preserves arbitrary-precision canonical integers without a heavyweight arithmetic
dependency; typed metainfo fields still reject values outside their protocol
domain with field context. `ParseLimits::with_max_integer_digits` bounds hostile
digit runs independently of numeric conversion.

## Create

```rust,no_run
# extern crate btpc_core;
use btpc_core::create::{
    CreateMode, CreateOptions, Creator, HashThreads, NoProgress, PieceLength,
};

let options = CreateOptions::builder()
    .mode(CreateMode::Hybrid)
    .piece_length(PieceLength::Exact(16_384))
    .hash_threads(HashThreads::Exact(1))
    .creation_date(0)
    .build()?;
let result = Creator::new("payload")
    .options(options)
    .create(&NoProgress)?;
assert!(result.info_hash_v1().is_some());
assert!(result.info_hash_v2().is_some());
# Ok::<(), btpc_core::Error>(())
```

Creation scans deterministically, streams payload files with bounded memory,
selects deterministic automatic piece lengths, and supports v1, v2, and hybrid.
`ManifestOptionsBuilder::include` and `exclude` are UTF-8 glob APIs and reject a
non-UTF-8 payload path when active. Use `include_raw_paths` and
`exclude_raw_paths` for exact component-by-component byte matching. Top-level
symlinks are rejected by default; skip also reports an error because it cannot
produce a meaningful manifest, while follow retains the selected link name and
still enforces nested escape and cycle checks.

Manifest entries snapshot file type, size, available timestamps, and device/inode
on Unix or volume/file identity on Windows. Every hasher validates both the
pathname and opened handle before and after reading. This detects replacement,
rename-over-open, truncation/regrowth, and most concurrent mutation. Filesystems
that reuse identity and restore every exposed timestamp can still defeat metadata
snapshotting, so applications requiring a frozen tree should provide one at the
filesystem or storage layer.

Use `Creator::create_to_path` for atomic output with an explicit overwrite policy.
`Creator::create_to_path_with_durability` additionally accepts
`DurabilityPolicy::FileAndDirectory`, which syncs the parent directory after
publication where the platform supports directory syncing. Deny mode never
replaces an existing path, including a symlink. Replace mode replaces the
symlink entry rather than its target and preserves permissions when replacing a
regular file.
`HashThreads::Exact(1)` retains the sequential oracle; automatic and explicit
parallel modes remain bounded to the operation.

## Verify

```rust,no_run
# extern crate btpc_core;
use btpc_core::Metainfo;
use btpc_core::create::NoProgress;
use btpc_core::verify::Verifier;

let torrent = Metainfo::from_path("payload.torrent")?;
let report = Verifier::new(&torrent, "payload").verify(&NoProgress)?;
assert!(report.is_valid());
# Ok::<(), btpc_core::Error>(())
```

Verification reports deterministic mismatch values and checks every hash domain
applicable to the mode. Symlinks are rejected rather than followed. Options select
fail-fast behavior and extra-file reporting.

## Progress, Cancellation, and Errors

Implement `create::ProgressSink` for presentation-independent progress snapshots.
Clone `create::CancellationToken` into the caller and operation to request
cooperative cancellation. Public functions return `btpc_core::Result<T>` with a
structured `ErrorCategory`, contextual byte offset/field/path where applicable,
and preserved I/O sources.

## Layering

Applications should depend on `btpc-core` for protocol behavior. Do not duplicate
bencode parsing, hashing, traversal, Merkle, canonicalization, or verification in
adapters. The CLI and Python packages demonstrate the intended thin-wrapper model.

## Stability And Features

The supported namespaces are the crate-root facade plus `bencode`, `create`,
`edit`, `magnet`, `metainfo`, and `verify`. Private implementation modules remain
private, and public signatures use standard-library or BTPC-owned types rather
than exposing hashing, glob, CLI, Python, URL, or executor dependencies.

`btpc-core` currently has no optional behavior: its explicit default feature set
is empty, and default, no-default-feature, and all-feature builds are equivalent.
New features require a concrete dependency or platform benefit and must remain
additive. `tests/rust-consumer` compiles documented facade paths as an independent
crate. Pull requests run `cargo-semver-checks` against the base revision; release
branches run `scripts/check_rust_api.sh <accepted-tag-or-revision>` before version
publication.
