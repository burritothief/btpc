# Contributing

Run the same focused gates used by CI before opening a pull request:

```console
make check
make docs-site
```

Use `make docs-serve` to preview the project-subpath site. Keep generated `site/`
output untracked and update source docs, Python docstrings, Rust rustdoc, or the CLI
command model instead of hand-editing generated HTML.

Implementation contracts are maintained separately for contributors in the
[normative specifications][specs]. Public guides describe supported behavior rather
than requirement identifiers.

[specs]: https://github.com/burritothief/btpc/tree/main/specs
