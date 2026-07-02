#!/usr/bin/env bash
set -euo pipefail

mapfile_command=(find python tests/python -type f \( -name '*.py' -o -name '*.pyi' \) ! -path 'tests/python/typing/negative.py' -print0)
files=()
while IFS= read -r -d '' file; do
  files+=("$file")
done < <("${mapfile_command[@]}")
uv run pyrefly check "${files[@]}"

negative_output=$(mktemp)
trap 'rm -f "$negative_output"' EXIT
if uv run pyrefly check tests/python/typing/negative.py >"$negative_output" 2>&1; then
  cat "$negative_output"
  echo "negative Pyrefly fixture unexpectedly passed" >&2
  exit 1
fi
cat "$negative_output"
grep -q 'INFO 3 errors' "$negative_output"

uv run pyright tests/python/typing/consumer.py
