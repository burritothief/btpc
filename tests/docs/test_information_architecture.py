from __future__ import annotations

import subprocess
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlparse

import yaml

ROOT = Path(__file__).parents[2]
CONFIG = ROOT / "mkdocs.yml"
BUILDER = ROOT / "scripts/build_docs_site.py"
TOP_LEVEL = [
    "Home",
    "Getting Started",
    "Guides",
    "Concepts",
    "CLI",
    "Python",
    "Rust",
    "Performance",
    "Compatibility",
    "Security",
    "Contributing",
]
PALETTE_COUNT = 2
REQUIRED_PAGES = [
    "getting-started/index.md",
    "getting-started/installation.md",
    "getting-started/cli.md",
    "getting-started/python.md",
    "getting-started/rust.md",
    "guides/index.md",
    "guides/creating.md",
    "guides/inspecting-validating.md",
    "guides/verifying.md",
    "guides/editing.md",
    "guides/configuration-presets.md",
    "cli/index.md",
    "cli/configuration.md",
    "cli/completion.md",
    "python/index.md",
    "rust/index.md",
    "concepts/index.md",
    "concepts/v1.md",
    "concepts/v2.md",
    "concepts/hybrid.md",
    "concepts/piece-length.md",
    "concepts/reproducibility-bytes.md",
    "performance.md",
    "compatibility.md",
    "security.md",
    "contributing.md",
]


class PageInspector(HTMLParser):
    """Collect structural and runtime-asset facts from generated HTML."""

    def __init__(self) -> None:
        """Initialize an empty page inspection."""
        super().__init__()
        self.title = False
        self.h1_count = 0
        self.images_without_alt: list[str] = []
        self.external_runtime_assets: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        """Record titles, headings, images, and runtime asset URLs."""
        values = dict(attrs)
        if tag == "title":
            self.title = True
        elif tag == "h1":
            self.h1_count += 1
        elif tag == "img" and not values.get("alt"):
            self.images_without_alt.append(values.get("src") or "<missing src>")
        if tag == "script" and values.get("src"):
            self._record_runtime_asset(values["src"])
        if tag == "link" and values.get("rel") == "stylesheet" and values.get("href"):
            self._record_runtime_asset(values["href"])

    def _record_runtime_asset(self, target: str) -> None:
        if urlparse(target).scheme in {"http", "https"}:
            self.external_runtime_assets.append(target)


def test_navigation_and_required_pages_match_the_plan() -> None:
    config = yaml.safe_load(CONFIG.read_text())
    assert [next(iter(item)) for item in config["nav"]] == TOP_LEVEL
    for relative in REQUIRED_PAGES:
        page = ROOT / "docs" / relative
        assert page.is_file(), relative
        headings: list[str] = []
        in_fence = False
        for line in page.read_text().splitlines():
            if line.startswith("```"):
                in_fence = not in_fence
            elif not in_fence and line.startswith("# "):
                headings.append(line)
        assert len(headings) == 1, relative


def test_theme_is_accessible_private_and_self_contained() -> None:
    config = yaml.safe_load(CONFIG.read_text())
    theme = config["theme"]
    assert theme["font"] is False
    assert "content.code.copy" in theme["features"]
    assert len(theme["palette"]) == PALETTE_COUNT
    assert all("toggle" in palette for palette in theme["palette"])
    assert config["extra_css"] == ["stylesheets/extra.css"]
    assert "analytics" not in config

    css = (ROOT / "docs/stylesheets/extra.css").read_text()
    assert ":focus-visible" in css
    assert "prefers-reduced-motion" in css
    override = (ROOT / "docs/overrides/main.html").read_text()
    assert "Development documentation" in override


def test_generated_html_has_accessible_structure_and_local_assets(
    tmp_path: Path,
) -> None:
    destination = tmp_path / "site"
    subprocess.run(  # noqa: S603
        [sys.executable, str(BUILDER), "--site-dir", str(destination)],
        cwd=tmp_path,
        check=True,
    )

    not_found = (destination / "404.html").read_text()
    assert "Page not found" in not_found
    assert "The requested BTPC documentation page does not exist." in not_found
    for page in destination.rglob("*.html"):
        inspector = PageInspector()
        inspector.feed(page.read_text())
        assert inspector.title, page
        assert inspector.h1_count == 1, page
        assert not inspector.images_without_alt, page
        assert not inspector.external_runtime_assets, page

    homepage = (destination / "index.html").read_text()
    assert "Development documentation" in homepage
    assert 'name="viewport"' in homepage
    assert 'data-md-component="palette"' in homepage
    assert 'data-md-component="search"' in homepage
    assert 'data-md-component="announce"' in homepage
    assert '"content.code.copy"' in homepage
