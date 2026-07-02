---
spec_id: RELEASE
title: "Compatibility and Release"
status: Accepted
owners:
  - "release maintainers"
source_paths:
  - "Cargo.toml"
  - "pyproject.toml"
  - "LICENSE"
  - "rust-toolchain.toml"
  - "docs/compatibility.md"
  - ".github/workflows/release.yml"
  - "scripts/package_source.py"
  - "scripts/verify_artifacts.py"
  - "CHANGELOG.md"
test_paths:
  - ".github/workflows/ci.yml"
  - "tests/python/test_release.py"
last_reviewed: "2026-07-01"
---

# Compatibility and Release

## Requirements

### RELEASE-VERSION-001 — Use one verified release version

- **Status:** Implemented
- **Sources:** `Cargo.toml`, `pyproject.toml`, `CHANGELOG.md`, `scripts/check_version.py`
- **Verification:** `tests/python/test_release.py`, `.github/workflows/release.yml`
- **Depends on:** `RUSTAPI-COMPAT-001`

Cargo packages, Python metadata/runtime, CLI output, wheels, Python sdists, native
CLI archives, and the full project source archive **MUST** derive from or be
atomically verified against one release version. Every distributable archive
**MUST** contain the project license.

### RELEASE-MATRIX-001 — Test declared compatibility floors

- **Status:** Accepted
- **Sources:** `Cargo.toml`, `pyproject.toml`, `rust-toolchain.toml`
- **Verification:** `.github/workflows/ci.yml`
- **Depends on:** `RUSTAPI-COMPAT-001`

CI **MUST** test current stable Rust, the declared MSRV, supported operating
systems, and every supported CPython minor version before release.

### RELEASE-ARTIFACT-001 — Test final distributable artifacts

- **Status:** Implemented
- **Sources:** `pyproject.toml`, `LICENSE`, `.github/workflows/release.yml`, `scripts/package_cli.py`, `scripts/package_source.py`, `scripts/verify_artifacts.py`
- **Verification:** `tests/python/test_release.py`, `scripts/smoke_wheel.py`, `scripts/smoke_cli.py`
- **Depends on:** `RELEASE-VERSION-001`, `RELEASE-MATRIX-001`

Release wheels and native archives **MUST** be smoke-tested in clean target
environments. Publication **SHOULD** use least-privilege trusted publishing and
artifact provenance when available.

The manual workflow builds CPython 3.11–3.14 wheels and native CLI archives on
Linux x86-64/AArch64, macOS x86-64/AArch64, and Windows x86-64, plus a Python
sdist and full project source archive. It validates metadata, license inclusion,
and checksums in dry-run mode. Publication remains disabled unless the operator
explicitly selects it; publishing requires the protected `pypi` environment and
an existing version-matching tag, creates a draft GitHub release, uses PyPI
Trusted Publishing, and emits GitHub build provenance.

### RELEASE-GATE-001 — Gate claims on correctness and benchmarks

- **Status:** Accepted
- **Sources:** `specs/testing.md`, `specs/performance.md`, `specs/benchmarking.md`
- **Verification:** `.github/workflows/ci.yml`
- **Depends on:** `PERF-BENCH-001`, `TEST-LAYERS-001`

A release **MUST NOT** claim protocol support or performance leadership without
passing its conformance, interoperability, artifact, and documented benchmark
gates.

### RELEASE-QUALITY-001 — Pass all source and artifact quality gates

- **Status:** Accepted
- **Sources:** `.github/workflows/ci.yml`, `AGENTS.md`
- **Verification:** `.github/workflows/ci.yml`
- **Depends on:** `RELEASE-MATRIX-001`, `RELEASE-ARTIFACT-001`, `TEST-TRACE-001`

Every release **MUST** pass protocol, Rust, Python, CLI, specification, formatting,
lint, documentation, dependency-policy, and clean-install artifact checks. Public
Rust APIs **MUST** be documented, and no required check may be silently skipped.
Local Markdown links and generated CLI references **MUST** also be checked.

### RELEASE-REPORT-001 — Publish compatibility and performance evidence

- **Status:** Accepted
- **Sources:** `specs/benchmarking.md`, `specs/release.md`
- **Verification:** `.github/workflows/ci.yml`
- **Depends on:** `RELEASE-GATE-001`, `RELEASE-QUALITY-001`

Release notes **MUST** identify intentional API/output compatibility changes and
link to raw benchmark methodology/results for performance claims. Benchmark reports
**MUST** name competitor versions and limitations rather than making unsupported
universal claims.

### RELEASE-CLI-DOC-001 — Ship current CLI reference artifacts

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/command/mod.rs`, `crates/btpc-cli/src/reference.rs`, `docs/reference`, `docs/completions`, `.github/workflows/release.yml`
- **Verification:** `crates/btpc-cli/tests/reference.rs`, `tests/python/test_release.py`
- **Depends on:** `CLI-DOC-001`, `RELEASE-ARTIFACT-001`

Release preparation **MUST** regenerate and drift-check the manpage, command help
reference, and completion scripts for every supported shell. Native archives
**SHOULD** include these artifacts in documented locations. Generated artifacts
**MUST** derive from the same Clap command model as the executable.

### RELEASE-PY-TYPING-001 — Validate typing from built Python artifacts

- **Status:** Implemented
- **Sources:** `pyproject.toml`, `python/btpc/py.typed`, `python/btpc/_native.pyi`, `.github/workflows/release.yml`
- **Verification:** `tests/python`, `tests/python/test_release.py`, `scripts/smoke_wheel.py`
- **Depends on:** `PYAPI-TYPE-COMPLETE-001`, `RELEASE-ARTIFACT-001`

Release validation **MUST** install the built wheel into a clean environment and
run external Pyrefly and Pyright compatibility examples against that installation. Wheels
and sdists **MUST** include `py.typed` and the private extension stub, and the
installed package **MUST NOT** resolve type information from the source checkout.

## Design Rationale

Compatibility and performance claims apply to shipped artifacts, not only a
developer checkout. A single version source prevents cross-language drift.
