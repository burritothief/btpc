from __future__ import annotations

from pathlib import Path

import yaml

ROOT = Path(__file__).parents[2]
HOOK_STAGE_COUNT = 2


def test_canonical_docs_gate_and_hooks_share_commands() -> None:
    makefile = (ROOT / "Makefile").read_text()
    assert "docs-check:" in makefile
    assert "scripts/check_docs_site.py site" in makefile
    assert "cargo test -p btpc-core --doc" in makefile
    assert "uv run pytest tests/docs -q" in makefile

    hooks = (ROOT / "scripts/run-hook-stage.sh").read_text()
    assert hooks.count("make docs-check") == HOOK_STAGE_COUNT
    config = yaml.safe_load((ROOT / ".pre-commit-config.yaml").read_text())
    local = next(
        repository for repository in config["repos"] if repository["repo"] == "local"
    )
    docs_fast = next(hook for hook in local["hooks"] if hook["id"] == "docs-fast")
    assert docs_fast["entry"] == "make docs-fast"


def test_documented_site_budgets_match_the_validator() -> None:
    checker = (ROOT / "scripts/check_docs_site.py").read_text()
    contributing = (ROOT / "CONTRIBUTING.md").read_text()
    for budget in ("16_000_000", "4_500_000"):
        assert budget in checker
        assert budget.replace("_", ",") in contributing
