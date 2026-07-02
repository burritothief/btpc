from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).parents[1]
MARKDOWN_LINK = re.compile(r"(?<!!)\[[^]]+\]\(([^)]+)\)")
SKIP_PREFIXES = ("http://", "https://", "mailto:", "#")


def markdown_files() -> list[Path]:
    files = [ROOT / "README.md", ROOT / "CONTRIBUTING.md"]
    files.extend((ROOT / "docs").rglob("*.md"))
    files.extend((ROOT / "specs").rglob("*.md"))
    files.extend((ROOT / "benches").rglob("*.md"))
    return sorted(path for path in files if path.is_file())


def main() -> int:
    errors: list[str] = []
    for document in markdown_files():
        for line_number, line in enumerate(document.read_text().splitlines(), 1):
            for raw_target in MARKDOWN_LINK.findall(line):
                target = raw_target.split("#", 1)[0]
                if not target or target.startswith(SKIP_PREFIXES):
                    continue
                resolved = (document.parent / target).resolve()
                if not resolved.exists():
                    relative = document.relative_to(ROOT)
                    errors.append(f"{relative}:{line_number}: missing {raw_target}")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"validated links in {len(markdown_files())} Markdown files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
