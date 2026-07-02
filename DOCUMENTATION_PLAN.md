# BTPC Documentation Site Plan

## Decision Summary

BTPC will have one unified documentation website containing handwritten guides,
the CLI reference, the Python API reference, and the Rust API reference.

The recommended stack is:

| Concern | Choice |
| --- | --- |
| Main documentation site | Material for MkDocs |
| Python API generation | mkdocstrings-python |
| Rust API generation | rustdoc through `cargo doc` |
| CLI reference generation | Existing Clap-based generators |
| Hosting | GitHub Pages |
| Deployment | GitHub Actions |
| Documentation source | `docs/` in this repository |
| Rust release documentation | docs.rs after publishing |

## One Site, Two Native API Renderers

BTPC should not maintain unrelated Python and Rust documentation websites. Users
should have one obvious documentation destination with language-specific sections:

```text
BTPC Documentation
|-- Getting Started
|-- Concepts
|-- CLI Guide
|-- Python API
|   |-- btpc.creation
|   |-- btpc.metainfo
|   |-- btpc.verification
|   |-- btpc.types
|   `-- btpc.errors
`-- Rust API
    `-- btpc-core rustdoc
```

Each language should still use its native documentation generator. Python API
pages will use the MkDocs theme, while Rust API pages retain rustdoc's standard
presentation. Rustdoc provides better trait, implementation, feature, source-link,
and intra-doc-link handling than a language-neutral renderer.

## Proposed Repository Layout

```text
btpc/
|-- mkdocs.yml
|-- docs/
|   |-- index.md
|   |-- getting-started.md
|   |-- concepts/
|   |   |-- bittorrent-v1.md
|   |   |-- bittorrent-v2.md
|   |   `-- hybrid.md
|   |-- cli/
|   |   |-- index.md
|   |   `-- configuration.md
|   |-- python/
|   |   |-- index.md
|   |   `-- reference/
|   |       |-- creation.md
|   |       |-- metainfo.md
|   |       |-- verification.md
|   |       |-- types.md
|   |       `-- errors.md
|   `-- rust/
|       `-- index.md
|-- python/btpc/
|-- crates/btpc-core/
`-- .github/workflows/docs.yml
```

Keeping documentation beside the code lets an API implementation, its docstrings,
examples, and generated reference changes land in the same pull request.

## Main Site

Use Material for MkDocs for navigation, search, responsive presentation, syntax
highlighting, admonitions, and handwritten guides. MkDocs produces a static site,
so deployment does not require a server or database.

The main site should contain:

- Installation and build instructions.
- Getting-started tutorials for the CLI, Python, and Rust.
- BitTorrent v1, v2, and hybrid concepts.
- CLI workflows, configuration, presets, output formats, and shell completion.
- Python tutorials and generated API reference.
- A Rust API landing page linking into generated rustdoc.
- Performance and benchmarking methodology.
- Compatibility, release, and migration guidance.

## Python API Reference

Use `mkdocstrings-python` to generate API pages from the public Python modules,
annotations, signatures, and docstrings. The authoritative documentation should
live with the public implementation in:

- `btpc.creation`
- `btpc.metainfo`
- `btpc.verification`
- `btpc.types`
- `btpc.errors`

Common names remain re-exported from `btpc`, but each object has one canonical
defining module.

A reference page can remain intentionally small:

```markdown
# Creation

::: btpc.creation
    options:
      show_root_heading: true
      show_source: false
      members_order: source
      show_signature_annotations: true
      separate_signature: true
```

Documentation generation should use the public Python facade rather than private
PyO3 implementation classes. Build the extension before documentation generation
only when runtime import is required.

## Rust API Reference

Generate the Rust reference with rustdoc:

```console
RUSTDOCFLAGS="-D warnings" cargo doc -p btpc-core --no-deps
```

Public Rust modules and items should use `//!` and `///` documentation with:

- Concise purpose and behavior.
- Errors and panic behavior.
- Safety and resource guarantees where relevant.
- Executable examples and doctests.
- Intra-doc links to related types and operations.

The documentation workflow will copy generated rustdoc from `target/doc` into the
final static site under a stable location such as:

```text
site/rust/btpc_core/index.html
```

After `btpc-core` is published to crates.io, docs.rs will provide version-specific
release documentation automatically. The BTPC site may continue embedding the
documentation for `main` while linking released versions to docs.rs.

## CLI Reference

Continue generating CLI reference material from the single Clap command model.
Generated command help, shell completions, and the manpage must not be maintained by
hand. The documentation build should either generate these artifacts directly or
verify that checked-in generated artifacts have no drift.

The main documentation site should provide task-oriented CLI guides in addition to
the complete generated command reference.

## Generated and Handwritten Content

Use handwritten documentation for:

- Tutorials and workflows.
- Installation and packaging.
- Protocol concepts.
- Performance guidance.
- Choosing between the Rust, Python, and CLI surfaces.
- Migration and compatibility notes.

Use generated documentation for:

- Python modules, classes, functions, properties, and signatures.
- Rust types, traits, methods, implementations, and doctests.
- CLI commands, options, aliases, completions, and manpage content.

Generated API references explain what exists. Handwritten guides explain how and
why to use it.

## Hosting

Host the generated static site with GitHub Pages. The initial project URL will be
similar to:

```text
https://burritothief.github.io/btpc/
```

A custom domain can be added later without changing the documentation architecture.
GitHub Pages is sufficient because the complete output is static HTML and assets.

## GitHub Actions

Use separate checking and deployment responsibilities.

### Pull Request Documentation Check

The pull-request workflow should:

1. Install the locked Rust and Python toolchains.
2. Install documentation dependencies.
3. Build the Python extension if API collection requires it.
4. Build MkDocs in strict mode.
5. Build rustdoc with warnings denied.
6. Run Rust doctests.
7. Generate or drift-check CLI reference artifacts.
8. Check internal links and required pages.

The check should fail on broken links, missing imports, invalid docstrings, rustdoc
warnings, failing doctests, missing navigation pages, or generated-reference drift.

### Main Branch Deployment

The deployment workflow should:

1. Build the MkDocs site.
2. Build rustdoc.
3. Copy rustdoc into the MkDocs output directory under `rust/`.
4. Upload the combined static site with the official Pages artifact action.
5. Deploy it with the official GitHub Pages deployment action.

Keep deployment permissions isolated from normal CI. Ordinary test workflows need
read-only repository permissions; only the Pages deployment job needs Pages and
identity-token permissions.

## Documentation Dependencies

Add a dedicated documentation dependency group rather than mixing site tooling into
runtime dependencies. The expected packages are:

```text
mkdocs
mkdocs-material
mkdocstrings[python]
```

Pin them through the existing lockfile workflow. Documentation tooling must not
become a dependency of the installed `btpc` Python package.

## Versioning Strategy

Initially publish documentation for the current `main` branch only. Avoid adding a
documentation-version manager before the first public release.

After releases stabilize:

- Let docs.rs host versioned Rust crate documentation.
- Consider versioned Python/CLI paths such as `/latest/`, `/0.1/`, and `/0.2/`.
- Keep the site root pointed at the latest stable or clearly labeled development
  documentation.

Versioning should be introduced only after there is more than one supported release
line.

## Implementation Sequence

1. Add the documentation dependency group and `mkdocs.yml`.
2. Create the site navigation and landing pages.
3. Add Python reference pages for each public domain module.
4. Improve public Python docstrings until strict generation passes.
5. Enable strict rustdoc and fill missing public Rust documentation.
6. Integrate generated CLI reference pages.
7. Add the pull-request documentation check.
8. Add the GitHub Pages build and deployment workflow.
9. Enable Pages for the repository and verify the published URL.
10. Add versioning only after the first stable release requires it.

## Reference Documentation

- Material for MkDocs: <https://squidfunk.github.io/mkdocs-material/>
- mkdocstrings: <https://mkdocstrings.github.io/>
- Cargo `doc`: <https://doc.rust-lang.org/cargo/commands/cargo-doc.html>
- GitHub Pages: <https://docs.github.com/pages>
- docs.rs builds: <https://docs.rs/about/builds>
