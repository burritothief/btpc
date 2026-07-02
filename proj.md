## Goal

Build a high-performance torrent creation tool with:

- A fast Rust core library
- A native CLI binary
- Python bindings for use from Python code
- Optional Python package distribution through PyPI
- A design that can compete with tools like `mkbrr`, `mktorrent`, `torf`, and `torrenttools`

The intended architecture is:

```
Rust core crate
├── Rust CLI binary
├── Python extension module
└── Reusable library API
```

---

## Recommended Technology Stack

### Core implementation

Use **Rust** for:

- File traversal
- Piece splitting
- SHA-1 hashing
- Torrent metainfo construction
- Bencode serialization
- Parallelism
- Error handling

Recommended crates:

```toml
[dependencies]
anyhow = "1"
thiserror = "1"
clap = { version = "4", features = ["derive"] }
rayon = "1"
sha1 = "0.10"
walkdir = "2"
pyo3 = { version = "0.22", features = ["extension-module"] }
```

Optional later:

```toml
memmap2 = "0.9"
jwalk = "0.8"
indicatif = "0.17"
serde = "1"
serde_bytes = "0.11"
```

### Python bindings

Use:

- **PyO3** for Rust-to-Python bindings
- **maturin** for building and publishing wheels

Install during development:

```bash
pip install maturin
```

---

## Project Layout

Suggested repository structure:

```
torrent-maker/
├── Cargo.toml
├── pyproject.toml
├── README.md
├── src/
│   ├── lib.rs          # Core Rust library API
│   ├── main.rs         # CLI entrypoint
│   ├── py.rs           # Python bindings
│   ├── bencode.rs      # Bencode encoder
│   ├── metainfo.rs     # Torrent metainfo structures
│   ├── hashing.rs      # Piece hashing logic
│   ├── files.rs        # File walking and file metadata
│   └── errors.rs       # Error types
├── python/
│   └── torrent_maker/
│       ├── __init__.py
│       └── py.typed
├── tests/
│   ├── cli_tests.rs
│   └── fixtures/
└── benches/
    └── create_torrent.rs
```

---

## Cargo Setup

`Cargo.toml` should support both a Rust library and a Python extension module:

```toml
[package]
name = "torrent-maker"
version = "0.1.0"
edition = "2021"

[lib]
name = "torrent_maker"
crate-type = ["cdylib", "rlib"]

[[bin]]
name = "torrent-maker"
path = "src/main.rs"

[dependencies]
anyhow = "1"
thiserror = "1"
clap = { version = "4", features = ["derive"] }
rayon = "1"
sha1 = "0.10"
walkdir = "2"
pyo3 = { version = "0.22", features = ["extension-module"] }
```

---

## Python Packaging Setup

`pyproject.toml`:

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "torrent-maker"
version = "0.1.0"
description = "Fast torrent creation library powered by Rust"
requires-python = ">=3.9"
classifiers = [
    "Programming Language :: Python",
    "Programming Language :: Rust",
]

[tool.maturin]
features = ["pyo3/extension-module"]
python-source = "python"
module-name = "torrent_maker._native"
```

---

## Core Rust API Design

Start with a small, stable Rust API:

```rust
use std::path::{Path, PathBuf};

pub struct TorrentOptions {
    pub announce: String,
    pub piece_length: Option<usize>,
    pub private: bool,
    pub source: Option<String>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
}

pub struct TorrentFile {
    pub bytes: Vec<u8>,
    pub info_hash_v1: [u8; 20],
}

pub fn create_torrent_from_path(
    path: &Path,
    options: TorrentOptions,
) -> Result<TorrentFile, TorrentError> {
    todo!()
}
```

Keep this API independent of Python. The Python bindings should call into this core API instead of containing torrent logic directly.

---

## CLI Design

Example CLI usage:

```bash
torrent-maker ./movie.mkv \
  --announce <https://tracker.example/announce> \
  --output movie.torrent
```

Possible options:

```
Usage: torrent-maker <PATH> [OPTIONS]

Arguments:
  <PATH>                      File or directory to torrent

Options:
  -a, --announce <URL>         Tracker announce URL
  -o, --output <PATH>          Output .torrent path
  -p, --piece-length <SIZE>    Piece length in bytes
      --private                Set private flag
      --source <SOURCE>        Add source field
      --comment <COMMENT>      Add comment field
      --created-by <TEXT>      Override created by field
      --threads <N>            Number of hashing threads
      --no-progress            Disable progress bar
  -h, --help                   Show help
```

CLI implementation should be thin:

```rust
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let options = TorrentOptions::from(args);
    let torrent = torrent_maker::create_torrent_from_path(&args.path, options)?;
    std::fs::write(args.output, torrent.bytes)?;
    Ok(())
}
```

---

## Python API Design

Python users should get a simple API:

```python
from torrent_maker import create_torrent

create_torrent(
    path="./movie.mkv",
    announce="<https://tracker.example/announce>",
    output="movie.torrent",
    private=True,
)
```

Also expose a lower-level bytes-returning function:

```python
from torrent_maker import create_torrent_bytes

data = create_torrent_bytes(
    path="./movie.mkv",
    announce="<https://tracker.example/announce>",
)

with open("movie.torrent", "wb") as f:
    f.write(data)
```

Python type hints:

```python
from pathlib import Path
from typing import Optional, Union

StrPath = Union[str, Path]

def create_torrent(
    path: StrPath,
    announce: str,
    output: StrPath,
    *,
    piece_length: Optional[int] = None,
    private: bool = False,
    source: Optional[str] = None,
    comment: Optional[str] = None,
    created_by: Optional[str] = None,
) -> None: ...

def create_torrent_bytes(
    path: StrPath,
    announce: str,
    *,
    piece_length: Optional[int] = None,
    private: bool = False,
    source: Optional[str] = None,
    comment: Optional[str] = None,
    created_by: Optional[str] = None,
) -> bytes: ...
```

---

## PyO3 Binding Sketch

`src/py.rs`:

```rust
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyfunction]
#[pyo3(signature = (
    path,
    announce,
    piece_length = None,
    private = false,
    source = None,
    comment = None,
    created_by = None,
))]
fn create_torrent_bytes(
    path: PathBuf,
    announce: String,
    piece_length: Option<usize>,
    private: bool,
    source: Option<String>,
    comment: Option<String>,
    created_by: Option<String>,
) -> PyResult<Vec<u8>> {
    let options = crate::TorrentOptions {
        announce,
        piece_length,
        private,
        source,
        comment,
        created_by,
    };

    let torrent = crate::create_torrent_from_path(&path, options)
        .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))?;

    Ok(torrent.bytes)
}

#[pymodule]
fn _native(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_torrent_bytes, m)?)?;
    Ok(())
}
```

`python/torrent_maker/__init__.py`:

```python
from pathlib import Path
from typing import Optional, Union

from ._native import create_torrent_bytes

StrPath = Union[str, Path]

def create_torrent(
    path: StrPath,
    announce: str,
    output: StrPath,
    *,
    piece_length: Optional[int] = None,
    private: bool = False,
    source: Optional[str] = None,
    comment: Optional[str] = None,
    created_by: Optional[str] = None,
) -> None:
    data = create_torrent_bytes(
        path,
        announce,
        piece_length=piece_length,
        private=private,
        source=source,
        comment=comment,
        created_by=created_by,
    )
    Path(output).write_bytes(data)
```

---

## Torrent Creation Pipeline

The high-level torrent creation flow:

```
1. Accept file or directory path
2. Walk files in deterministic order
3. Compute total size
4. Choose piece length
5. Stream file bytes in order
6. Split data into fixed-size pieces
7. Hash each piece with SHA-1
8. Concatenate 20-byte piece hashes
9. Build info dictionary
10. Bencode metainfo dictionary
11. Compute info hash
12. Return or write .torrent bytes
```

---

## Performance Strategy

### Important principle

Language alone will not win benchmarks. The performance-sensitive parts are:

- File walking
- Disk reads
- Buffer reuse
- Piece construction
- SHA-1 hashing
- Parallel scheduling
- Avoiding unnecessary copies

### Initial simple implementation

Start with a correct streaming implementation:

```
read files sequentially → fill piece buffer → hash piece → append digest
```

This is simpler and easier to validate.

### Optimized implementation

Then improve to:

```
reader thread → bounded queue → worker threads hash pieces → ordered collector
```

Important: torrent piece hashes must be written in piece order, even if hashing happens in parallel.

Possible pipeline:

```
File reader
  ↓
Piece assembler
  ↓
Hash job queue
  ↓
Parallel SHA-1 workers
  ↓
Ordered piece-hash collector
  ↓
Bencode writer
```

### Avoid these early mistakes

- Loading huge torrents fully into memory
- Hashing files independently instead of hashing the continuous torrent byte stream
- Emitting piece hashes out of order
- Sorting file paths incorrectly
- Creating many small allocations per piece
- Using Python for inner hashing loops

---

## Bencode Design

You can write a small custom bencode encoder.

Bencode value model:

```rust
pub enum BValue {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<BValue>),
    Dict(Vec<(Vec<u8>, BValue)>),
}
```

Dictionary keys must be sorted lexicographically by raw bytes:

```rust
items.sort_by(|a, b| a.0.cmp(&b.0));
```

For maximum performance, encode directly into a `Vec<u8>`:

```rust
pub fn encode_into(value: &BValue, out: &mut Vec<u8>) {
    match value {
        BValue::Int(n) => {
            out.extend_from_slice(b"i");
            out.extend_from_slice(n.to_string().as_bytes());
            out.extend_from_slice(b"e");
        }
        BValue::Bytes(bytes) => {
            out.extend_from_slice(bytes.len().to_string().as_bytes());
            out.extend_from_slice(b":");
            out.extend_from_slice(bytes);
        }
        BValue::List(items) => {
            out.extend_from_slice(b"l");
            for item in items {
                encode_into(item, out);
            }
            out.extend_from_slice(b"e");
        }
        BValue::Dict(items) => {
            out.extend_from_slice(b"d");
            for (key, value) in items {
                encode_into(&BValue::Bytes(key.clone()), out);
                encode_into(value, out);
            }
            out.extend_from_slice(b"e");
        }
    }
}
```

Later optimization: avoid cloning keys during encoding.

---

## Milestones

### Milestone 1: Correct single-file torrent creation

- Create torrent for one file
- Support announce URL
- Support fixed piece length
- Write `.torrent` file
- Verify output with an existing torrent parser

### Milestone 2: Directory torrent support

- Recursively walk directories
- Sort file paths deterministically
- Build multi-file `info.files` list
- Handle path components correctly

### Milestone 3: Python bindings

- Add PyO3 module
- Return torrent bytes to Python
- Add Python wrapper function that writes to disk
- Add `py.typed`
- Add type hints

### Milestone 4: CLI polish

- Add clap-based CLI
- Add useful errors
- Add progress bar
- Add options for private/source/comment/created-by
- Add automatic piece length selection

### Milestone 5: Performance pass

- Benchmark against `mktorrent`, `mkbrr`, `torf`, and `torrenttools`
- Add parallel hashing
- Reuse buffers
- Tune queue sizes
- Test HDD and SSD behavior separately
- Measure memory usage

### Milestone 6: Packaging

- Build Rust binary releases with GitHub Actions
- Build Python wheels with maturin
- Publish to PyPI
- Publish CLI binaries for Linux, macOS, and Windows

---

## Benchmarking Plan

Benchmark against realistic datasets:

```
small-file dataset: many tiny files
single-large-file dataset: one 10–100 GB file
mixed dataset: directories with varied file sizes
HDD test: sequential-read-sensitive
SSD test: parallelism-sensitive
```

Measure:

```
wall-clock time
CPU utilization
memory usage
disk throughput
hashing throughput
output correctness
```

Example benchmark command pattern:

```bash
hyperfine \
  'torrent-maker ./dataset --announce URL --output out1.torrent' \
  'mkbrr create ./dataset --tracker URL --output out2.torrent' \
  'mktorrent -a URL -o out3.torrent ./dataset'
```

---

## Correctness Tests

Test cases:

- Empty file
- Small file smaller than one piece
- File exactly one piece long
- File crossing piece boundary
- Multi-file directory
- Unicode paths
- Nested directories
- Private torrent flag
- Custom source field
- Deterministic output

Important property:

```
Same input + same options should produce identical torrent bytes.
```

---

## Suggested Development Order

Recommended sequence:

```
1. Implement bencode encoder
2. Implement single-file torrent creation
3. Add tests against known expected bencode
4. Add CLI wrapper
5. Add directory support
6. Add Python bindings
7. Add benchmarks
8. Optimize hashing pipeline
9. Add release packaging
```

Do not optimize before correctness. A very fast invalid torrent creator is useless.

---

## Long-Term Features

Possible later additions:

- BitTorrent v2 support
- Hybrid v1/v2 torrents
- Web seeds
- Multiple trackers
- Tracker tiers
- Include/exclude glob patterns
- Piece length auto-tuning
- JSON output mode
- Dry-run mode
- Existing torrent inspection
- Torrent validation
- Magnet link generation

---

## Final Recommendation

Use this split:

```
Rust core: performance-critical torrent creation
Rust CLI: fast standalone executable
Python wrapper: ergonomic scripting API
maturin: Python wheel builds
PyO3: Rust/Python bridge
```

This gives you the performance profile of a native CLI while still allowing Python users to build libraries and automation on top of the same engine.
