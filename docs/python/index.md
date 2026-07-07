---
title: Python API guide
---

# Python API Guide

## Public Modules

BTPC supports concise root imports and canonical domain imports with identical
objects. The stable public modules are:

- `btpc.errors` for the exception hierarchy.
- `btpc.types` for protocol modes, hashes, parse options, and byte/path values.
- `btpc.metainfo` for parsed metainfo, files, and validation reports.
- `btpc.creation` for creation options, results, cancellation, and functions.
- `btpc.verification` for mismatch values, reports, and verification.

For example, `btpc.Metainfo is btpc.metainfo.Metainfo`. The compiled
`btpc._native` module and `btpc._conversion` helpers are private implementation
details and are not exported through `btpc.__all__`.

The public package is `btpc`; `btpc._native` is private. The package is typed,
ships `py.typed`, supports CPython 3.11–3.14, and delegates parsing, hashing,
traversal, Merkle construction, creation, and verification to Rust.

## Workflow Reference

- Create torrent bytes with [`create_bytes`](reference/creation.md#btpc.creation.create_bytes)
  or publish atomically with [`create`](reference/creation.md#btpc.creation.create),
  configured by [`CreateOptions`](reference/creation.md#btpc.creation.CreateOptions).
- Parse with [`Metainfo.from_bytes`](reference/metainfo.md#btpc.metainfo.Metainfo.from_bytes)
  or [`Metainfo.read`](reference/metainfo.md#btpc.metainfo.Metainfo.read), then inspect
  [`TorrentFile`](reference/metainfo.md#btpc.metainfo.TorrentFile) values.
- Edit with [`Metainfo.edit`](reference/metainfo.md#btpc.metainfo.Metainfo.edit),
  serialize with [`Metainfo.to_bytes`](reference/metainfo.md#btpc.metainfo.Metainfo.to_bytes),
  or build a magnet URI with [`Metainfo.magnet`](reference/metainfo.md#btpc.metainfo.Metainfo.magnet).
- Verify through [`Metainfo.verify`](reference/metainfo.md#btpc.metainfo.Metainfo.verify)
  or [`verify`](reference/verification.md#btpc.verification.verify) and inspect the
  resulting [`PayloadVerificationReport`](reference/verification.md#btpc.verification.PayloadVerificationReport).

## Public API Index

Root imports are aliases of the canonical objects below. Each object is documented
once on its defining-module page.

- **Creation:** [`CancellationToken`](reference/creation.md#btpc.creation.CancellationToken),
  [`CreateMetrics`](reference/creation.md#btpc.creation.CreateMetrics),
  [`CreateOptions`](reference/creation.md#btpc.creation.CreateOptions),
  [`CreateResult`](reference/creation.md#btpc.creation.CreateResult),
  [`create`](reference/creation.md#btpc.creation.create), and
  [`create_bytes`](reference/creation.md#btpc.creation.create_bytes).
- **Metainfo:** [`Metainfo`](reference/metainfo.md#btpc.metainfo.Metainfo),
  [`TorrentFile`](reference/metainfo.md#btpc.metainfo.TorrentFile), and
  [`ValidationReport`](reference/metainfo.md#btpc.metainfo.ValidationReport).
- **Verification:** [`MismatchKind`](reference/verification.md#btpc.verification.MismatchKind),
  [`PayloadMismatch`](reference/verification.md#btpc.verification.PayloadMismatch),
  [`PayloadVerificationReport`](reference/verification.md#btpc.verification.PayloadVerificationReport),
  and [`verify`](reference/verification.md#btpc.verification.verify).
- **Types:** [`UNCHANGED`](reference/types.md#btpc.types.UNCHANGED),
  [`HashValue`](reference/types.md#btpc.types.HashValue),
  [`ParseOptions`](reference/types.md#btpc.types.ParseOptions),
  [`TorrentBytes`](reference/types.md#btpc.types.TorrentBytes),
  [`TorrentMode`](reference/types.md#btpc.types.TorrentMode), and
  [`TorrentPath`](reference/types.md#btpc.types.TorrentPath).
- **Errors:** [`BencodeError`](reference/errors.md#btpc.errors.BencodeError),
  [`BtpcError`](reference/errors.md#btpc.errors.BtpcError),
  [`CancelledError`](reference/errors.md#btpc.errors.CancelledError),
  [`MetainfoError`](reference/errors.md#btpc.errors.MetainfoError),
  [`PathError`](reference/errors.md#btpc.errors.PathError),
  [`ResourceLimitError`](reference/errors.md#btpc.errors.ResourceLimitError),
  [`UnsupportedError`](reference/errors.md#btpc.errors.UnsupportedError), and
  [`VerificationError`](reference/errors.md#btpc.errors.VerificationError).

<a id="package-version"></a>
### Package version

`btpc.__version__` is the installed package version string. The module names
`btpc.creation`, `btpc.metainfo`, `btpc.verification`, `btpc.types`, and
`btpc.errors` are also available from the package root.

## Create, Read, Verify

```python
from pathlib import Path

from btpc import CreateOptions, Metainfo, TorrentMode, create

payload = Path("payload")
result = create(
    payload,
    "payload.torrent",
    options=CreateOptions(
        mode=TorrentMode.HYBRID,
        piece_length=16_384,
        threads=1,
        creation_date=0,
        trackers=(("https://tracker.example/announce",),),
        nodes=(("router.example", 6881),),
    ),
)
torrent = Metainfo.from_bytes(result.bytes)
assert torrent.mode is TorrentMode.HYBRID
assert torrent.verify(payload).is_valid
print(torrent.info_hash_v1, torrent.info_hash_v2)
print(torrent.magnet())
```

`create_bytes()` returns the same `CreateResult` without writing a destination.
Creation includes `created by = btpc/<version>` by default. Set
`CreateOptions(created_by="my-tool")` to override it or
`CreateOptions(omit_created_by=True)` to omit the field. Creator metadata is
top-level and does not change applicable info hashes.
`create()` writes atomically and requires `overwrite=True` to replace a file.
Pass `durable=True` to also sync the destination directory after publication
where the platform supports directory syncing.
Creation options match core defaults: v1, automatic piece length, conservative
automatic threads, no timestamp, and no optional metadata.

## Byte-Safe Inspection

Protocol identities are bytes:

```python
from btpc import Metainfo

torrent = Metainfo.read("payload.torrent")
print(torrent.name)       # bytes
print(torrent.name_text)  # str | None
for file in torrent.files:
    print(file.path, file.path_text, file.is_padding)
```

`from_bytes()` accepts `bytes`, `bytearray`, and contiguous buffer objects. Exact
source bytes remain in `original_bytes`; `to_bytes()` returns canonical bencode.
For parseable noncanonical input those byte sequences, and therefore source versus
canonical info hashes, can differ by design.

`Metainfo` and `CreateResult` are immutable, non-subclassable facades over owned
native Rust objects. Parsing does not eagerly create Python copies of source or
canonical bytes, file entries, trackers, web seeds, unknown fields, or validation
details. Those values are converted once on first property access and then cached
with stable property identity. Canonical serialization is also lazy in the Rust
core. These native-backed objects are intentionally not picklable; serialize a
`Metainfo` with `to_bytes(canonical=False)` or `to_bytes()`, and serialize a
`CreateResult` through its `bytes` property.

The private extension returns typed immutable `_NativeMetainfo`,
`_NativeTorrentFile`, `_NativeValidationReport`, `_NativeCreateResult`,
`_NativePayloadMismatch`, and `_NativeVerificationReport` objects rather than
snapshot dictionaries. Public applications should continue using the `btpc`
facades; the underscore-prefixed native classes remain implementation details.

Hybrid `files` includes validated v1 alignment entries. These have
`TorrentFile.is_padding == True`, a reserved `.pad` path, and no v2 pieces root.

Both loading methods use conservative native limits by default. Pass
`ParseOptions(max_total_input=..., max_owned_allocation=...,
max_integer_digits=...)` to set explicit limits; violations raise
`ResourceLimitError` before an oversized input, digit run, or owned snapshot is
accepted.

## Editing

`Metainfo.edit()` returns a newly validated object. Top-level-only edits such as
trackers or comments preserve the exact original `info` bytes and hashes, including
noncanonical source encoding. Edits inside `info`, including `private`, `source`,
or file attributes, canonicalize the updated dictionary and recompute applicable
hashes. `to_bytes()` remains the explicit fully canonical output choice.

```python
from btpc import UNCHANGED

edited = torrent.edit(comment="release")
assert edited.info_hash_v1 == torrent.info_hash_v1

unchanged = edited.edit(comment=UNCHANGED)
removed = edited.edit(comment=None)
```

Optional edit fields use a three-state contract: `UNCHANGED` preserves the current
value, `None` removes it, and a typed value sets it. The former paired `set_*`
keywords were intentionally removed before 1.0 rather than silently reinterpreted.

`Metainfo` equality compares exact original source bytes in native code, including
noncanonical encodings and top-level metadata. `Metainfo` is intentionally
unhashable before 1.0 so equality never requires copying the full source into a
Python `bytes` object or committing to a process-hash implementation.

## Progress and Cancellation

```python
from btpc import CancellationToken, CreateOptions, create_bytes

cancel = CancellationToken()

def progress(bytes_hashed: int, total_bytes: int, pieces_hashed: int) -> None:
    if bytes_hashed > total_bytes // 2:
        cancel.cancel()

create_bytes(
    "payload",
    options=CreateOptions(threads=1),
    progress=progress,
    cancellation=cancel,
)
```

Long-running native work releases the GIL. Callbacks reacquire it safely and an
exception from a callback cancels the core operation before propagating.

The extension itself currently requires the GIL, including on free-threaded
CPython builds. Subinterpreters are rejected by PyO3 and are not supported; see
the [compatibility guide](../compatibility.md#python-runtime-model).

## Errors

All public failures derive from `BtpcError` and may expose `offset`, `field`, or
`path`:

- `BencodeError`: syntax or canonical bencode failure.
- `MetainfoError`: invalid protocol field or invariant.
- `PathError`: filesystem/path failure.
- `ResourceLimitError`: configured parse or ownership budget exceeded.
- `VerificationError`: verification-category failure.
- `UnsupportedError`: unsupported policy or object.
- `CancelledError`: cooperative cancellation.

A normal verification mismatch is returned in `PayloadVerificationReport` rather
than raised. Inspect `mismatches`, each containing a stable `MismatchKind`, path,
and optional piece index.

## Optional metadata inspection

Validated inspection exposes `trackers`, `web_seeds`, `nodes`, `source`,
`comment`, `created_by`, and `creation_date` from the same native optional-metadata
model used by Rust and the CLI. Hosts and textual metadata remain `bytes`; node
ports and creation dates are integers.
