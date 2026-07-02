.PHONY: check gate-summary specs spec-sync docs docs-generate docs-site docs-serve version test-rust test-python lint build-wheel benchmark-iso hooks-manual hooks-push install-hooks uninstall-hooks

check: specs docs lint test-rust test-python

gate-summary:
	uv run python scripts/write_gate_summary.py --output .tmp/gate-summary.json --status passed --command "make check"

specs:
	uv run python scripts/check_specs.py

spec-sync:
	@test -n "$(BASE)" || (echo "usage: make spec-sync BASE=<git-ref>" >&2; exit 2)
	uv run python scripts/check_spec_sync.py --base "$(BASE)"

docs:
	uv run python scripts/check_docs.py
	cargo test -p btpc-cli --test reference

docs-generate:
	./scripts/generate-cli-reference.sh

docs-site:
	uv run --group docs python scripts/build_docs_site.py --site-dir site

docs-serve:
	uv run --group docs mkdocs serve --strict --config-file mkdocs.yml

version:
	@test -n "$(VERSION)" || (echo "usage: make version VERSION=X.Y.Z" >&2; exit 2)
	uv run python scripts/set_version.py "$(VERSION)"

test-rust:
	cargo test --workspace

test-python:
	uv run pytest tests/python

lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	uv run ruff check .
	uv run ruff format --check .
	scripts/check_python_types.sh

build-wheel:
	uv run maturin build --release

benchmark-iso:
	@test -n "$(ISO)" || (echo "usage: make benchmark-iso ISO=/path/to/debian-13.5.0-amd64-DVD-1.iso" >&2; exit 2)
	uv run python benches/torrent_creation.py preflight "$(ISO)" --output benchmark-results/preflight.json

hooks-manual:
	uv run pre-commit run --all-files --hook-stage manual

hooks-push:
	uv run pre-commit run --all-files --hook-stage pre-push

install-hooks:
	uv run pre-commit install --hook-type pre-commit --hook-type pre-push

uninstall-hooks:
	uv run pre-commit uninstall --hook-type pre-commit
	uv run pre-commit uninstall --hook-type pre-push
