from __future__ import annotations

from pathlib import Path

import yaml

ROOT = Path(__file__).parents[2]


def _targets(items: list[object]) -> list[str]:
    targets: list[str] = []
    for item in items:
        assert isinstance(item, dict)
        value = next(iter(item.values()))
        if isinstance(value, list):
            targets.extend(_targets(value))
        else:
            assert isinstance(value, str)
            targets.append(value)
    return targets


def test_cli_reference_navigation_matches_generated_pages() -> None:
    config = yaml.safe_load((ROOT / "mkdocs.yml").read_text())
    cli = next(item["CLI"] for item in config["nav"] if "CLI" in item)
    reference = next(
        item["Command Reference"] for item in cli if "Command Reference" in item
    )
    nav_targets = set(_targets(reference))
    source_targets = {
        f"cli/reference/{page.name}"
        for page in (ROOT / "docs/cli/reference").glob("*.md")
    }
    assert nav_targets == source_targets
