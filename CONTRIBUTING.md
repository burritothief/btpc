# Contributing to BTPC

BTPC uses Rust 1.94.1, pinned in `rust-toolchain.toml`, as the reviewed
development, Clippy, documentation, and rustfmt owner. Cargo's `rust-version =
"1.85"` remains the library compatibility floor and is tested separately; do not
format with the MSRV toolchain. Python 3.11 or newer is managed through uv.
Install uv, then prepare the mixed project with:

```console
uv sync --all-groups
uv run maturin develop
```

## Local verification

Behavioral changes start in `specs/` and tests cite stable requirement IDs with a
`Spec: REQUIREMENT-ID` comment. Run the specification validator before the normal
language gates:

```console
uv run python scripts/check_specs.py
```

Install the repository's commit and push hooks with:

```console
make install-hooks
```

The commit stage runs specification validation and fast formatting/lint checks.
Ruff applies safe fixes and formatting at commit time, so review and re-stage any
changed files. The push stage builds the extension once, then runs strict Pyrefly,
workspace Clippy, nextest, doctests, pytest, and cargo-deny. Run `make hooks-push`
explicitly. The manual stage runs docs/reference, workflow, spec, and external
consumer checks through `make hooks-manual`. First runs may download hook and
tool caches; unchanged reruns reuse pre-commit, Cargo, and uv caches. Use pre-commit's
`SKIP=<hook-id>` only for a documented exceptional reason; `git commit --no-verify`
and `git push --no-verify` bypass local safeguards but never bypass CI.

Run the same checks used by CI:

```console
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --workspace --doc
cargo doc --workspace --all-features --no-deps
cargo deny check
uv run ruff check .
uv run ruff format --check .
scripts/check_python_types.sh
uv run pytest tests/python
```

The `Makefile` exposes focused `check`, `test-rust`, `test-python`, `lint`, and
`build-wheel` wrappers. CI additionally checks Rust 1.94.1 on Linux, macOS, and
Windows, Rust 1.85 as the MSRV with minimal direct dependencies, CPython 3.11 through 3.14, and a non-publishing
wheel build. Cargo and uv download caches are reused, but project build artifacts
are not shared across unrelated toolchains.

### Required pull-request checks

Protect `main` by requiring a pull request, one approving review, dismissal of
stale approvals, resolution of review conversations, and the following stable CI
check names before merge:

- `Repository / pre-commit`
- `Rust / quality` and `Rust / public API`
- `Rust / stable / ubuntu-latest`, `Rust / stable / macos-latest`, and
  `Rust / stable / windows-latest`
- `Rust / MSRV 1.85`
- `Python / 3.11`, `Python / 3.12`, `Python / 3.13`, and `Python / 3.14`
- `Dependencies / policy`
- `Wheel / clean install`

Do not allow force pushes or branch deletion. Require branches to be current
before merging so the recorded checks apply to the exact merge candidate. The CI
workflow has read-only repository permissions for pull requests, uses no
`pull_request_target` jobs, and cancels stale runs for superseding pushes.

The scheduled `dependency-refresh.yml` workflow upgrades Cargo and uv lockfiles
in an ephemeral runner, runs policy and correctness checks, and uploads the diff
for human review. It never commits or opens an automatic merge. Toolchain bumps
are reviewed changes: update `rust-toolchain.toml`, matching CI pins, this guide,
and the exact-version verification evidence together.

Dependabot proposes weekly grouped Cargo, uv, and GitHub Actions updates after a
seven-day cooldown. Its `uv` ecosystem updates both `pyproject.toml` constraints
and `uv.lock`; all dependency pull requests still pass Dependency Review and the
complete CI gate. CodeQL scans Rust and Python on pull requests, `main`, and a
weekly schedule. OpenSSF Scorecard and documentation/spelling maintenance run on
their own schedules. Coverage is informational and retains separate Rust LCOV and
Python Coverage.py reports rather than inventing a combined percentage. Alert
ownership, remediation, and exception requirements are in `docs/security.md`.

## Dependency policy

Add Rust dependencies with `cargo add` and Python development dependencies with
`uv add --dev`. Commit `Cargo.lock` and `uv.lock`. `cargo deny check` rejects
unknown registries and git sources, audits advisories, reports duplicate crate
versions, and permits only the reviewed license set in `deny.toml`.

## Contribution License

BTPC is distributed under the MIT License. By submitting a contribution, you
agree that your contribution may be distributed under the terms in `LICENSE`.
Do not submit code or assets that you do not have the right to license under
those terms.
