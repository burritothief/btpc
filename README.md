# BTPC

BTPC is a high-performance BitTorrent metainfo toolkit with one Rust core powering
a native `btpc` CLI and typed Python bindings. It reads, validates, creates,
inspects, edits, writes, generates magnets for, and verifies v1, v2, and hybrid
metainfo. It is not a downloader, peer, tracker, DHT node, or payload editor.

**[Documentation](https://burritothief.github.io/btpc/)** ·
**[Source](https://github.com/burritothief/btpc)** ·
**[Issues](https://github.com/burritothief/btpc/issues)**

## Installation

BTPC is pre-1.0 and is not published yet. Build the CLI from a checkout with the
Rust toolchain declared in `rust-toolchain.toml`:

```console
cargo build --release -p btpc-cli
./target/release/btpc --version
```

Build and install the Python extension into the uv environment with:

```console
uv sync --all-groups --locked
uv run maturin develop --release
uv run python -c "import btpc; print(btpc.__version__)"
```

The supported development matrix is Rust stable plus MSRV 1.85, Linux/macOS/
Windows, and CPython 3.11 through 3.14.

Python wheels currently require the CPython GIL and do not support
subinterpreters. Native hashing releases the GIL while it runs.

## Five-Minute CLI Tour

```console
mkdir payload
printf 'hello torrent\n' > payload/hello.txt

btpc create payload -o payload-v1.torrent
btpc create payload --mode v2 --piece-length 16384 -o payload-v2.torrent
btpc create payload --mode hybrid --piece-length 16384 -o payload-hybrid.torrent

btpc inspect payload-hybrid.torrent
btpc validate payload-hybrid.torrent
btpc magnet payload-hybrid.torrent
btpc verify payload-hybrid.torrent payload
```

Add `--json` to `create`, `inspect`, `validate`, or `verify` for versioned,
machine-readable stdout. Diagnostics and progress belong on stderr.

### CLI Toolkit

The CLI keeps protocol work in `btpc-core` while providing configuration,
batching, selective inspection, safe editing, and shell integration:

```console
btpc create INPUT... --preset private --output-dir ./torrents
btpc create --batch jobs.toml --dry-run
btpc inspect file.torrent --field info-hash-v1 --format plain
btpc inspect file.torrent --tree --pretty
btpc edit file.torrent --output edited.torrent --comment "release"
btpc config preset list
btpc config explain create --preset private ./payload
btpc completion install zsh --dry-run
```

Configuration uses one versioned user-scoped TOML file with tracker aliases,
tracker groups, and ordered creation presets. Resolution is deterministic:
hardcoded defaults, global config, selected presets, then explicit CLI arguments.
`--no-config` provides a reproducible config-free execution path.

Default human output remains intentionally minimal and pipe-safe. Pretty output,
symbols, and progress are opt-in or terminal-aware; JSON and plain field output
remain stable automation interfaces.

The next human-inspection presentation work follows mkbrr's compact style:
a `Torrent info:` heading, aligned labels, IEC sizes, applicable info hashes,
magnet URI, and grouped tracker/web-seed URLs. Pretty or verbose mode adds exact
bytes, metadata, warnings, and file trees while machine formats remain unchanged.

## Python Quick Start

```python
from btpc import CreateOptions, Metainfo, TorrentMode, create

result = create(
    "payload",
    "payload-hybrid.torrent",
    options=CreateOptions(
        mode=TorrentMode.HYBRID,
        piece_length=16_384,
        creation_date=0,
        threads=1,
    ),
)
torrent = Metainfo.from_bytes(result.bytes)
assert torrent.verify("payload").is_valid
print(torrent.magnet())
```

Python exposes immutable typed values, a structured `BtpcError` hierarchy,
progress callbacks, and cooperative cancellation.

Python textual creation/editing inputs are
ordinary `str` values—for example tracker URLs and `created_by="my-tool"`—while
keeping raw parsed torrent paths and extension bytes lossless. New torrents will
default to `created by = btpc/<version>` unless explicitly overridden or omitted.
BTPC ships `py.typed` and native stubs. Pyrefly is the primary repository checker,
with a strict Pyright compatibility smoke for Pylance consumers.

## Rust Quick Start

```rust
use btpc_core::create::{CreateMode, CreateOptions, Creator, NoProgress, PieceLength};

let options = CreateOptions::builder()
    .mode(CreateMode::Hybrid)
    .piece_length(PieceLength::Exact(16_384))
    .creation_date(0)
    .build()?;
let result = Creator::new("payload")
    .options(options)
    .create(&NoProgress)?;
println!("{} bytes", result.bytes().len());
# Ok::<(), btpc_core::Error>(())
```

`btpc-core` keeps protocol logic, hashing, traversal, canonical serialization,
and verification in safe Rust. Generate crate documentation with `cargo doc`.

## Protocol Modes

| Mode | Hash domain | Representation | Typical compatibility |
| --- | --- | --- | --- |
| v1 | SHA-1 pieces over the concatenated file stream | `pieces`, `files`/`length` | Broadest legacy client support |
| v2 | SHA-256 Merkle tree per file (BEP 52) | `file tree`, optional `piece layers` | Modern BEP 52 clients |
| hybrid | Both matching v1 and v2 views | Both representations, including v1 alignment padding | Migration between v1 and v2 clients |

For v2 and hybrid creation, explicit piece lengths must satisfy BEP 52 and be at
least 16 KiB. Automatic selection is deterministic. Hybrid torrents apply v1
padding while retaining the v2 per-file tree.

## Reproducibility and Bytes

- Omit timestamps by default, or set a fixed `--creation-date`/`creation_date`.
- Use `--threads 1` or `threads=1` for the sequential correctness oracle; `0`
  selects a conservative automatic count and explicit values select bounded
  per-operation concurrency.
- Traversal, torrent path ordering, dictionary ordering, and automatic piece
  selection are deterministic.
- Protocol byte strings remain bytes. CLI JSON renders non-UTF-8 values as tagged
  hex; Python exposes `bytes` plus optional decoded convenience properties.
- Parsed objects retain exact original bytes and compute source info hashes from
  the original raw `info` slice. `to_bytes()` emits canonical bencode, so a
  parseable noncanonical source may have different canonical bytes and hashes.

## Verification, Errors, and Cancellation

Verification checks structure and every applicable hash domain. Missing, wrong
size, extra, unsafe path, v1 hash, and v2 hash mismatches are deterministic.
Symlinks are never followed by default. CLI exit codes are stable: `1` internal,
`2` usage, `3` I/O/path, `4` invalid data, `5` unsupported, `6` payload mismatch,
and `130` cancellation/interrupt.

Rust returns structured `btpc_core::Error`; Python maps the same categories to
`BtpcError` subclasses. Creation and verification accept progress observers and
cooperative cancellation without ambient global thread pools.

## Performance and Benchmarks

BTPC streams payloads with bounded memory and retains sequential correctness
oracles for optimized paths. Performance claims should record pinned competitor
versions, input identity, command lines, elapsed-time distributions, throughput,
peak RSS, cache state, and output validation.

## Architecture and Non-Goals

```text
btpc-cli ───────► btpc-core ◄────── btpc-python
                         ▲
benchmark harness ───────┘
```

BTPC intentionally excludes peer wire protocols, downloading, seeding, tracker
hosting, DHT operation, and payload modification. It also does not promise that
canonical serialization is byte-identical to a noncanonical source.

## License

BTPC is licensed under the [MIT License](LICENSE).

Copyright (c) 2026 Jeff.
