"""Require contract-bearing source changes to update their owning specifications."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tomllib

from check_specs import ROOT, load_specifications

MINIMUM_WAIVER_LENGTH = 20


def changed_paths(base: str) -> set[str]:
    result = subprocess.run(
        ["git", "diff", "--name-only", f"{base}...HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return {line for line in result.stdout.splitlines() if line}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", required=True, help="base commit or ref")
    args = parser.parse_args()

    changed = changed_paths(args.base)
    ownership = tomllib.loads((ROOT / "specs/ownership.toml").read_text())
    path_to_specs = {
        entry["path"]: set(entry["specs"]) for entry in ownership["ownership"]
    }
    spec_paths = {
        spec.spec_id: spec.path.relative_to(ROOT).as_posix()
        for spec in load_specifications()
    }
    required_specs = set().union(*(path_to_specs.get(path, set()) for path in changed))
    missing = {
        spec_id for spec_id in required_specs if spec_paths[spec_id] not in changed
    }
    if not missing:
        print("source/spec synchronization check passed")
        return 0

    waiver = os.environ.get("SPEC_SYNC_WAIVER", "")
    marker = "Spec-Sync-Waiver:"
    reason = waiver.partition(marker)[2].strip() if marker in waiver else ""
    if len(reason) >= MINIMUM_WAIVER_LENGTH:
        print(f"source/spec synchronization waived: {reason}")
        return 0
    print(
        "contract-bearing source changed without its owning specs: "
        f"{sorted(missing)}; update the specs or add '{marker} <reason>' "
        "with at least 20 characters to the pull request body",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
