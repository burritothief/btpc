---
title: Documentation release checklist
---

# Documentation release checklist

Before publishing a release or announcing documentation:

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
