from __future__ import annotations

import argparse
import gzip
import re
import sys
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path, PurePosixPath
from urllib.parse import unquote, urljoin, urlparse

ROOT = Path(__file__).parents[1]
SITE_BASE = "https://burritothief.github.io/btpc/"
SITE_PATH = "/btpc/"
MAX_UNCOMPRESSED_BYTES = 16_000_000
MAX_COMPRESSED_BYTES = 4_500_000
REQUIRED_ENTRIES = (
    "index.html",
    "404.html",
    "python/index.html",
    "python/reference/creation/index.html",
    "cli/reference/index.html",
    "rust/index.html",
    "rust/btpc_core/index.html",
)
PRIVATE_PYTHON_ID = re.compile(r"\bid=[\"']btpc\._(?:native|conversion)")


@dataclass(frozen=True)
class Reference:
    """One generated HTML URL reference."""

    target: str
    runtime_asset: bool


@dataclass(frozen=True)
class Page:
    """Facts collected from one generated HTML page."""

    title: str
    ids: frozenset[str]
    canonical: tuple[str, ...]
    references: tuple[Reference, ...]


@dataclass(frozen=True)
class SiteReport:
    """Deterministic generated-site inventory and sizes."""

    files: int
    html_files: int
    uncompressed_bytes: int
    compressed_bytes: int


class PageParser(HTMLParser):
    """Collect titles, anchors, canonical URLs, and local references."""

    def __init__(self) -> None:
        """Initialize an empty page parser."""
        super().__init__()
        self._in_title = False
        self._title: list[str] = []
        self.ids: set[str] = set()
        self.canonical: list[str] = []
        self.references: list[Reference] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        """Record structural metadata and referenced URLs."""
        values = dict(attrs)
        if tag == "title":
            self._in_title = True
        if identifier := values.get("id"):
            self.ids.add(identifier)

        relation = set((values.get("rel") or "").split())
        href = values.get("href")
        if tag == "link" and "canonical" in relation and href:
            self.canonical.append(href)
        elif href:
            self.references.append(
                Reference(href, tag == "link" and "stylesheet" in relation)
            )
        if source := values.get("src"):
            self.references.append(Reference(source, runtime_asset=True))

    def handle_endtag(self, tag: str) -> None:
        """Stop title collection at the closing element."""
        if tag == "title":
            self._in_title = False

    def handle_data(self, data: str) -> None:
        """Collect visible title fragments."""
        if self._in_title:
            self._title.append(data)

    def page(self) -> Page:
        """Return normalized facts for the parsed page."""
        title = " ".join(" ".join(self._title).split())
        return Page(
            title,
            frozenset(self.ids),
            tuple(self.canonical),
            tuple(self.references),
        )


def _public_path(relative: PurePosixPath) -> str:
    value = relative.as_posix()
    if value == "index.html":
        return ""
    if value.endswith("/index.html"):
        return f"{value.removesuffix('index.html')}"
    return value


def _artifact_target(
    site: Path, page: PurePosixPath, target: str
) -> tuple[Path, str] | str | None:
    parsed = urlparse(target)
    hostname = (parsed.hostname or "").lower()
    if parsed.scheme == "file":
        result: tuple[Path, str] | str | None = f"forbidden URL scheme in {target}"
    elif parsed.scheme == "javascript":
        result = None
    elif parsed.scheme in {"http", "https"}:
        if hostname in {"localhost", "127.0.0.1", "::1"}:
            result = f"localhost URL in {target}"
        else:
            result = None
    elif parsed.scheme or parsed.netloc:
        result = None
    else:
        base = urljoin(SITE_BASE, _public_path(page))
        resolved = urlparse(urljoin(base, target))
        if not resolved.path.startswith(SITE_PATH):
            result = f"URL escapes /btpc/: {target}"
        else:
            relative_url = unquote(resolved.path.removeprefix(SITE_PATH))
            candidate = site / relative_url
            if resolved.path.endswith("/") or candidate.is_dir():
                candidate /= "index.html"
            result = candidate, unquote(resolved.fragment)
    return result


def _is_embedded_rustdoc(relative: PurePosixPath) -> bool:
    return relative.parts[:1] == ("rust",) and relative != PurePosixPath(
        "rust/index.html"
    )


def _site_report(site: Path, files: list[Path]) -> SiteReport:
    payload = bytearray()
    total = 0
    for path in files:
        content = path.read_bytes()
        relative = path.relative_to(site).as_posix().encode()
        total += len(content)
        payload.extend(relative)
        payload.extend(b"\0")
        payload.extend(str(len(content)).encode())
        payload.extend(b"\0")
        payload.extend(content)
    compressed = gzip.compress(bytes(payload), compresslevel=9, mtime=0)
    return SiteReport(
        files=len(files),
        html_files=sum(path.suffix == ".html" for path in files),
        uncompressed_bytes=total,
        compressed_bytes=len(compressed),
    )


def _budget_errors(
    report: SiteReport, max_uncompressed: int, max_compressed: int
) -> list[str]:
    errors: list[str] = []
    if report.uncompressed_bytes > max_uncompressed:
        errors.append(
            "uncompressed size budget exceeded: "
            f"{report.uncompressed_bytes} > {max_uncompressed} bytes"
        )
    if report.compressed_bytes > max_compressed:
        errors.append(
            "compressed size budget exceeded: "
            f"{report.compressed_bytes} > {max_compressed} bytes"
        )
    return errors


def _load_pages(
    site: Path, files: list[Path]
) -> tuple[dict[PurePosixPath, Page], dict[PurePosixPath, str]]:
    pages: dict[PurePosixPath, Page] = {}
    sources: dict[PurePosixPath, str] = {}
    for path in files:
        if path.suffix != ".html":
            continue
        relative = PurePosixPath(path.relative_to(site).as_posix())
        source = path.read_text(errors="replace")
        parser = PageParser()
        parser.feed(source)
        pages[relative] = parser.page()
        sources[relative] = source
    return pages, sources


def _page_errors(
    pages: dict[PurePosixPath, Page], sources: dict[PurePosixPath, str]
) -> list[str]:
    errors: list[str] = []
    title_pages: dict[str, PurePosixPath] = {}
    for relative, page in pages.items():
        if not page.title:
            errors.append(f"missing page title: {relative}")
        if not _is_embedded_rustdoc(relative) and relative != PurePosixPath("404.html"):
            if previous := title_pages.get(page.title):
                errors.append(
                    f"duplicate page title {page.title!r}: {previous} and {relative}"
                )
            else:
                title_pages[page.title] = relative
        if not _is_embedded_rustdoc(relative):
            if not page.canonical:
                errors.append(f"missing canonical URL: {relative}")
            errors.extend(
                f"invalid canonical URL in {relative}: {canonical}"
                for canonical in page.canonical
                if not canonical.startswith(SITE_BASE)
            )
        source = sources[relative]
        if relative.parts[:2] == ("python", "reference") and PRIVATE_PYTHON_ID.search(
            source
        ):
            errors.append(f"private Python API rendered in {relative}")
        if str(ROOT) in source:
            errors.append(f"checkout path leaked in {relative}: {ROOT}")
    return errors


def _reference_errors(site: Path, pages: dict[PurePosixPath, Page]) -> list[str]:
    errors: list[str] = []
    for relative, page in pages.items():
        for reference in page.references:
            parsed = urlparse(reference.target)
            if reference.runtime_asset and parsed.scheme in {"http", "https"}:
                errors.append(
                    f"external runtime asset in {relative}: {reference.target}"
                )
                continue
            resolved = _artifact_target(site, relative, reference.target)
            if resolved is None:
                continue
            if isinstance(resolved, str):
                errors.append(f"{relative}: {resolved}")
                continue
            target, fragment = resolved
            if not target.is_file():
                errors.append(
                    f"missing internal target from {relative}: {reference.target}"
                )
                continue
            target_relative = PurePosixPath(target.relative_to(site).as_posix())
            if (
                fragment
                and target.suffix == ".html"
                and not _is_embedded_rustdoc(relative)
                and not _is_embedded_rustdoc(target_relative)
            ):
                target_page = pages.get(target_relative)
                if target_page is None or fragment not in target_page.ids:
                    errors.append(f"missing anchor from {relative}: {reference.target}")
    return errors


def validate_site(
    site: Path,
    *,
    max_uncompressed: int = MAX_UNCOMPRESSED_BYTES,
    max_compressed: int = MAX_COMPRESSED_BYTES,
) -> tuple[list[str], SiteReport]:
    """Validate one complete generated site without network access."""
    files = sorted(path for path in site.rglob("*") if path.is_file())
    report = _site_report(site, files)
    errors = _budget_errors(report, max_uncompressed, max_compressed)
    errors.extend(
        f"missing required entry point: {entry}"
        for entry in REQUIRED_ENTRIES
        if not (site / entry).is_file()
    )
    pages, sources = _load_pages(site, files)
    errors.extend(_page_errors(pages, sources))
    errors.extend(_reference_errors(site, pages))
    return errors, report


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Validate a generated BTPC site")
    parser.add_argument("site", type=Path, help="generated site directory")
    parser.add_argument(
        "--max-uncompressed",
        type=int,
        default=MAX_UNCOMPRESSED_BYTES,
        help="maximum sum of generated file bytes",
    )
    parser.add_argument(
        "--max-compressed",
        type=int,
        default=MAX_COMPRESSED_BYTES,
        help="maximum normalized gzip payload bytes",
    )
    return parser


def main() -> int:
    arguments = _parser().parse_args()
    errors, report = validate_site(
        arguments.site.resolve(),
        max_uncompressed=arguments.max_uncompressed,
        max_compressed=arguments.max_compressed,
    )
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(
        "validated generated site: "
        f"{report.files} files, {report.html_files} HTML, "
        f"{report.uncompressed_bytes} bytes uncompressed, "
        f"{report.compressed_bytes} bytes normalized gzip"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
