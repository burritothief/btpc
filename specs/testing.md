---
spec_id: TEST
title: "Testing and Traceability"
status: Accepted
owners:
  - "all maintainers"
source_paths:
  - "crates/btpc-core/tests"
  - "tests/python"
test_paths:
  - "crates/btpc-core/tests"
  - "tests/python"
last_reviewed: "2026-07-01"
---

# Testing and Traceability

## Requirements

### TEST-TDD-001 — Develop contract behavior test-first

- **Status:** Accepted
- **Sources:** `crates/btpc-core/tests`, `tests/python`
- **Verification:** `crates/btpc-core/tests`, `tests/python`
- **Depends on:** None

Behavioral work **MUST** begin with a failing test for the relevant accepted
requirement before production implementation, except mechanical bootstrap changes.

### TEST-TRACE-001 — Trace implemented requirements to tests

- **Status:** Accepted
- **Sources:** `scripts/check_specs.py`
- **Verification:** `scripts/check_specs.py`
- **Depends on:** `TEST-TDD-001`

Every `Implemented` requirement **MUST** reference automated verification and have
at least one test annotation. Unknown and non-implemented annotations **MUST** fail
spec validation.

### TEST-LAYERS-001 — Use complementary test layers

- **Status:** Accepted
- **Sources:** `crates/btpc-core/tests`, `tests/python`
- **Verification:** `crates/btpc-core/tests`, `tests/python`
- **Depends on:** `TEST-TDD-001`

Protocol work **SHOULD** combine focused unit/integration tests, independent golden
fixtures, bounded property tests, fuzzing, adapter parity tests, and external
interoperability checks appropriate to its risk.

### TEST-FIXTURE-001 — Record fixture provenance

- **Status:** Accepted
- **Sources:** `crates/btpc-core/tests`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `TEST-LAYERS-001`

Committed protocol fixtures **MUST** record origin, generating tool/version,
payload checksum, mode, and expected hashes or failures.

### TEST-CLI-001 — Verify CLI ergonomics across terminals and configuration

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/tests`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `TEST-TDD-001`, `CLI-COMPAT-001`

CLI changes **MUST** include command/help snapshots, config-isolation and
precedence tests, TTY/non-TTY and `NO_COLOR` coverage, stdout/stderr separation,
secret-redaction assertions, and compatibility tests for existing aliases, JSON,
and exit codes. Completion generation/install paths **MUST** be tested without
editing real shell startup files. End-to-end tests **MUST** cover both config-driven
usage and `--no-config` reproducibility.

### TEST-CLI-DISPLAY-001 — Golden-test human inspect presentation

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/tests/inspect.rs`
- **Verification:** `crates/btpc-cli/tests/inspect.rs`
- **Depends on:** `TEST-CLI-001`, `CLI-INSPECT-DISPLAY-001`

Human inspect output **MUST** have color-stripped golden coverage for v1, v2,
hybrid, tracker tiers, web seeds, optional metadata, non-UTF-8 values, multi-file
trees, warnings, narrow terminals, and non-TTY output. Tests **MUST** assert IEC
size rounding, stable field/section order, redaction, ANSI equivalence, no Unicode
dependency in default output, and unchanged JSON/plain/TSV output.

### TEST-PY-TYPING-001 — Test Python typing as an external editor consumer

- **Status:** Implemented
- **Sources:** `tests/python`, `python/btpc/_native.pyi`
- **Verification:** `tests/python`, `.github/workflows/ci.yml`
- **Depends on:** `PYAPI-TYPE-COMPLETE-001`

Typing tests **MUST** run outside the package source tree against an installed wheel
using Pyrefly and a strict Pyright compatibility smoke. Positive examples **MUST** assert inferred
types for all public API families, and negative examples **MUST** prove invalid
metadata bytes, callback signatures, option values, mutation attempts, and return
type assumptions are rejected. CI **MUST** also compare private native stub symbols
and signatures with the built extension and fail when either side drifts.

## Design Rationale

Requirement-level traceability makes agent work reviewable without mistaking test
quantity for coverage. Independent oracles reduce self-confirming protocol bugs.
