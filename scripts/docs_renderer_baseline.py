from __future__ import annotations

import argparse
import gzip
import html
import json
import re
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlparse

SCHEMA = "btpc.docs-renderer-baseline.v1"
SITE_URL = "https://burritothief.github.io/btpc/"
IMPORTANT_ANCHORS = {
    "cli/reference/create/index.html": ["btpc-create", "synopsis", "options"],
    "cli/reference/inspect/index.html": ["btpc-inspect", "synopsis", "options"],
    "cli/reference/verify/index.html": ["btpc-verify", "synopsis", "options"],
    "python/reference/creation/index.html": [
        "btpc.creation.CreateOptions",
        "btpc.creation.create",
    ],
    "python/reference/errors/index.html": [
        "btpc.errors.BtpcError",
        "btpc.errors.PathError",
    ],
    "python/reference/metainfo/index.html": [
        "btpc.metainfo.Metainfo",
        "btpc.metainfo.Metainfo.from_bytes",
    ],
    "python/reference/types/index.html": [
        "btpc.types.ParseOptions",
        "btpc.types.TorrentPath",
    ],
    "python/reference/verification/index.html": [
        "btpc.verification.PayloadVerificationReport",
        "btpc.verification.verify",
    ],
    "rust/btpc_core/index.html": ["main-content", "reexport.Metainfo", "functions"],
}
REQUIRED_FEATURES = [
    "client-side-search",
    "light-dark-theme-toggle",
    "edit-links",
    "code-copy-buttons",
    "keyboard-navigation",
    "custom-404",
]
PRIVACY = [
    "no-remote-runtime-assets",
    "no-analytics",
    "no-tracking",
    "local-search-index",
]
ASSET_PREFIXES = [
    "css/",
    "docs/stylesheets/",
    "fonts/",
    "rust/static.files/",
    "stylesheets/",
]


class _PageParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.ids: set[str] = set()
        self.canonical: str | None = None
        self.redirect: str | None = None

    def handle_starttag(
        self, tag: str, attributes: list[tuple[str, str | None]]
    ) -> None:
        values = dict(attributes)
        if identifier := values.get("id"):
            self.ids.add(identifier)
        if tag == "link" and values.get("rel") == "canonical":
            self.canonical = values.get("href")
        if tag == "meta" and values.get("name") == "btpc-redirect":
            self.redirect = values.get("content")


def page_facts(path: Path) -> _PageParser:
    parser = _PageParser()
    parser.feed(path.read_text(errors="replace"))
    return parser


def artifact_stats(site: Path) -> dict[str, int]:
    files = sorted(path for path in site.rglob("*") if path.is_file())
    payload = bytearray()
    total = 0
    for path in files:
        content = path.read_bytes()
        total += len(content)
        payload.extend(path.relative_to(site).as_posix().encode())
        payload.extend(b"\0")
        payload.extend(str(len(content)).encode())
        payload.extend(b"\0")
        payload.extend(content)
    return {
        "files": len(files),
        "html_pages": sum(path.suffix == ".html" for path in files),
        "uncompressed_bytes": total,
        "normalized_gzip_bytes": len(
            gzip.compress(bytes(payload), compresslevel=9, mtime=0)
        ),
    }


def sitemap_routes(site: Path) -> list[str]:
    sitemap = site / "sitemap.xml"
    if not sitemap.is_file():
        return []
    locations = re.findall(r"<loc>([^<]+)</loc>", sitemap.read_text())
    return sorted(urlparse(html.unescape(location)).path for location in locations)


def generate_manifest(arguments: argparse.Namespace) -> dict[str, object]:
    site = arguments.site_dir.resolve()
    html = sorted(
        (path for path in site.rglob("*.html") if path.is_file()),
        key=lambda path: path.relative_to(site).as_posix(),
    )
    routes = [
        {"path": path.relative_to(site).as_posix(), "mode": "direct"} for path in html
    ]
    canonical_urls: dict[str, str] = {}
    page_ids: dict[str, set[str]] = {}
    for path in html:
        relative = path.relative_to(site).as_posix()
        facts = page_facts(path)
        page_ids[relative] = facts.ids
        if facts.canonical:
            canonical_urls[relative] = facts.canonical
    anchors = {
        route: anchors
        for route, anchors in IMPORTANT_ANCHORS.items()
        if route in page_ids and all(anchor in page_ids[route] for anchor in anchors)
    }
    stats = artifact_stats(site)
    stats["build_duration_milliseconds"] = arguments.build_duration_ms
    navigation_routes = sorted(
        route["path"]
        for route in routes
        if not route["path"].startswith("rust/")
        and route["path"] not in {"404.html", "404/index.html"}
    )
    return {
        "schema": SCHEMA,
        "site_url": SITE_URL,
        "pages_root": "/btpc/",
        "deployed": {
            "commit": arguments.deployed_commit,
            "completed_at": arguments.deployed_at,
            "workflow_run": arguments.workflow_run,
            "required_urls": arguments.live_url,
        },
        "artifact": stats,
        "required_features": REQUIRED_FEATURES,
        "privacy": PRIVACY,
        "routes": routes,
        "navigation_routes": navigation_routes,
        "anchors": anchors,
        "canonical_urls": dict(sorted(canonical_urls.items())),
        "assets": ASSET_PREFIXES,
        "sitemap_routes": sitemap_routes(site),
        "custom_404": "404.html",
    }


def _route_facts(
    site: Path, manifest: dict[str, object]
) -> tuple[list[str], dict[str, _PageParser]]:
    errors: list[str] = []
    pages: dict[str, _PageParser] = {}
    for route in manifest.get("routes", []):
        path = str(route["path"])
        target = site / path
        if not target.is_file():
            errors.append(f"missing route: {path}")
        elif target.suffix == ".html":
            pages[path] = page_facts(target)
    return errors, pages


def _anchor_errors(
    site: Path, manifest: dict[str, object], pages: dict[str, _PageParser]
) -> list[str]:
    errors: list[str] = []
    for route, anchors in manifest.get("anchors", {}).items():
        facts = pages.get(route)
        if facts is None and (site / route).is_file():
            facts = page_facts(site / route)
        if facts is not None and facts.redirect is not None:
            target = site / facts.redirect.removeprefix("/btpc/")
            facts = page_facts(target) if target.is_file() else None
        errors.extend(
            f"missing anchor: {route}#{anchor}"
            for anchor in anchors
            if facts is None or anchor not in facts.ids
        )
    return errors


def _canonical_errors(site: Path, manifest: dict[str, object]) -> list[str]:
    errors: list[str] = []
    for route, expected in manifest.get("canonical_urls", {}).items():
        target = site / route
        actual = page_facts(target).canonical if target.is_file() else None
        if actual != expected:
            errors.append(f"canonical mismatch: {route}: {actual!r} != {expected!r}")
    return errors


def _asset_errors(site: Path, manifest: dict[str, object]) -> list[str]:
    files = [
        path.relative_to(site).as_posix() for path in site.rglob("*") if path.is_file()
    ]
    return [
        f"missing asset class: {prefix}"
        for prefix in manifest.get("assets", [])
        if not any(path.startswith(prefix) for path in files)
    ]


def _sitemap_errors(site: Path, manifest: dict[str, object]) -> list[str]:
    actual_sitemap = set(sitemap_routes(site))
    return [
        f"missing sitemap route: {route}"
        for route in manifest.get("sitemap_routes", [])
        if route not in actual_sitemap
    ]


def compare(site: Path, manifest: dict[str, object]) -> list[str]:
    errors, pages = _route_facts(site, manifest)
    errors.extend(_anchor_errors(site, manifest, pages))
    errors.extend(_canonical_errors(site, manifest))
    errors.extend(_asset_errors(site, manifest))
    errors.extend(_sitemap_errors(site, manifest))
    custom_404 = str(manifest.get("custom_404", "404.html"))
    if not (site / custom_404).is_file():
        errors.append(f"missing custom 404: {custom_404}")
    return errors


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(
        description="Generate or compare renderer-neutral docs baselines"
    )
    commands = root.add_subparsers(dest="command", required=True)
    generate = commands.add_parser("generate")
    generate.add_argument("--site-dir", type=Path, required=True)
    generate.add_argument("--output", type=Path, required=True)
    generate.add_argument("--build-duration-ms", type=int, required=True)
    generate.add_argument("--deployed-commit", required=True)
    generate.add_argument("--deployed-at", required=True)
    generate.add_argument("--workflow-run", required=True)
    generate.add_argument("--live-url", action="append", default=[])
    comparison = commands.add_parser("compare")
    comparison.add_argument("--site-dir", type=Path, required=True)
    comparison.add_argument("--manifest", type=Path, required=True)
    return root


def main() -> int:
    arguments = parser().parse_args()
    if arguments.command == "generate":
        manifest = generate_manifest(arguments)
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(
            json.dumps(manifest, indent=2, sort_keys=True) + "\n"
        )
        return 0
    manifest = json.loads(arguments.manifest.read_text())
    errors = compare(arguments.site_dir.resolve(), manifest)
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"renderer baseline satisfied: {len(manifest['routes'])} routes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
