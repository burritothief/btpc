from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).parents[2]
BOOK = ROOT / "book.toml"
SUMMARY = ROOT / "docs/SUMMARY.md"
VERSION = ROOT / ".mdbook-version"
CHECKSUM = ROOT / ".mdbook-sha256"
HEADING_SPLIT_LEVEL = 3


def test_mdbook_configuration_is_pinned_private_and_side_by_side() -> None:
    config = tomllib.loads(BOOK.read_text())
    assert VERSION.read_text().strip() == "0.5.3"
    assert CHECKSUM.read_text().strip() == (
        "742264af649df2323b283a4c1a8abc21b6f6880cf030d642500ef85c2ce81598"
    )
    assert config["book"]["src"] == "docs"
    assert config["build"]["create-missing"] is False
    assert config["rust"]["edition"] == "2024"
    html = config["output"]["html"]
    assert html["site-url"] == "/btpc/"
    assert html["input-404"] == "404.md"
    assert html["git-repository-url"] == "https://github.com/burritothief/btpc"
    assert "edit/main/{path}" in html["edit-url-template"]
    assert html["additional-css"] == ["docs/stylesheets/mdbook.css"]
    assert html["search"]["enable"] is True
    assert html["search"]["heading-split-level"] == HEADING_SPLIT_LEVEL
    assert "analytics" not in html
    assert config["build"]["extra-watch-dirs"] == [
        "python/btpc",
        "crates/btpc-cli/src",
        "crates/btpc-core/src",
    ]


def test_summary_lists_every_public_chapter_exactly_once() -> None:
    links = re.findall(r"\[[^]]+\]\(([^)]+\.md)\)", SUMMARY.read_text())
    assert len(links) == len(set(links))
    expected = {
        path.relative_to(ROOT / "docs").as_posix()
        for path in (ROOT / "docs").rglob("*.md")
        if path.name not in {"404.md", "SUMMARY.md"}
    }
    assert set(links) == expected
    assert all((ROOT / "docs" / link).is_file() for link in links)


def test_mdbook_version_check_has_actionable_missing_tool_error(tmp_path: Path) -> None:
    missing = tmp_path / "missing-mdbook"
    result = subprocess.run(  # noqa: S603
        [sys.executable, ROOT / "scripts/check_mdbook.py", "--mdbook", missing],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 1
    assert "cargo install mdbook --version 0.5.3 --locked" in result.stderr

    wrong = subprocess.run(  # noqa: S603
        [sys.executable, ROOT / "scripts/check_mdbook.py", "--mdbook", sys.executable],
        check=False,
        capture_output=True,
        text=True,
    )
    assert wrong.returncode == 1
    assert "expected mdbook v0.5.3" in wrong.stderr


def test_makefile_keeps_mdbook_build_explicit_and_temporary() -> None:
    makefile = (ROOT / "Makefile").read_text()
    assert "docs-mdbook-site:" in makefile
    assert "scripts/build_mdbook_site.py --site-dir .tmp/mdbook-site" in makefile
