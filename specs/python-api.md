---
spec_id: PYAPI
title: "Python Public API"
status: Accepted
owners:
  - "Python maintainers"
source_paths:
  - "crates/btpc-python/src/lib.rs"
  - "python/btpc/__init__.py"
  - "python/btpc/_native.pyi"
  - "docs/python/index.md"
test_paths:
  - "tests/python/test_import.py"
  - "tests/python/test_create.py"
last_reviewed: "2026-07-01"
---

# Python Public API

## Requirements

### PYAPI-PACKAGE-001 — Keep the native extension private

- **Status:** Implemented
- **Sources:** `crates/btpc-python/src/lib.rs`, `python/btpc/__init__.py`, `python/btpc/_native.pyi`
- **Verification:** `tests/python/test_import.py`
- **Depends on:** `ARCH-BOUND-001`

The distribution and import package **MUST** be named `btpc`; the compiled
`btpc._native` module **MUST** remain a private implementation detail behind the
typed Python package.

### PYAPI-PARITY-001 — Match core behavior and defaults

- **Status:** Implemented
- **Sources:** `crates/btpc-python/src/lib.rs`, `python/btpc/__init__.py`
- **Verification:** `tests/python/test_create.py`
- **Depends on:** `ARCH-BOUND-001`

Python parsing, creation, verification, and serialization **MUST** delegate to the
core and match Rust defaults and canonical bytes.

### PYAPI-GIL-001 — Release the GIL for expensive core work

- **Status:** Implemented
- **Sources:** `crates/btpc-python/src/lib.rs`
- **Verification:** `tests/python/test_create.py`
- **Depends on:** `PYAPI-PARITY-001`

Long-running scan, I/O, hashing, and verification calls **MUST** release the GIL.
Callbacks **MUST** reacquire it safely, be throttled, and propagate exceptions by
cancelling core work.

### PYAPI-TYPES-001 — Ship complete typing and structured exceptions

- **Status:** Implemented
- **Sources:** `python/btpc/__init__.py`, `python/btpc/_native.pyi`
- **Verification:** `tests/python/test_import.py`
- **Depends on:** `ERR-MAP-001`

The package **MUST** include `py.typed`, public type information, Pythonic immutable
value objects, and a stable `BtpcError` hierarchy with structured context.

Python exposes immutable `Metainfo`, `TorrentMode`, `HashValue`, `TorrentFile`,
and `ValidationReport` wrappers. Parsing accepts bytes, bytearray, and contiguous
buffer objects; path loading accepts string path-like objects. Native inspection
functions remain private and return owned immutable PyO3 objects consumed by the
typed wrapper. Large byte strings, file lists, trackers, web seeds, unknown fields,
validation reports, and creation results **MUST** be converted lazily and cached on
first access; ordinary inspection **MUST NOT** request canonical serialization.
Native and public owner objects are non-subclassable and intentionally
non-picklable until a stable serialized-object policy is specified.
`ValidationReport` exposes protocol validity, canonicality, canonical issue
offset/message, and compatibility warnings as separate fields.
`TorrentBytes` and `TorrentPath` are frozen, hashable, raw-identity values with
optional decoded views. `TorrentPath.to_path()` returns `None` when conversion
would be lossy.

### PYAPI-DOC-001 — Keep public examples executable

- **Status:** Implemented
- **Sources:** `README.md`, `docs/python/index.md`
- **Verification:** `tests/python/test_documentation.py`, `scripts/smoke_wheel.py`
- **Depends on:** `PYAPI-PARITY-001`, `PYAPI-TYPES-001`

The documented Python create, parse, magnet, and verify workflow **MUST** execute
for v1, v2, and hybrid. The same workflow **MUST** be reusable against an
installed wheel in a clean environment without importing repository internals.

### PYAPI-DOCSTRING-001 — Provide concise editor-ready public API documentation

- **Status:** Implemented
- **Sources:** `python/btpc`
- **Verification:** `tests/python`
- **Depends on:** `PYAPI-MODULES-001`, `PYAPI-TYPE-COMPLETE-001`, `PYAPI-DOC-001`

Every public Python module, class, function, method, property, enum, exception, and
dataclass field **MUST** have human-readable documentation suitable for editor hover
help and generated API reference pages. Docstrings **MUST** use the repository's
Google-style convention, open with a direct one-line summary, and describe behavior
rather than restating the signature or implementation. Public names **MUST NOT** be
documented in terms of private native classes, adapter internals, requirement IDs,
or implementation history.

High-use workflow APIs **MUST** document parameters, returns, important exceptions,
byte/text behavior, side effects, defaults that materially affect output, and one
short representative example. This tier includes `CreateOptions`, `create`,
`create_bytes`, `Metainfo`, `Metainfo.from_bytes`, `Metainfo.read`,
`Metainfo.edit`, `Metainfo.verify`, `Metainfo.magnet`, `Metainfo.to_bytes`, the
top-level `verify`, `ParseOptions`, and cancellation/progress behavior. Examples
**SHOULD** prefer ordinary root imports, small deterministic inputs, and the shortest
code that demonstrates a real task. They **MUST NOT** use placeholder prose,
unexplained ellipses, or examples that contradict supported signatures.

Simple value properties **SHOULD** remain concise when their type and containing
class provide sufficient context. `Args`, `Returns`, `Raises`, `Attributes`,
`Examples`, and `Notes` sections **SHOULD** appear only when they add useful
information. Notes **MUST** call out easy-to-miss semantics such as exact source-byte
identity, canonical serialization, non-UTF-8 handling, atomic writes, overwrite and
durability behavior, callback argument meaning, creator defaults, and edit
`UNCHANGED`/remove/set states. Inline comments **MUST** be reserved for non-obvious
invariants or boundary decisions and **MUST NOT** narrate straightforward code.

### PYAPI-TEXT-001 — Accept natural Python strings for textual metadata

- **Status:** Implemented
- **Sources:** `python/btpc/__init__.py`, `crates/btpc-python/src/lib.rs`, `python/btpc/_native.pyi`
- **Verification:** `tests/python/test_create.py`, `tests/python/test_metainfo.py`
- **Depends on:** `PYAPI-PARITY-001`, `BENC-BYTES-001`

Public Python creation and editing APIs **MUST** accept `str` for tracker URLs, web
seed URLs, DHT node hosts, source, comment, created-by text, and other fields whose
public meaning is textual. The wrapper **MUST** encode these strings as strict
UTF-8 at the private native boundary and **MUST NOT** use replacement encoding.
Tracker tiers use `Sequence[Sequence[str]]`, web seeds use `Sequence[str]`, and
nodes use `Sequence[tuple[str, int]]`; ergonomic immutable tuples **MAY** remain the
stored normalized form but callers **SHOULD NOT** need to construct byte tuples.

Parsed metainfo and explicitly raw extension/path APIs **MUST** continue returning
or accepting `bytes` where byte identity is protocol-significant. `TorrentBytes`,
raw path components, unknown fields, raw bencode, hashes, and file attributes
**MUST NOT** be silently converted to text. Passing `bytes` to a text-only public
parameter **MUST** raise `TypeError` with the parameter name rather than remain an
undocumented compatibility path.

### PYAPI-CREATOR-001 — Apply and expose the default creator identity

- **Status:** Implemented
- **Sources:** `python/btpc/__init__.py`, `crates/btpc-python/src/lib.rs`
- **Verification:** `tests/python/test_create.py`
- **Depends on:** `PYAPI-PARITY-001`, `CREATE-CREATOR-001`

Python creation **MUST** inherit the core default creator string `btpc/<version>`
when callers omit `created_by`. Callers **MUST** be able to override it with a
Python `str` or explicitly omit the field through an unambiguous option distinct
from the default-valued omission state. Rust, CLI, and Python outputs **MUST** agree
for equivalent creator settings.

### PYAPI-TYPE-COMPLETE-001 — Certify editor-complete public typing

- **Status:** Implemented
- **Sources:** `python/btpc/__init__.py`, `python/btpc/_native.pyi`, `python/btpc/py.typed`, `pyproject.toml`
- **Verification:** `tests/python`, `tests/python/test_release.py`, `.github/workflows/ci.yml`
- **Depends on:** `PYAPI-TYPES-001`, `RELEASE-ARTIFACT-001`

An installed BTPC wheel **MUST** provide complete static information for every
documented public export, constructor, method, property, enum, callback, exception,
and top-level function so Pyright/Pylance and Pyrefly can offer completion and validate
signatures without importing repository sources. Public signatures **MUST NOT**
degrade to implicit `Any`; generic containers, callback parameters, path-like
inputs, return types, overloads, and nullable fields **MUST** be explicit.

The private native `.pyi` **MUST** remain synchronized with the extension's runtime
symbols and callable signatures. CI **MUST** run Pyrefly plus a strict Pyright compatibility smoke on
external-consumer examples installed from the built wheel and run an automated
stub/runtime parity check. Wheel/sdist tests **MUST** verify `py.typed` and every
required `.pyi` file are included. Typing examples **MUST** use `assert_type` or
equivalent assertions for creation, parsing, inspection, editing, verification,
progress callbacks, cancellation, errors, hashes, files, and raw-byte values.

### PYAPI-PYREFLY-001 — Use Pyrefly as the primary Python type checker

- **Status:** Accepted
- **Sources:** `pyproject.toml`, `python/btpc`, `tests/python/typing`, `.github/workflows/ci.yml`
- **Verification:** `tests/python/test_typing.py`, `.github/workflows/ci.yml`
- **Depends on:** `PYAPI-TYPE-COMPLETE-001`

Pyrefly **MUST** be the primary repository and CI type checker for the public Python
package, tests, and installed-wheel consumer examples. Configuration **MUST** use
the strictest practical project mode, fail on implicit or leaked `Any`, and retain
complete editor-visible signatures for every public export. The migration **MUST**
establish diagnostic parity for existing positive and negative typing fixtures
before removing mypy or Pyright from required gates. Pyright/Pylance compatibility
**MUST** remain a supported editor outcome through standard annotations and stubs,
even if Pyright is no longer a required command-line gate.

### PYAPI-NATIVE-OBJECT-001 — Reuse owned native metainfo across operations

- **Status:** Accepted
- **Sources:** `python/btpc`, `crates/btpc-python/src/lib.rs`
- **Verification:** `tests/python`
- **Depends on:** `PYAPI-PACKAGE-001`, `PYAPI-PARITY-001`, `PYAPI-GIL-001`

Once Python has parsed a `Metainfo`, magnet generation, editing, verification, and
other native operations **MUST** operate on its owned native metainfo object rather
than serializing to Python `bytes` and reparsing in Rust. The public extension
**MUST** remain private and the Python facade **MUST** preserve current behavior,
immutability, error mapping, and GIL-release guarantees. Explicit serialization
methods remain the only operations that **SHOULD** materialize complete metainfo
bytes solely for the caller.

### PYAPI-MODULES-001 — Expose public domain modules and keep machinery private

- **Status:** Implemented
- **Sources:** `python/btpc`
- **Verification:** `tests/python/test_import.py`, `tests/python/typing`, `tests/python/test_release.py`
- **Depends on:** `PYAPI-PACKAGE-001`, `PYAPI-TYPE-COMPLETE-001`

The Python package **MUST** use a hybrid module layout. `btpc.errors`,
`btpc.types`, `btpc.metainfo`, `btpc.creation`, and `btpc.verification` are public
domain modules whose documented imports follow normal compatibility guarantees.
Native and adapter machinery, including `btpc._native` and `btpc._conversion`,
**MUST** remain private. Common public names **MUST** continue to be re-exported
from `btpc` so concise imports such as `from btpc import Metainfo, CreateOptions`
remain supported. Every public object **MUST** have one canonical defining module,
and root re-exports **MUST** preserve object identity rather than wrapping or
duplicating classes. Wheels and sdists **MUST** include the complete module layout,
typing marker, and native stub without exposing private modules in `btpc.__all__`.

### PYAPI-BUFFER-001 — Minimize full-input copies at the Python boundary

- **Status:** Accepted
- **Sources:** `python/btpc`, `crates/btpc-python/src/lib.rs`
- **Verification:** `tests/python`, `benches`
- **Depends on:** `PYAPI-GIL-001`, `PERF-MEM-001`

Parsing a contiguous Python buffer **SHOULD** perform no more than one unavoidable
full-input ownership copy before Rust owns the metainfo. Optimizations **MUST** keep
Python buffer lifetimes safe, preserve support for documented contiguous buffer
objects, enforce parse limits before excessive allocation, and avoid unsafe Rust
unless measured benefit justifies a separately reviewed invariant.

### PYAPI-EDIT-001 — Represent edit preserve, remove, and set states explicitly

- **Status:** Accepted
- **Sources:** `python/btpc`
- **Verification:** `tests/python/test_edit.py`, `tests/python/typing`
- **Depends on:** `PYAPI-TYPES-001`, `PYAPI-TEXT-001`, `PYAPI-PARITY-001`

Each editable optional field **MUST** expose three unambiguous states: preserve the
existing value, remove the field, or set a typed value. The stable Python API
**SHOULD** use a typed sentinel or equivalent single-parameter representation
rather than paired value and `set_*` booleans. Any transition from the pre-1.0
paired form **MUST** include explicit compatibility tests and documentation; raw
extension fields retain their byte-safe types.

### PYAPI-IDENTITY-001 — Keep metainfo identity exact and predictably priced

- **Status:** Accepted
- **Sources:** `python/btpc`, `crates/btpc-python/src/lib.rs`
- **Verification:** `tests/python/test_metainfo.py`, `tests/python/typing`
- **Depends on:** `META-RAW-001`, `META-HASH-001`, `PYAPI-TYPES-001`

`Metainfo` equality **MUST** mean exact original metainfo-byte identity, not info-hash
or semantic equality. Equality and hashing **MUST NOT** repeatedly allocate or copy
the complete source bytes after construction. If hashability is retained, the hash
**MUST** be cached or computed natively while preserving Python's equal-values-have-
equal-hashes rule; otherwise objects **MUST** become explicitly unhashable before
1.0 with a documented compatibility decision.

## Design Rationale

Python owns ergonomics and typing; Rust owns protocol and hot paths. Keeping the
extension private permits binding changes without destabilizing the public API.
Text is ergonomic at the Python input boundary, while raw protocol identity remains
bytes in inspection and extension APIs.
