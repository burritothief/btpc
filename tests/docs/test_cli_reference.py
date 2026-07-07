from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).parents[2]
TARGET = re.compile(r"\((cli/reference/[^)]+\.md)\)")


def test_cli_reference_navigation_matches_generated_pages() -> None:
    summary = (ROOT / "docs/SUMMARY.md").read_text()
    nav_targets = set(TARGET.findall(summary))
    source_targets = {
        f"cli/reference/{page.name}"
        for page in (ROOT / "docs/cli/reference").glob("*.md")
    }
    assert nav_targets == source_targets
