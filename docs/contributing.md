# Contributing

Run the same focused gates used by CI before opening a pull request:

```console
make check
make docs-check
```

Use `make docs-serve` to preview the project-subpath site. Handwritten pages live
in `docs/`; generated Python and Rust API text comes from source docstrings and
rustdoc, while CLI references come from the Clap model. Keep generated `site/`
output untracked and never hand-edit generated CLI files or HTML.

`make docs-check` enforces offline links and anchors, canonical URLs, required
entry points, private-name protection, and budgets of 16,000,000 uncompressed
bytes and 4,500,000 normalized gzip bytes. Budget increases require recorded
artifact measurements.

Implementation contracts are maintained separately for contributors in the
[normative specifications][specs]. Public guides describe supported behavior rather
than requirement identifiers.

[specs]: https://github.com/burritothief/btpc/tree/main/specs
