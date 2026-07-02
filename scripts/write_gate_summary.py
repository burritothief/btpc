from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from datetime import UTC, datetime
from pathlib import Path

EXCLUDED_PARTS = {
    ".git",
    ".pytest_cache",
    ".ruff_cache",
    ".tmp",
    ".venv",
    "benchmark-data",
    "benchmark-results",
    "dist",
    "target",
}


def source_revision(root: Path) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "--verify", "HEAD"],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip() if result.returncode == 0 else "no-commit"


def tree_digest(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(root.rglob("*")):
        relative = path.relative_to(root)
        if not path.is_file() or any(part in EXCLUDED_PARTS for part in relative.parts):
            continue
        digest.update(relative.as_posix().encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--status", choices=["passed", "failed"], required=True)
    parser.add_argument("--command", action="append", default=[])
    arguments = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    summary = {
        "schema_version": 1,
        "status": arguments.status,
        "source_revision": source_revision(root),
        "source_tree_sha256": tree_digest(root),
        "generated_at": datetime.now(UTC).isoformat(),
        "commands": arguments.command,
    }
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    print(arguments.output)


if __name__ == "__main__":
    main()
