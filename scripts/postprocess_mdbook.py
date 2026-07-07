from __future__ import annotations

import argparse
import html
import json
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).parents[1]
BASELINE = ROOT / "tests/docs/fixtures/renderer_migration_baseline.json"
PAGES_ROOT = "/btpc/"
NOTICE = (
    '<aside class="btpc-development-notice" role="note">'
    "<strong>Development documentation:</strong> this site describes the current "
    "<code>main</code> branch before BTPC 1.0.</aside>"
)


def _inject_head(source: str, markup: str) -> str:
    return source.replace("</head>", f"{markup}\n    </head>", 1)


def _canonical_markup(url: str) -> str:
    return f'<link rel="canonical" href="{html.escape(url, quote=True)}">'


def _redirect_page(*, canonical: str, route: str, target: str) -> str:
    escaped_target = html.escape(target, quote=True)
    script_target = json.dumps(target)
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="btpc-redirect" content="{escaped_target}">
  <meta http-equiv="refresh" content="0; url={escaped_target}">
  {_canonical_markup(canonical)}
  <title>BTPC documentation moved: {html.escape(route)}</title>
</head>
<body>
  <p>This documentation page moved to <a href="{escaped_target}">{escaped_target}</a>.</p>
  <script>location.replace({script_target} + location.search + location.hash);</script>
</body>
</html>
"""


def _candidate(route: str, site: Path) -> str | None:
    if not route.endswith("/index.html"):
        return None
    candidate = f"{route.removesuffix('/index.html')}.html"
    if (site / candidate).is_file():
        return candidate
    return None


def postprocess(site: Path) -> tuple[int, int]:
    baseline = json.loads(BASELINE.read_text())
    canonicals = baseline["canonical_urls"]
    target_canonicals: dict[str, str] = {}
    redirects: list[tuple[str, str, str]] = []
    for route in baseline["routes"]:
        relative = route["path"]
        if relative.startswith("rust/") and relative != "rust/index.html":
            continue
        if (site / relative).is_file():
            if canonical := canonicals.get(relative):
                target_canonicals[relative] = canonical
            continue
        candidate = _candidate(relative, site)
        if candidate is None:
            raise RuntimeError(f"no mdBook target for baseline route: {relative}")
        canonical = canonicals.get(
            relative, f"https://burritothief.github.io/btpc/{relative}"
        )
        redirects.append((relative, candidate, canonical))
        target_canonicals.setdefault(candidate, canonical)

    processed = 0
    for path in sorted(site.rglob("*.html")):
        relative = path.relative_to(site).as_posix()
        source = path.read_text()
        if "<main>" in source and "btpc-development-notice" not in source:
            source = source.replace("<main>", f"<main>\n{NOTICE}", 1)
        if (
            canonical := target_canonicals.get(relative)
        ) and 'rel="canonical"' not in source:
            source = _inject_head(source, _canonical_markup(canonical))
        path.write_text(source)
        processed += 1

    for route, target, canonical in redirects:
        destination = site / PurePosixPath(route)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            _redirect_page(
                canonical=canonical,
                route=route,
                target=f"{PAGES_ROOT}{target}",
            )
        )
    return processed, len(redirects)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Finalize the side-by-side mdBook site"
    )
    parser.add_argument("--site-dir", type=Path, required=True)
    arguments = parser.parse_args()
    pages, redirects = postprocess(arguments.site_dir.resolve())
    print(f"postprocessed {pages} mdBook pages and created {redirects} redirects")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
