from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parents[1]
VERSION = (ROOT / ".mdbook-version").read_text().strip()
INSTALL = f"cargo install mdbook --version {VERSION} --locked"


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate the pinned mdBook tool")
    parser.add_argument("--mdbook", type=Path, default=Path("mdbook"))
    arguments = parser.parse_args()
    try:
        result = subprocess.run(
            [str(arguments.mdbook), "--version"],
            check=True,
            capture_output=True,
            text=True,
        )
    except (FileNotFoundError, subprocess.CalledProcessError):
        print(
            f"mdBook {VERSION} is required; install it with: {INSTALL}", file=sys.stderr
        )
        return 1
    actual = result.stdout.strip()
    if actual != f"mdbook v{VERSION}":
        print(
            f"expected mdbook v{VERSION}, found {actual!r}; install it with: {INSTALL}",
            file=sys.stderr,
        )
        return 1
    print(actual)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
