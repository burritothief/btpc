---
spec_id: ERR
title: "Errors and Failure Mapping"
status: Accepted
owners:
  - "core maintainers"
source_paths:
  - "crates/btpc-core/src/error.rs"
test_paths:
  - "crates/btpc-core/tests/error.rs"
last_reviewed: "2026-07-01"
---

# Errors and Failure Mapping

## Requirements

### ERR-CORE-001 — Expose structured core errors

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/error.rs`
- **Verification:** `crates/btpc-core/tests/error.rs`
- **Depends on:** None

Core APIs **MUST** return a non-exhaustive structured error with stable categories
and optional offset, field, filesystem path, limit, and source context. Public core
APIs **MUST NOT** expose `anyhow`.

### ERR-IO-001 — Preserve I/O source and path context

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/error.rs`
- **Verification:** `crates/btpc-core/tests/error.rs`
- **Depends on:** `ERR-CORE-001`

I/O failures **MUST** preserve the underlying error source and the relevant path.

### ERR-MAP-001 — Map core errors consistently at adapters

- **Status:** Accepted
- **Sources:** `crates/btpc-cli/src/diagnostics.rs`, `crates/btpc-python/src/lib.rs`
- **Verification:** `tests/python/test_import.py`
- **Depends on:** `ERR-CORE-001`

CLI exit codes and Python exception subclasses **MUST** map core categories
consistently while retaining structured context and safe human-readable messages.

### ERR-PANIC-001 — Prevent Rust panics from crossing adapters

- **Status:** Accepted
- **Sources:** `crates/btpc-python/src/lib.rs`, `crates/btpc-cli/src/diagnostics.rs`
- **Verification:** `tests/python/test_import.py`
- **Depends on:** `ERR-MAP-001`

Expected invalid input **MUST NOT** panic. Unexpected panics **MUST NOT** unwind
across the Python boundary and **MUST** be treated as defects in tests.

### ERR-PY-NATIVE-001 — Transport Python failures as typed native exceptions

- **Status:** Accepted
- **Sources:** `crates/btpc-python/src/lib.rs`, `python/btpc`
- **Verification:** `tests/python/test_import.py`, `tests/python/test_metainfo.py`
- **Depends on:** `ERR-MAP-001`, `ERR-PANIC-001`, `PYAPI-TYPES-001`

The native extension **MUST** communicate core error categories and context through
typed exception classes or an equivalently typed structured object, not debug-
formatted category strings attached opportunistically to a generic `ValueError`.
Offset, field, lossless filesystem path, resource-limit values, and the safe display
message **MUST** survive mapping to the public Python exception hierarchy. Failure
to attach or map structured context **MUST NOT** be silently ignored.

## Design Rationale

One core taxonomy lets adapters remain thin while presenting idiomatic failures.
Context is structured so callers need not parse display strings.
