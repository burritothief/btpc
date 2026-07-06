from __future__ import annotations

import argparse
import re
from collections import defaultdict
from html.parser import HTMLParser
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterable

ROOT = Path(__file__).parents[1]
DEFAULT_OUTPUT = ROOT / ".tmp/docs-external-links.md"
TOP_LEVEL_SOURCES = (
    "README.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "CHANGELOG.md",
    "DOCUMENTATION_PLAN.md",
)
MARKDOWN_URL = re.compile(r"https?://[^\s<>\"'`)]+")


class _HtmlLinks(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.urls: set[str] = set()

    def handle_starttag(self, _tag: str, attrs: list[tuple[str, str | None]]) -> None:
        for name, value in attrs:
            if (
                name in {"href", "src", "action"}
                and value is not None
                and value.startswith(("https://", "http://"))
            ):
                self.urls.add(value)


def _urls(path: Path) -> set[str]:
    source = path.read_text(errors="replace")
    if path.suffix == ".html":
        parser = _HtmlLinks()
        parser.feed(source)
        return parser.urls
    return {match.group().rstrip(".,;:") for match in MARKDOWN_URL.finditer(source)}


def collect(paths: Iterable[Path], *, root: Path) -> dict[str, tuple[str, ...]]:
    """Collect absolute links with deterministic source attribution."""
    sources: dict[str, set[str]] = defaultdict(set)
    for path in sorted(paths):
        relative = path.relative_to(root).as_posix()
        for url in _urls(path):
            sources[url].add(relative)
    return {
        url: tuple(sorted(url_sources)) for url, url_sources in sorted(sources.items())
    }


def render(links: dict[str, tuple[str, ...]]) -> str:
    """Render a compact Markdown input that keeps source names visible."""
    lines = ["# Documentation external-link inventory", ""]
    for url, sources in links.items():
        source_list = ", ".join(f"`{source}`" for source in sources)
        lines.append(f"- [check]({url}) — sources: {source_list}")
    return "\n".join(lines) + "\n"


def _default_paths() -> tuple[Path, ...]:
    paths = [ROOT / relative for relative in TOP_LEVEL_SOURCES]
    paths.extend((ROOT / "docs").rglob("*.md"))
    site = ROOT / "site"
    paths.extend(
        path
        for path in site.rglob("*.html")
        if path.relative_to(site).parts[:1] != ("rust",)
        or path.relative_to(site).as_posix() == "rust/index.html"
    )
    return tuple(paths)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Collect external links from source and generated documentation"
    )
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    return parser


def main(argv: list[str] | None = None) -> int:
    """Write the deterministic external-link inventory."""
    arguments = _parser().parse_args(argv)
    links = collect(_default_paths(), root=ROOT)
    if not links:
        raise RuntimeError("documentation external-link inventory is empty")
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(render(links))
    print(f"collected {len(links)} external documentation URLs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
