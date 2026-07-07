from __future__ import annotations

import json
import re
import subprocess
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlparse

ROOT = Path(__file__).parents[2]
BASELINE = ROOT / "tests/docs/fixtures/renderer_migration_baseline.json"
BUILDER = ROOT / "scripts/build_mdbook_site.py"


class _PageInspector(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.ids: set[str] = set()
        self.redirect: str | None = None
        self.canonical: str | None = None
        self.links: list[str] = []

    def handle_starttag(
        self, tag: str, attributes: list[tuple[str, str | None]]
    ) -> None:
        values = dict(attributes)
        if identifier := values.get("id"):
            self.ids.add(identifier)
        if tag == "meta" and values.get("name") == "btpc-redirect":
            self.redirect = values.get("content")
        if tag == "link" and values.get("rel") == "canonical":
            self.canonical = values.get("href")
        if tag == "a" and values.get("href"):
            self.links.append(values["href"])


def _inspect(path: Path) -> _PageInspector:
    inspector = _PageInspector()
    inspector.feed(path.read_text())
    return inspector


def test_handwritten_markdown_is_renderer_neutral() -> None:
    for path in (ROOT / "docs").rglob("*.md"):
        text = path.read_text()
        if text.startswith("---\n"):
            end = text.find("\n---\n", 4)
            assert end >= 0, path
            text = text[end + 5 :]
        assert not re.search(r"^!!! |^\?\?\? |^=== ", text, re.MULTILINE), path
        headings: list[str] = []
        in_fence = False
        for line in text.splitlines():
            if line.startswith("```"):
                in_fence = not in_fence
            elif not in_fence and line.startswith("# "):
                headings.append(line)
        if path.name != "SUMMARY.md":
            assert len(headings) == 1, path


def test_mdbook_port_preserves_features_and_non_rust_routes(tmp_path: Path) -> None:
    site = tmp_path / "site"
    subprocess.run(  # noqa: S603
        [sys.executable, BUILDER, "--site-dir", site],
        cwd=tmp_path,
        check=True,
    )
    homepage = (site / "index.html").read_text()
    assert 'class="btpc-development-notice"' in homepage
    assert "current <code>main</code> branch" in homepage
    assert (
        "title: Getting started"
        not in (site / "getting-started/index.html").read_text()
    )
    assert "mdbook-theme-list" in homepage
    assert "mdbook-search-toggle" in homepage
    assert "fa-copy" in homepage
    assert (site / "print.html").is_file()
    assert (site / "404.html").is_file()

    baseline = json.loads(BASELINE.read_text())
    for route in baseline["routes"]:
        relative = route["path"]
        if relative.startswith("rust/") and relative != "rust/index.html":
            continue
        target = site / relative
        assert target.is_file(), relative
        inspected = _inspect(target)
        if inspected.redirect is None:
            continue
        assert inspected.redirect.startswith("/btpc/"), relative
        assert inspected.redirect != f"/btpc/{relative}", relative
        redirect_target = site / inspected.redirect.removeprefix("/btpc/")
        assert redirect_target.is_file(), relative
        assert _inspect(redirect_target).redirect is None, relative

        expected_canonical = baseline["canonical_urls"].get(relative)
        if expected_canonical is not None:
            assert inspected.canonical == expected_canonical, relative


def test_important_fragments_resolve_after_one_redirect(tmp_path: Path) -> None:
    site = tmp_path / "site"
    subprocess.run(  # noqa: S603
        [sys.executable, BUILDER, "--site-dir", site],
        cwd=tmp_path,
        check=True,
    )
    baseline = json.loads(BASELINE.read_text())
    for route, anchors in baseline["anchors"].items():
        if route.startswith(("rust/", "python/reference/")):
            continue
        page = _inspect(site / route)
        target = (
            site / page.redirect.removeprefix("/btpc/")
            if page.redirect is not None
            else site / route
        )
        ids = _inspect(target).ids
        assert set(anchors) <= ids, route


def test_handwritten_and_cli_rendered_links_resolve(tmp_path: Path) -> None:
    site = tmp_path / "site"
    subprocess.run(  # noqa: S603
        [sys.executable, BUILDER, "--site-dir", site],
        cwd=tmp_path,
        check=True,
    )
    for page in site.rglob("*.html"):
        relative = page.relative_to(site).as_posix()
        if relative == "print.html" or "/index.html" in relative:
            continue
        inspector = _inspect(page)
        for link in inspector.links:
            parsed = urlparse(link)
            if parsed.scheme or parsed.netloc or link.startswith(("mailto:", "#")):
                continue
            path = unquote(parsed.path)
            if path.startswith("/btpc/"):
                target = site / path.removeprefix("/btpc/")
            else:
                target = (page.parent / path).resolve()
            if path.endswith("/"):
                target /= "index.html"
            assert target.is_file(), f"{relative}: {link}"
            if not parsed.fragment:
                continue
            target_relative = target.relative_to(site).as_posix()
            if target_relative.startswith("python/reference/"):
                continue
            assert parsed.fragment in _inspect(target).ids, f"{relative}: {link}"
