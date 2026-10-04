---
title: Release checklist
---

# Release checklist

BTPC has not been published to a package registry. Complete registry setup before
the first publication. The workflow keeps publication disabled by default and
selects TestPyPI as its default registry.

## Registry setup

Sign in to the PyPI account that will own `btpc`. Enable two-factor authentication.
Confirm that the project name is available or that the account can publish an
existing project. For a new project, add a pending Trusted Publisher. For an
existing project, add the publisher in its Publishing settings. Repeat this setup
on TestPyPI; the two registries have separate accounts and publisher records.

Use these fields for each GitHub Actions publisher:

| Field | PyPI | TestPyPI |
| --- | --- | --- |
| Project name | `btpc` | `btpc` |
| Repository owner | `burritothief` | `burritothief` |
| Repository name | `btpc` | `btpc` |
| Workflow filename | `release.yml` | `release.yml` |
| Environment name | `pypi` | `testpypi` |

Create the matching `pypi` and `testpypi` GitHub environments. Add a required
maintainer reviewer and allow deployment only from version tags matching `v*`.
For a sole maintainer, the reviewer must be able to approve their own run. With
another available reviewer, enable prevention of self-review. Configure these
rules before dispatching a publishing run. The workflow requests a short-lived
OIDC credential; no PyPI API token belongs in repository secrets.
Protect release version tags from updates and deletion.

See PyPI's [pending publisher guide](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/)
and [publisher usage guide](https://docs.pypi.org/trusted-publishers/using-a-publisher/).
Record ownership and publisher setup privately. Do not publish account credentials.

## Validate a release candidate

1. Choose a version and update it with `make version VERSION=X.Y.Z`. Update the
   changelog and its release date when publication is scheduled. Confirm that the
   README identifies the actual publication state and installation commands.
2. Run the required source gates in `AGENTS.md`, `make docs-check`, and
   `cargo deny check`. Confirm that the spec and generated CLI checks pass.
3. Build a release wheel with `uv run maturin build --release --locked --out dist`.
   Run `uv run python scripts/check_wheel.py dist/<wheel>.whl`. This gate installs
   only that wheel in a fresh environment outside the checkout. It checks all
   torrent modes, edits, hash failures, progress, cancellation, native stub
   signatures, and external Pyrefly/Pyright consumers.
4. Create a version-matching tag on the reviewed commit. Run **Release artifacts**
   with that tag and `publish=false`. Review every quality, documentation, wheel,
   sdist, CLI, packaged-crate, API, and assembled-artifact result. Each checkout
   must use the commit resolved by the version job.
5. Inspect wheel/sdist metadata, license files, supported platform tags, CLI
   archives, the source archive, and `SHA256SUMS`. The Python wheel does not
   contain the CLI. Confirm that all artifacts use the same version.

## Rehearse on TestPyPI

Dispatch the workflow from the release tag, with the matching tag input:

```console
gh workflow run release.yml --ref vX.Y.Z -f tag=vX.Y.Z -f publish=true -f repository=testpypi
```

Approve the `testpypi` environment after reviewing the candidate. This run
publishes Python artifacts and provenance. It does not publish the Rust crate or
create a GitHub release. The registry check downloads the published CPython 3.11
Linux wheel from TestPyPI and repeats the clean installation checks. Review the
TestPyPI project page, README rendering, files, and ownership.

You can also install it in a fresh environment:

```console
python -m pip install --index-url https://test.pypi.org/simple/ --only-binary=:all: --no-deps btpc==X.Y.Z
python -c "import btpc; print(btpc.__version__)"
```

BTPC has no Python runtime dependencies. Use only the selected registry for this
rehearsal. Uploaded files cannot be replaced, so use a new version for a changed
candidate. See the [TestPyPI guide](https://packaging.python.org/en/latest/guides/using-testpypi/).

## Publish and inspect

After the TestPyPI rehearsal succeeds, dispatch the same tag with
`repository=pypi` and `publish=true`. Leave `publish_crate=false` for a Python-only
release. Approve the `pypi` environment. The workflow also prepares a draft GitHub
release.

To publish `btpc-core` in the same run, select `publish_crate=true`. Configure the
`crates-io` environment and its narrowly scoped `CRATES_IO_TOKEN` first. This
optional job requests a separate approval and runs only for PyPI publication.

Confirm the production registry download check, project ownership, metadata,
README rendering, complete wheel matrix, sdist, and provenance. Check installation
with `python -m pip install --only-binary=:all: btpc==X.Y.Z`. Update documentation
to describe registry pages as live only after publication succeeds. Review the
draft release notes before making the GitHub release public.

Registry uploads cannot be replaced. If a released artifact is defective, publish
a corrected version and consider yanking the defective version with an explicit
reason. Do not delete release history or reuse a version number. A failed
post-publication check does not roll back an upload.

## Documentation and Rust crate checks

- Run `make docs-check` from a clean checkout.
- Confirm the [production documentation](https://burritothief.github.io/btpc/)
  exposes Getting Started, CLI, Python, and Rust entry points.
- Verify repository, edit, issue, license, canonical, and sitemap links use the
  `burritothief/btpc` project and `/btpc/` Pages subpath.
- Confirm the current `main` branch documentation label remains visible. Do not
  describe it as versioned release documentation until a separate
  versioning policy is implemented.
- Inspect the successful Documentation workflow and `github-pages` deployment URL
  for the release commit.
- Smoke-test the custom 404, search index, local assets, and embedded rustdoc over
  HTTPS before publishing release notes.
- Run `scripts/check_crate_package.sh 1.85.0` and
  `scripts/check_crate_package.sh 1.94.1`, then inspect
  `target/package/btpc-core-<version>.crate` for `README.md`, `LICENSE`, sources,
  and `examples/inspect.rs`.
- Run `cargo publish -p btpc-core --locked --dry-run` and the Rust API compatibility
  check against the previous release tag when one exists.
- Protect the `crates-io` GitHub environment, configure its narrowly scoped
  `CRATES_IO_TOKEN`, and approve the manual release job only for an existing
  version-matching tag. Ordinary pushes never publish the crate.
- For the first publish, confirm crate ownership and the resulting crates.io and
  docs.rs pages manually before adding links that describe either page as live.
