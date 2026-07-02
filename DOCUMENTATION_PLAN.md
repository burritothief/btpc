# BTPC Production Documentation Site Plan

## Outcome

BTPC will publish one production documentation site at:

```text
https://burritothief.github.io/btpc/
```

The site will be rebuilt automatically from the default branch and deployed to
GitHub Pages. Pull requests will run the identical build in strict mode without
receiving deployment permissions. The repository remains the source of truth for
handwritten guides, generated CLI reference, Python docstrings, and Rust rustdoc.

## Architecture Decision

Use one unified site rather than separate Python and Rust websites:

| Concern | Decision |
| --- | --- |
| Site generator and theme | Material for MkDocs |
| Python API reference | mkdocstrings with the current Griffe-based Python handler |
| Rust API reference | `cargo doc`/rustdoc embedded below `/rust/` |
| CLI reference | Generated from the Clap command model |
| Hosting | GitHub Pages project site |
| Deployment | Official GitHub Pages Actions workflow |
| Source | `docs/` plus API documentation in source code |
| Contract | `specs/documentation-site.md` |
| Release Rust documentation | docs.rs after publication |

MkDocs owns the landing pages, navigation, search, tutorials, Python reference,
and CLI guide. Rustdoc remains the native Rust renderer because it provides the
best trait, implementation, source, feature, and intra-doc-link experience. The
MkDocs Rust landing page links into rustdoc copied under the same Pages artifact.

## Information Architecture

```text
BTPC Documentation
|-- Home
|-- Getting Started
|   |-- Installation
|   |-- CLI Quick Start
|   |-- Python Quick Start
|   `-- Rust Quick Start
|-- Guides
|   |-- Creating Torrents
|   |-- Inspecting and Validating
|   |-- Verifying Payloads
|   |-- Editing Metainfo
|   `-- Configuration and Presets
|-- Concepts
|   |-- BitTorrent v1
|   |-- BitTorrent v2
|   |-- Hybrid Torrents
|   |-- Piece Length
|   `-- Reproducibility and Bytes
|-- CLI
|   |-- Overview
|   |-- Configuration
|   |-- Shell Completion
|   `-- Command Reference
|-- Python
|   |-- Overview
|   |-- Examples
|   `-- API Reference
|       |-- creation
|       |-- metainfo
|       |-- verification
|       |-- types
|       `-- errors
|-- Rust
|   |-- Overview
|   `-- btpc-core rustdoc
|-- Performance
|-- Compatibility
|-- Security
`-- Contributing
```

The first release documents `main` and labels it as development documentation.
Do not introduce `mike`, multiple version roots, or another version manager until
BTPC has more than one supported release line.

## Repository Layout

```text
btpc/
|-- mkdocs.yml
|-- docs/
|   |-- index.md
|   |-- 404.md
|   |-- getting-started/
|   |-- guides/
|   |-- concepts/
|   |-- cli/
|   |   `-- reference/       # generated Markdown pages
|   |-- python/
|   |   `-- reference/       # small mkdocstrings directives
|   |-- rust/
|   |   `-- index.md
|   |-- performance.md
|   |-- compatibility.md
|   |-- security.md
|   `-- contributing.md
|-- python/btpc/             # authoritative Python docstrings
|-- crates/btpc-core/src/    # authoritative Rust docs and doctests
|-- scripts/build_docs.*     # one local/CI build entry point
|-- tests/docs/              # generated-site and navigation checks
`-- .github/workflows/docs.yml
```

Generated site output belongs in `site/` and must remain ignored. Generated CLI
Markdown may be checked in so changes to command help are reviewable and drift can
fail CI. Rustdoc and MkDocs HTML are build artifacts and must not be committed.

## Deterministic Build Pipeline

There must be one repository command that local development, pre-push checks,
pull requests, and Pages deployment all invoke. Its logical steps are:

1. Verify locked Python and Rust toolchains.
2. Regenerate CLI reference pages from the current `btpc` command model.
3. Fail if checked-in generated reference pages drift.
4. Build MkDocs with `--strict` into a clean staging directory.
5. Build `btpc-core` rustdoc with dependencies excluded and warnings denied.
6. Copy rustdoc into the staged site at `rust/btpc_core/`.
7. Validate required pages, local links, anchors, assets, canonical URLs, and the
   absence of repository-local paths or unpublished files.
8. Produce one self-contained Pages artifact with `index.html` at its root.

The build must not depend on a developer's editable installation, home directory,
network-fetched application data, or previously generated `target/doc` contents.
Use a dedicated Cargo target directory or clean rustdoc destination so stale docs
cannot survive between builds.

## Python API Reference

Generate the Python reference from the public modules:

- `btpc.creation`
- `btpc.metainfo`
- `btpc.verification`
- `btpc.types`
- `btpc.errors`

Use mkdocstrings' source discovery path instead of relying on the current working
directory. Render public symbols, annotations, signatures, attributes, examples,
and documented exceptions while excluding `_native`, `_conversion`, and private
implementation names. Common root re-exports should link to one canonical defining
module instead of producing duplicate pages.

Todo 95 supplies the polished public docstrings. The site implementation must add
an export inventory test so every supported public symbol is either rendered or
explicitly documented as intentionally omitted.

## Rust API Reference

Build rustdoc from the workspace's pinned stable toolchain:

```console
RUSTDOCFLAGS="-D warnings" cargo doc -p btpc-core --all-features --no-deps
cargo test -p btpc-core --doc
```

Rust public items should use concise `//!` and `///` documentation, executable
examples, error and panic contracts where relevant, and intra-doc links. Copy only
the fresh rustdoc output needed by `btpc-core` into the staged site. The public
landing page must explain that embedded rustdoc documents `main`; released crate
versions will link to docs.rs once available.

## CLI Reference

The Clap command model is the only source of truth for command reference. Extend
the current generator to produce readable Markdown pages with command synopsis,
options, inherited global flags, examples, and links between parent and child
commands. Preserve generated manpages and shell completions for packaging, but do
not expose raw completion scripts in the primary documentation navigation.

Task-oriented CLI guides remain handwritten. Generated pages answer “what flags
exist”; guides answer “how do I complete a workflow.” CI must fail when the binary
and checked-in command reference differ.

## Presentation and User Experience

The initial production theme should include:

- Responsive Material layout with light and dark palettes.
- Client-side search, code-copy controls, anchored headings, and readable tables.
- Syntax highlighting for shell, Python, Rust, TOML, JSON, and bencode examples.
- Repository, issue tracker, edit-page, and license links.
- A custom 404 page, generated sitemap, canonical `site_url`, and social metadata.
- Clear “development documentation” labeling until the first stable release.
- Keyboard-accessible navigation, meaningful heading order, alt text for images,
  visible focus states, and respect for reduced-motion preferences.

Do not add analytics, advertising, cookie banners, externally hosted fonts, or
third-party JavaScript initially. Prefer system fonts and self-contained assets so
the site remains fast, private, and reproducible.

## GitHub Actions Design

Use one documentation workflow triggered by `pull_request`, pushes to `main`, and
`workflow_dispatch`.

### Build Job

- Runs for pull requests, default-branch pushes, and manual dispatches.
- Uses read-only repository permissions.
- Installs locked dependencies with `uv` and the pinned Rust toolchain.
- Invokes the same repository documentation build command used locally.
- Uploads the complete static site as the Pages artifact.
- May upload a short-retention diagnostic artifact on failed or pull-request builds.
- Pins every third-party action to an immutable commit SHA with a version comment.

### Deploy Job

- Runs only after a successful build from `main` or an authorized manual dispatch.
- Uses the `github-pages` environment and reports the deployed URL.
- Receives only `pages: write` and `id-token: write` in addition to read access.
- Uses the official `actions/deploy-pages` action.
- Uses concurrency that cancels obsolete in-progress deployments but never cancels
  an active production deployment midway through publishing.
- Never runs for pull requests, including pull requests from forks.

GitHub Pages must be configured to use GitHub Actions as its publishing source.
Add a deployment protection rule that allows only the default branch to deploy.

## Quality Gates

Every pull request must prove:

- `mkdocs build --strict` succeeds from the locked environment.
- Python API collection succeeds and the public export inventory is complete.
- rustdoc has no warnings and Rust doctests pass.
- CLI generated reference has no drift.
- Internal links, anchors, images, scripts, styles, and canonical project-subpath
  URLs resolve from the generated artifact.
- The root page, custom 404 page, Python reference, CLI reference, and rustdoc entry
  point exist.
- Spelling checks cover handwritten documentation but exclude generated command
  output where appropriate.
- The resulting Pages artifact remains below a documented size budget.
- Workflow files pass existing YAML, action-pinning, and `zizmor` checks.

Run external-link validation separately on a schedule. Network instability should
not make ordinary documentation pull requests flaky, but recurring broken links
must remain visible as a maintenance failure.

## Production Operations

- Publish on every successful push to `main` and permit manual redeployment.
- Keep only the current `main` site until versioned docs are justified.
- Add a weekly live-site smoke check for the homepage and key entry points.
- Keep a maintainer runbook for enabling Pages, re-running deployment, diagnosing
  404/base-path problems, rotating a future custom domain, and rolling back by
  redeploying a known-good commit.
- Validate HTTPS and the canonical project URL after first deployment.
- Add the documentation URL to `README.md`, Cargo package metadata, Python project
  URLs, and the GitHub repository homepage.

## Implementation Sequence

1. Accept the documentation-site contract and lock the toolchain.
2. Add the deterministic site builder and minimal strict MkDocs skeleton.
3. Build the public information architecture and production theme.
4. Generate and validate the Python API reference.
5. Generate and embed fresh Rust rustdoc.
6. Generate readable CLI Markdown and enforce drift checks.
7. Add generated-site QA, contributor commands, and artifact budgets.
8. Add the least-privilege GitHub Pages build/deploy workflow.
9. Add discoverability metadata and maintainer operations guidance.
10. Enable Pages and verify the live production site end to end.

## Deferred Work

- Versioned documentation and `mike`.
- A custom domain.
- Analytics or telemetry.
- Search services that require an external crawler.
- PR-specific public preview environments.
- Translations.

These can be reconsidered after the first stable release or demonstrated user
need. They are not required for a production-quality initial GitHub Pages site.

## Primary References

- GitHub Pages custom workflows:
  <https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages>
- GitHub Pages publishing sources:
  <https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site>
- Material for MkDocs:
  <https://squidfunk.github.io/mkdocs-material/>
- mkdocstrings Python handler:
  <https://mkdocstrings.github.io/python/usage/>
- Cargo `doc`:
  <https://doc.rust-lang.org/cargo/commands/cargo-doc.html>
- docs.rs builds: <https://docs.rs/about/builds>
