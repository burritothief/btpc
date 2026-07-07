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

Build the complete documentation site from locked dependencies with `make
docs-site`. The command removes its own staging directory and recreates `site/`
from scratch, so generated output never depends on an earlier build. Preview the
same configuration at `http://127.0.0.1:8000/btpc/` with `make docs-serve`; stop the
server with Ctrl-C. Both commands resolve repository inputs through the checked-in
configuration, and `scripts/build_mdbook_site.py --site-dir PATH` may be invoked from
outside the checkout when an explicit destination is useful.

Run `make docs-check` before pushing documentation changes. It is the canonical
offline gate for source links, generated CLI drift, Python inventory, Rust
doctests/rustdoc, strict rendering, generated-site links and anchors, canonical
metadata, private-name leakage, and size budgets. `make docs-fast` is the
pre-commit subset.

The documentation build requires exactly mdBook 0.5.3. Install the reviewed
crates.io release with:

```console
cargo install mdbook --version 0.5.3 --locked
```

The downloaded `mdbook-0.5.3.crate` SHA-256 is recorded in `.mdbook-sha256` as
`742264af649df2323b283a4c1a8abc21b6f6880cf030d642500ef85c2ce81598`.
`make docs-site` validates the exact binary version and atomically publishes the
complete artifact to `site/`. `make docs-mdbook-site` is a scratch-output alias for
`.tmp/mdbook-site`.

Handwritten pages live under `docs/`; Python API text comes from public docstrings,
Rust API text comes from rustdoc comments, and CLI reference pages, raw help,
manpages, and completions come from the Clap model through
`scripts/generate-cli-reference.sh`. Never edit generated CLI files or generated
HTML directly. Generated `site/` output remains ignored and must not be committed.

The complete mdBook site budgets are 12,000,000 uncompressed bytes and 3,600,000
normalized gzip bytes. The first complete artifact recorded on July 7, 2026 is
9,908,665 and 3,017,002 bytes respectively. Increase either budget only with
measured artifact evidence recorded in the relevant todo or review.

### Documentation operations

Prerequisites are the pinned Rust toolchain, uv, and the locked dependency groups:

```console
uv sync --all-groups --locked
make docs-check
make docs-serve
```

Install the exact mdBook release shown above before previewing. A missing binary or
version mismatch is reported by `scripts/check_mdbook.py`; do not update
`.mdbook-version` without completing the upgrade checklist below.

The preview is served under the `/btpc/` project subpath. A root-relative asset,
incorrect canonical URL, or copied `404.html` that resolves above that project
subpath will fail generated-site QA. Inspect `site/` after `make docs-site` when a
page, asset, search index, sitemap, or custom 404 behaves differently in preview.
Generated HTML is disposable; fix handwritten Markdown, Python docstrings, Rust
rustdoc, the Clap command model, or the builder instead.

For build failures, start with the named pipeline stage from
`scripts/build_mdbook_site.py --list-stages`:

- A Python preprocessor protocol error means `scripts/mdbook_python_api.py` did not
  receive or return valid mdBook JSON. Run
  `uv run pytest tests/docs/test_mdbook_python_api.py -q` before changing docstrings.
- A missing chapter error means a maintained Markdown page is absent from
  `docs/SUMMARY.md`, or a summary entry names a file that does not exist. Keep the
  navigation explicit; do not hide a page from validation.
- A migration-route failure means a baseline route is missing or its
  `btpc-redirect` shim points outside `/btpc/`, loops, or takes more than one hop.
  Compare `site/` with `tests/docs/fixtures/renderer_migration_baseline.json` and
  fix `scripts/postprocess_mdbook.py` rather than hand-editing generated HTML.
- A search failure should first confirm that the hashed `searchindex-*.js` named by
  the generated page exists, contains the expected chapters, and loads under
  `/btpc/`. Rebuild from clean output before changing mdBook configuration.

One-time repository administration uses **Settings → Pages → Source: GitHub
Actions**. The `github-pages` environment must permit deployments only from the
default `main` branch without mandatory human approval. Normal pushes deploy
automatically; maintainers can use the `workflow_dispatch` control on the
Documentation workflow to rebuild a known commit. Review the workflow run and the
`github-pages` environment deployment status before announcing recovery.

**Rollback:** revert or reset the source through the normal reviewed
workflow to a known-good commit and manually dispatch the Documentation workflow
for that commit. Do not create a `gh-pages` branch, hand-upload `site/`, or add a
deployment token. If deployment fails, first distinguish a successful build from
Pages source/environment rejection, then verify the uploaded artifact structure
and the project subpath/404 behavior locally.

The weekly Repository maintenance workflow runs the offline gate, Lychee 0.24.2,
and the live Pages contract. Reproduce it after `make docs-site` with:

```console
cargo install lychee --version 0.24.2 --locked
make docs-health
```

The command first writes `.tmp/docs-external-links.md`, a deterministic inventory
of absolute links from source Markdown and generated non-rustdoc HTML with every
originating page listed beside its URL. Lychee retries transient failures, limits
host concurrency, and excludes only the documented local preview, reserved example
trackers, Pages URLs owned by the live validator, generated edit actions, and the
not-yet-published `v0.1.0` release and comparison URLs in `.lychee.toml`. Do not
suppress an entire HTTP status class. For a failure, rerun the exact reported URL,
identify its source page from Lychee's detailed output, and distinguish a temporary
timeout, rate limit, or server error from a deliberate URL migration. Update a link
at its authoritative source; add an exclusion only for an intentional non-routable
example or a URL covered by a more specific validator. The live validator also
rejects generic GitHub Pages error HTML, missing page-specific markers, insecure
final URLs, mixed-content assets, live HTML that differs from the locally built
artifact, missing baseline anchors, and compatibility redirects that loop or exceed
one document hop. When a production route intentionally changes, update
`.github/docs-health.json` and the renderer migration baseline in the same reviewed
change and verify the replacement URL over HTTPS.

**mdBook upgrades:** update `.mdbook-version` and `.mdbook-sha256`, the pinned CI
installer arguments, and contributor commands together. Then rerun preprocessor
protocol tests, all theme/navigation/search checks, the complete route and anchor
baseline, custom-404 checks, and both artifact-size budgets. Inspect the uploaded
Pages artifact before deployment, deploy through the normal Documentation workflow,
and run the live maintenance workflow. Never retain the previous renderer or a
second documentation branch as a rollback mechanism; recovery is always a
redeployment of the last known-good source commit.

The release checklist is maintained in
[`docs/release-checklist.md`](docs/release-checklist.md).

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
