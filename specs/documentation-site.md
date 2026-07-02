---
spec_id: DOCSITE
title: "Production Documentation Site"
status: Accepted
owners:
  - "documentation maintainers"
source_paths:
  - "DOCUMENTATION_PLAN.md"
  - "docs"
  - "python/btpc"
  - "crates/btpc-core/src"
  - "crates/btpc-cli/src"
test_paths:
  - "scripts/check_docs.py"
  - "crates/btpc-cli/tests/reference.rs"
  - ".github/workflows/maintenance.yml"
last_reviewed: "2026-07-02"
---

# Production Documentation Site

## Requirements

### DOCSITE-ARCH-001 — Publish one unified documentation site

- **Status:** Accepted
- **Sources:** `DOCUMENTATION_PLAN.md`, `docs`
- **Verification:** `scripts/check_docs.py`
- **Depends on:** None

BTPC **MUST** provide one static documentation site containing handwritten guides,
generated CLI reference, generated Python API reference, and an entry point to
embedded `btpc-core` rustdoc. The initial site **MUST** document `main` and clearly
identify itself as development documentation.

### DOCSITE-BUILD-001 — Build the complete site reproducibly

- **Status:** Accepted
- **Sources:** `DOCUMENTATION_PLAN.md`, `docs`, `scripts/check_docs.py`
- **Verification:** `scripts/check_docs.py`, `.github/workflows/maintenance.yml`
- **Depends on:** `DOCSITE-ARCH-001`

One documented repository command **MUST** create the same complete static site in
local development and CI from locked dependencies. It **MUST** begin from clean
output, run MkDocs in strict mode, reject stale generated references, and avoid
depending on a developer home directory, editable installation, or stale rustdoc.

### DOCSITE-PYTHON-001 — Generate Python reference from the public facade

- **Status:** Accepted
- **Sources:** `python/btpc`, `docs`
- **Verification:** `scripts/check_docs.py`
- **Depends on:** `DOCSITE-BUILD-001`, `PYAPI-DOCSTRING-001`, `PYAPI-MODULES-001`

The Python API reference **MUST** be generated from the documented public modules,
annotations, signatures, and docstrings. Private native and conversion machinery
**MUST NOT** appear as public API. Automated verification **MUST** account for every
supported public export and prevent duplicate canonical pages for root re-exports.

### DOCSITE-RUST-001 — Embed warning-free btpc-core rustdoc

- **Status:** Accepted
- **Sources:** `crates/btpc-core/src`, `docs`
- **Verification:** `.github/workflows/maintenance.yml`
- **Depends on:** `DOCSITE-BUILD-001`, `RUSTAPI-DOC-001`

The site build **MUST** generate fresh `btpc-core` rustdoc with warnings denied and
dependencies excluded, run Rust doctests, and publish the result below a stable
same-origin path. The Rust landing page **MUST** distinguish `main` documentation
from future released documentation hosted by docs.rs.

### DOCSITE-CLI-001 — Generate CLI reference from the command model

- **Status:** Accepted
- **Sources:** `crates/btpc-cli/src`, `docs`
- **Verification:** `crates/btpc-cli/tests/reference.rs`
- **Depends on:** `DOCSITE-BUILD-001`, `CLI-DOC-001`, `RELEASE-CLI-DOC-001`

The website's command reference **MUST** be generated from the Clap command model
and **MUST** expose readable command, option, global-flag, and subcommand pages.
CI **MUST** fail when checked-in generated reference content differs from the
current binary.

### DOCSITE-UX-001 — Provide accessible and private documentation UX

- **Status:** Accepted
- **Sources:** `DOCUMENTATION_PLAN.md`, `docs`
- **Verification:** `scripts/check_docs.py`
- **Depends on:** `DOCSITE-ARCH-001`

The site **MUST** provide responsive navigation, client-side search, light and dark
palettes, keyboard-visible focus, meaningful heading order, image alt text, syntax
highlighting, and a custom 404 page. The initial site **MUST NOT** require analytics,
cookies, externally hosted fonts, advertising, or third-party runtime JavaScript.

### DOCSITE-QUALITY-001 — Validate generated-site behavior before deployment

- **Status:** Accepted
- **Sources:** `scripts/check_docs.py`, `docs`
- **Verification:** `scripts/check_docs.py`, `.github/workflows/maintenance.yml`
- **Depends on:** `DOCSITE-PYTHON-001`, `DOCSITE-RUST-001`, `DOCSITE-CLI-001`, `DOCSITE-UX-001`

Pull requests **MUST** validate strict site generation, required entry points,
internal links and anchors, local assets, project-subpath URLs, canonical metadata,
spelling, generated-reference drift, and a documented artifact-size budget.
External network link checks **SHOULD** run on a separate schedule to avoid flaky
merge gates.

### DOCSITE-DEPLOY-001 — Deploy main through least-privilege GitHub Actions

- **Status:** Accepted
- **Sources:** `DOCUMENTATION_PLAN.md`, `.github/workflows/ci.yml`
- **Verification:** `.github/workflows/maintenance.yml`
- **Depends on:** `DOCSITE-QUALITY-001`, `SEC-DEPS-001`

Every successful push to `main` **MUST** rebuild and deploy the site through a
GitHub Pages custom workflow. Pull requests **MUST** build without deployment
permissions. Only the deployment job may receive `pages: write` and
`id-token: write`; it **MUST** use the `github-pages` environment and official
Pages artifact/deployment actions pinned to immutable revisions.

### DOCSITE-OPS-001 — Keep production documentation observable and recoverable

- **Status:** Accepted
- **Sources:** `DOCUMENTATION_PLAN.md`, `.github/workflows/maintenance.yml`
- **Verification:** `.github/workflows/maintenance.yml`
- **Depends on:** `DOCSITE-DEPLOY-001`

Maintainers **MUST** have documented procedures for enabling Pages, manually
redeploying, diagnosing base-path and 404 failures, and rolling back by deploying a
known-good commit. A scheduled check **MUST** verify the live homepage and primary
CLI, Python, and Rust entry points over HTTPS.

## Design Rationale

A single site gives users one discovery path while retaining the native renderers
best suited to Python and Rust. A shared local/CI build prevents deployment-only
behavior, and job-scoped Pages permissions keep untrusted pull requests separated
from production publishing credentials.
