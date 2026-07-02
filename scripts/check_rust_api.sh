#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <baseline-revision>" >&2
  exit 2
fi

cargo semver-checks check-release \
  --package btpc-core \
  --baseline-rev "$1"
