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
