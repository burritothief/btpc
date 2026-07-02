#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  pre-push)
    uv run maturin develop
    scripts/check_python_types.sh
    uv run python scripts/check_native_stub.py
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo nextest run --workspace --all-features
    cargo test --workspace --doc
    uv run pytest tests/python
    cargo deny check
    make docs-check
    ;;
  manual)
    uv run python scripts/check_specs.py
    make docs-check
    uv run python scripts/render_benchmark_fixture.py
    uv run actionlint .github/workflows/*.yml
    uv run zizmor --offline --persona=regular --min-severity=medium .github
    cargo check --manifest-path tests/rust-consumer/Cargo.toml
    ;;
  *)
    echo "usage: $0 {pre-push|manual}" >&2
    exit 2
    ;;
esac
