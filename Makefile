.PHONY: check gate-summary specs spec-sync docs docs-fast docs-check docs-health docs-generate docs-site docs-serve version test-rust test-python lint build-wheel benchmark-iso hooks-manual hooks-push install-hooks uninstall-hooks

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

docs-fast:
	uv run python scripts/check_docs.py
	uv run pytest tests/docs/test_site_quality.py tests/docs/test_cli_reference.py -q

docs-check:
	cargo test -p btpc-core --doc
	uv run pytest tests/docs -q
	$(MAKE) docs-site
	uv run python scripts/check_docs_site.py site
	uv run python scripts/docs_renderer_baseline.py compare --site-dir site --manifest tests/docs/fixtures/renderer_migration_baseline.json
	uv run python scripts/check_docs.py
	uv run codespell README.md CONTRIBUTING.md SECURITY.md CHANGELOG.md DOCUMENTATION_PLAN.md AGENTS.md docs specs --skip='docs/completions/*,docs/reference/*,docs/cli/reference/*'

docs-health:
	uv run python scripts/collect_docs_external_links.py
	@set +e; \
	lychee --config .lychee.toml --format markdown --output .tmp/docs-link-health.md .tmp/docs-external-links.md; link_status=$$?; \
	uv run python scripts/check_docs_health.py; live_status=$$?; \
	test $$link_status -eq 0 -a $$live_status -eq 0

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
