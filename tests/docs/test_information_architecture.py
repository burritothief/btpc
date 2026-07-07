from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).parents[2]
BUILDER = ROOT / "scripts/build_mdbook_site.py"
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
        self.redirect = False

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        """Record titles, headings, images, and runtime asset URLs."""
        values = dict(attrs)
        if tag == "title":
            self.title = True
        elif tag == "h1" and "menu-title" not in (values.get("class") or ""):
            self.h1_count += 1
        elif tag == "img" and not values.get("alt"):
            self.images_without_alt.append(values.get("src") or "<missing src>")
        elif tag == "meta" and values.get("name") == "btpc-redirect":
            self.redirect = True
        if tag == "script" and values.get("src"):
            self._record_runtime_asset(values["src"])
        if tag == "link" and values.get("rel") == "stylesheet" and values.get("href"):
            self._record_runtime_asset(values["href"])

    def _record_runtime_asset(self, target: str) -> None:
        if urlparse(target).scheme in {"http", "https"}:
            self.external_runtime_assets.append(target)


def test_navigation_and_required_pages_match_the_plan() -> None:
    summary = (ROOT / "docs/SUMMARY.md").read_text()
    top_level = re.findall(r"^- \[([^]]+)]", summary, re.MULTILINE)
    assert top_level == TOP_LEVEL
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
    config = tomllib.loads((ROOT / "book.toml").read_text())
    html = config["output"]["html"]
    assert html["default-theme"] == "light"
    assert html["preferred-dark-theme"] == "navy"
    assert html["search"]["enable"] is True
    assert html["additional-css"] == ["docs/stylesheets/mdbook.css"]
    assert "analytics" not in config

    css = (ROOT / "docs/stylesheets/mdbook.css").read_text()
    assert ":focus-visible" in css
    assert "prefers-reduced-motion" in css
    assert "btpc-development-notice" in css


def test_generated_html_has_accessible_structure_and_local_assets(
    tmp_path: Path,
) -> None:
    destination = tmp_path / "site"
    subprocess.run(  # noqa: S603
        [sys.executable, BUILDER, "--site-dir", destination],
        cwd=tmp_path,
        check=True,
    )

    not_found = (destination / "404.html").read_text()
    assert "Page not found" in not_found
    assert "The requested BTPC documentation page does not exist." in not_found
    assert '<base href="/btpc/">' in not_found
    for page in destination.rglob("*.html"):
        inspector = PageInspector()
        inspector.feed(page.read_text())
        assert inspector.title, page
        assert not inspector.external_runtime_assets, page
        relative = page.relative_to(destination)
        is_embedded_rustdoc = relative.parts[0] == "rust" and relative != Path(
            "rust/index.html"
        )
        is_utility = relative.as_posix() in {"print.html", "toc.html"}
        if not is_embedded_rustdoc and not is_utility and not inspector.redirect:
            assert inspector.h1_count == 1, page
            assert not inspector.images_without_alt, page

    homepage = (destination / "index.html").read_text()
    assert "Development documentation" in homepage
    assert 'name="viewport"' in homepage
    assert "mdbook-theme-list" in homepage
    assert "mdbook-search-toggle" in homepage
    assert "fa-copy" in homepage
