from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parents[1]


def main() -> int:
    parser = argparse.ArgumentParser(description="Build the side-by-side mdBook site")
    parser.add_argument("--site-dir", type=Path, required=True)
    arguments = parser.parse_args()
    subprocess.run(
        [sys.executable, ROOT / "scripts/check_mdbook.py"], cwd=ROOT, check=True
    )
    destination = (ROOT / arguments.site_dir).resolve()
    shutil.rmtree(destination, ignore_errors=True)
    subprocess.run(
        ["mdbook", "build", str(ROOT), "--dest-dir", str(destination)],
        cwd=ROOT,
        check=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
