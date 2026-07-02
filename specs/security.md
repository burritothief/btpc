---
spec_id: SEC
title: "Security Boundaries"
status: Accepted
owners:
  - "security maintainers"
source_paths:
  - "crates/btpc-core/src/bencode.rs"
  - "crates/btpc-core/src/limits.rs"
test_paths:
  - "crates/btpc-core/tests/bencode_parser.rs"
  - "crates/btpc-core/tests/error.rs"
last_reviewed: "2026-07-01"
---

# Security Boundaries

## Requirements

### SEC-PARSE-001 — Bound untrusted metainfo parsing

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/bencode.rs`, `crates/btpc-core/src/limits.rs`
- **Verification:** `crates/btpc-core/tests/bencode_parser.rs`, `crates/btpc-core/tests/error.rs`
- **Depends on:** `BENC-LIMIT-001`

Untrusted bencode parsing **MUST** reject resource-limit violations and checked
arithmetic overflow without panicking or allocating from attacker-declared lengths.
All owned metainfo loading surfaces, including Rust, CLI, and Python path loaders,
**MUST** apply the same configurable input and ownership budgets.

### SEC-PATH-001 — Prevent payload-root escape

- **Status:** Implemented
- **Sources:** `crates/btpc-core/src/lib.rs`
- **Verification:** `crates/btpc-core/tests`
- **Depends on:** `VERIFY-PATH-001`

Creation and verification path handling **MUST** reject traversal, absolute torrent
paths, embedded separators, NUL, and symlink escape under the active policy.
Creation text filters **MUST NOT** collapse non-UTF-8 names through replacement
characters; callers needing those names use exact raw-byte path filters.

### SEC-DEPS-001 — Audit dependencies and automation

- **Status:** Accepted
- **Sources:** `deny.toml`, `.github/workflows/ci.yml`
- **Verification:** `deny.toml`, `.github/workflows/ci.yml`
- **Depends on:** None

CI **MUST** enforce dependency advisories/licenses/sources and least-privilege
workflow permissions. Third-party actions **SHOULD** be pinned to reviewed commits.

### SEC-CONFIG-001 — Treat CLI configuration as sensitive data

- **Status:** Implemented
- **Sources:** `crates/btpc-cli/src/config/mod.rs`, `crates/btpc-cli/src/diagnostics.rs`, `crates/btpc-cli/src/output.rs`, `crates/btpc-cli/src/handlers/mod.rs`
- **Verification:** `crates/btpc-cli/tests`
- **Depends on:** `SEC-PATH-001`

CLI configuration **MUST** contain data only and **MUST NOT** execute commands,
perform shell expansion, or interpolate environment variables implicitly. Config
writes **MUST** be atomic and owner-only where the platform supports permissions.
Tracker passkeys and equivalent URL credentials **MUST** be redacted from normal
output, logs, diagnostics, resolved-config explanations, and snapshots. Explicit
secret display **MUST** require `--show-secrets`, and config validation **MUST**
warn about insecure permissions.

## Design Rationale

BTPC handles attacker-controlled metadata and filesystem paths. Limits and path
containment are correctness requirements, not optional hardening.
