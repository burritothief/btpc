# BTPC mdBook Documentation Migration Plan

## Decision

BTPC will replace Material for MkDocs with mdBook as the primary documentation
renderer while keeping the existing GitHub Pages URL:

```text
https://burritothief.github.io/btpc/
```

The migration will be a staged cutover rather than an in-place rewrite. The current
MkDocs site remains deployable until the mdBook artifact has equivalent content,
API coverage, quality checks, and route compatibility. The final cutover removes
MkDocs, Material, mkdocstrings, and their configuration in one verified change.

## Why mdBook

mdBook is a good fit for BTPC because the project is Rust-first, the documentation
is already Markdown-based, and the site needs a fast static build with hierarchical
navigation, local search, syntax highlighting, light/dark themes, code-copy tools,
keyboard navigation, and Rust example testing. mdBook 0.5 also supports custom
preprocessors, redirects, custom CSS/theme assets, Git repository/edit links, a
project-relative `site-url`, and a GitHub Pages-compatible 404 page.

The tradeoff is Python API generation. mdBook has no native equivalent of
mkdocstrings, so BTPC will own a small, tested preprocessor that reads the public
Python facade with Griffe and emits deterministic mdBook Markdown. This keeps
docstrings and annotations authoritative without retaining the MkDocs runtime.

## Target Architecture

| Concern | Target |
| --- | --- |
| Main site renderer | mdBook 0.5.x, pinned exactly for CI and contributors |
| Source directory | Existing `docs/` tree |
| Navigation | `docs/SUMMARY.md` |
| Python API reference | In-repository Griffe-backed mdBook preprocessor |
| Rust API reference | Fresh `cargo doc` embedded below `/rust/btpc_core/` |
| CLI reference | Existing Clap-generated Markdown pages |
| Site post-processing | Existing typed Python build/validation scripts |
| Hosting | GitHub Pages custom workflow |
| Deployment URL | `https://burritothief.github.io/btpc/` |

The unified site remains organized as:

```text
BTPC Documentation
|-- Home
|-- Getting Started
|-- Guides
|-- Concepts
|-- CLI
|   `-- Generated Command Reference
|-- Python
|   `-- Generated API Reference
|-- Rust
|   `-- Embedded btpc-core rustdoc
|-- Performance
|-- Compatibility
|-- Security
`-- Contributing
```

## Repository Layout

The migration should preserve the existing documentation source locations and add
only the files mdBook requires:

```text
btpc/
|-- book.toml
|-- docs/
|   |-- SUMMARY.md
|   |-- index.md
|   |-- 404.md
|   |-- getting-started/
|   |-- guides/
|   |-- concepts/
|   |-- cli/
|   |   `-- reference/       # checked-in generated Markdown
|   |-- python/
|   |   `-- reference/       # preprocessor markers and prose
|   |-- rust/
|   |   `-- index.md
|   `-- theme/
|       `-- btpc.css
|-- scripts/
|   |-- build_docs_site.py
|   |-- mdbook_python_api.py
|   `-- check_docs_site.py
|-- tests/docs/
`-- .github/workflows/docs.yml
```

`mkdocs.yml`, `docs/overrides/`, Material-specific CSS, and MkDocs-only Python
dependencies are deleted only after the mdBook path passes the complete gate.

## Toolchain Policy

Pin mdBook to an exact reviewed 0.5.x release. At the time of this plan, the current
release is 0.5.3. mdBook requires a newer Rust version than BTPC's crate MSRV, so it
must remain an external documentation tool rather than a workspace dependency.

- CI installs the exact mdBook binary with a SHA-pinned installer action or a
  verified release artifact.
- Contributor setup documents the exact `cargo install mdbook --version ...
  --locked` command.
- The canonical documentation command checks the mdBook version and fails with a
  direct installation instruction when it is missing or incompatible.
- mdBook is not added to the workspace `Cargo.lock` and must not raise BTPC's MSRV.
- Griffe remains a direct locked dependency in the `docs` uv group; MkDocs,
  Material, mkdocstrings, and PyMdown dependencies are removed after cutover.

## mdBook Configuration

`book.toml` should use `docs` as the source directory and configure:

- `build.create-missing = false` so a bad `SUMMARY.md` cannot create files.
- Rust edition 2024 for examples.
- `output.html.site-url = "/btpc/"` for GitHub Pages subpath-safe assets and 404s.
- Built-in local search with a documented result limit and heading split level.
- Local light and dark themes, repository/edit links, code-copy support, and
  sidebar header navigation.
- `input-404 = "404.md"`.
- Local additional CSS only; no analytics, cookies, ads, remote fonts, or external
  runtime JavaScript.
- A custom BTPC Python API preprocessor ordered before the built-in links
  preprocessor.
- Extra watch directories for `python/btpc`, CLI command sources, and relevant Rust
  docs so the supported preview command rebuilds when authoritative inputs change.

`docs/SUMMARY.md` becomes the sole navigation manifest. Tests must verify that every
public handwritten or generated chapter is listed exactly once, every listed file
exists, and no removed page silently disappears from the book.

## Python API Reference

The source of truth remains the public modules and their docstrings:

- `btpc.creation`
- `btpc.metainfo`
- `btpc.verification`
- `btpc.types`
- `btpc.errors`

An in-repository Python preprocessor will implement mdBook's JSON preprocessor
protocol. Reference chapters contain a small explicit marker naming one public
module. The preprocessor statically loads the package with Griffe, never imports the
native extension, and replaces the marker with deterministic Markdown containing:

- Canonical module, class, function, method, property, and attribute headings.
- Exact public signatures and annotations.
- Concise source docstrings with Google-style sections rendered consistently.
- Stable explicit HTML anchors matching the current public anchor IDs where
  possible, such as `btpc.creation.create`.
- Cross-links for BTPC public types and related objects.
- No `_native`, `_conversion`, private members, or duplicate root re-exports.

The preprocessor must support the `supports html` handshake, reject unsupported or
unknown module markers, produce no diagnostics on stdout, and report actionable
errors on stderr. Golden tests should cover representative functions, dataclasses,
enums, exceptions, overloads, callbacks, examples, and cross-references. The public
export inventory remains a hard completeness check.

## CLI Reference

Keep the current generated Markdown command reference and byte-for-byte drift
checks. Add every generated page to `SUMMARY.md` in command hierarchy order.
Explicit anchors should preserve stable links for command sections. Raw manpages,
help text, and completion artifacts may remain release inputs, but only the readable
Markdown reference appears in primary book navigation.

The mdBook migration must not introduce a second command schema or hand-maintained
flag tables.

## Rust Reference

Continue generating native rustdoc with:

```console
RUSTDOCFLAGS="-D warnings" cargo doc -p btpc-core --all-features --no-deps
cargo test -p btpc-core --doc
```

The shared builder copies only fresh `btpc-core` rustdoc and required static assets
into the completed mdBook artifact at `rust/btpc_core/`. Rustdoc remains outside the
mdBook chapter renderer because its trait, implementation, source, and intra-doc
link presentation is superior to converting it into Markdown. The Rust overview
chapter links to the embedded reference and explains that it documents `main`.

## Route Compatibility

The existing MkDocs deployment uses directory-style routes such as:

```text
/btpc/getting-started/installation/
/btpc/cli/reference/create/
/btpc/python/reference/metainfo/
```

mdBook normally emits `.html` chapter routes. Before changing the production
workflow, capture the current generated route and anchor manifest. The mdBook build
must generate compatibility redirect files for every previous public route and
preserve important API/CLI fragment identifiers. Redirects must be relative,
project-subpath safe, loop-free, and validated offline. Existing root, 404, and
embedded rustdoc routes must remain directly available.

The migration may choose directory-index chapter paths for high-value routes where
that reduces redirects, but source layout should not be contorted solely to mimic
MkDocs. A checked-in route manifest and generated redirect shims provide an explicit
compatibility contract.

## Theme and User Experience

Use mdBook's maintained default HTML theme with surgical BTPC CSS rather than
forking the full theme templates. The result must preserve:

- Responsive sidebar navigation and readable mobile layouts.
- Local light/dark palettes and visible keyboard focus.
- Search, anchored headings, code-copy controls, keyboard shortcuts, and print.
- Syntax highlighting for shell, Python, Rust, TOML, JSON, and text examples.
- A development-documentation notice, repository/edit/license links, and custom 404.
- Reduced-motion behavior and meaningful semantic heading order.

Post-process generated chapter HTML only for capabilities mdBook does not expose
cleanly in configuration, such as per-page canonical URLs and the sitemap. Keep that
post-processing deterministic, idempotent, parser-based where practical, and
covered by fixture tests. Do not maintain a full copied `index.hbs` unless a tested
requirement cannot be met through configuration, CSS, or post-processing.

## Deterministic Build Pipeline

Retain one typed repository command for local builds and CI. Its stages become:

1. Check the exact mdBook version and locked Python docs environment.
2. Build the CLI and reject generated Markdown drift.
3. Run the Python API preprocessor inventory/golden checks.
4. Build mdBook from `book.toml` and `docs/SUMMARY.md` into a clean staging path.
5. Run `mdbook test` for Rust snippets, with the built `btpc-core` library path when
   required.
6. Generate fresh warning-denied `btpc-core` rustdoc into an isolated target path.
7. Copy rustdoc into the combined artifact.
8. Add canonical metadata, sitemap, route-compatibility redirects, and any required
   GitHub Pages metadata.
9. Run offline HTML, link, anchor, asset, privacy, route, and size-budget checks.
10. Atomically publish the completed staging tree to `site/`.

The builder must work from any current directory, remove stale staging output,
avoid developer-home state and editable imports, and leave no partially published
site when a stage fails.

## Test Migration

Port tests by contract rather than mechanically renaming assertions:

- Replace `mkdocs.yml` navigation tests with `book.toml` and strict `SUMMARY.md`
  coverage tests.
- Replace mkdocstrings HTML assumptions with preprocessor Markdown and rendered
  mdBook HTML assertions.
- Preserve public Python inventory, canonical anchor, private-symbol exclusion, CLI
  drift, rustdoc freshness, 404, canonical URL, sitemap, link, privacy, and artifact
  budget tests.
- Add preprocessor protocol tests, mdBook version checks, missing chapter failures,
  route redirect tests, and `mdbook test` execution.
- Keep the generated-site validator renderer-neutral where possible.
- Remove tests whose only purpose was verifying Material or MkDocs configuration.

The migration is complete only when no test, script, hook, workflow, contributor
command, or specification expects MkDocs.

## GitHub Actions Cutover

Keep the current least-privilege Pages architecture. Update only the build toolchain:

- Pull requests and pushes to `main` run the same canonical mdBook gate.
- CI installs the exact pinned mdBook version without changing the project MSRV.
- The build job retains read-only contents permission.
- Only the trusted deploy job receives `pages: write` and `id-token: write`.
- The official configure, artifact upload, and deploy Pages actions remain pinned to
  immutable revisions.
- Fork pull requests build but never deploy.
- The `github-pages` environment and production concurrency policy remain intact.

Before the first mdBook deployment, upload the artifact from a pull request or
manual non-deploy run and inspect it. After cutover, verify all canonical and legacy
routes against the live GitHub Pages site. Rollback is redeploying the last known
good MkDocs commit; do not introduce a `gh-pages` branch.

## Migration Sequence

1. Freeze a route, anchor, feature, and size baseline from the current MkDocs site.
2. Add pinned mdBook tooling, `book.toml`, and `SUMMARY.md` beside MkDocs.
3. Port content, navigation, theme behavior, and legacy-route redirects.
4. Implement the Griffe-backed Python API preprocessor.
5. Integrate CLI reference, Rust snippets, and embedded rustdoc.
6. Port the full generated-site quality gate and developer commands.
7. Switch CI and Pages builds to mdBook, then remove the MkDocs stack.
8. Deploy, verify production and legacy URLs, and update monitoring/runbooks.

## Removal Checklist

The final cleanup removes all of the following only after the mdBook gate passes:

- `mkdocs.yml`
- `docs/overrides/`
- Material-specific styles or template assumptions
- `mkdocs`, `mkdocs-material`, `mkdocstrings`, `mkdocstrings-python`, PyMdown, and
  dependencies needed only by those packages
- MkDocs-specific test helpers and YAML parsing dependencies if unused elsewhere
- `mkdocs`/`mkdocstrings` terminology in scripts, Make targets, hooks, workflows,
  contributor docs, specs, and agent guidance

Retain Griffe directly for Python source analysis unless a future replacement has
equivalent static typing and docstring support.

## Deferred Work

- Versioned documentation or multiple books.
- A custom domain.
- Analytics or telemetry.
- External hosted search.
- A fully forked mdBook theme.
- Public pull-request preview deployments.

## Primary References

- mdBook introduction and current version: <https://rust-lang.github.io/mdBook/>
- `SUMMARY.md` format: <https://rust-lang.github.io/mdBook/format/summary.html>
- General configuration: <https://rust-lang.github.io/mdBook/format/configuration/general.html>
- HTML renderer, redirects, search, and `site-url`: <https://rust-lang.github.io/mdBook/format/configuration/renderers.html>
- Preprocessors: <https://rust-lang.github.io/mdBook/for_developers/preprocessors.html>
- Continuous integration: <https://rust-lang.github.io/mdBook/continuous-integration.html>
- GitHub Pages custom workflows: <https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages>
