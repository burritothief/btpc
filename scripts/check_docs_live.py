from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urljoin, urlsplit
from urllib.request import Request, urlopen

ROOT = Path(__file__).parents[1]
SITE_URL = "https://burritothief.github.io/btpc/"
DEFAULT_MANIFEST = ROOT / "tests/docs/fixtures/renderer_migration_baseline.json"
USER_AGENT = "btpc-documentation-live-baseline/1"
GENERIC_GITHUB_ERRORS = (
    "Site not found · GitHub Pages",
    "There isn't a GitHub Pages site here.",
)
MIXED_CONTENT = re.compile(r"(?:href|src|action)=[\"']http://", re.IGNORECASE)
TOO_MANY_REQUESTS = 429
SERVER_ERROR_MINIMUM = 500
OK = 200


@dataclass(frozen=True)
class Response:
    """Relevant fields from one live HTML response."""

    status: int
    final_url: str
    content_type: str
    body: str


class PageParser(HTMLParser):
    """Collect anchors and BTPC document redirects from generated HTML."""

    def __init__(self) -> None:
        """Initialize an empty page description."""
        super().__init__()
        self.ids: set[str] = set()
        self.redirect: str | None = None

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        """Collect relevant attributes from an opening HTML tag."""
        values = dict(attrs)
        if identifier := values.get("id"):
            self.ids.add(identifier)
        if tag == "meta" and values.get("name") == "btpc-redirect":
            self.redirect = values.get("content")


def page_facts(body: str) -> PageParser:
    parser = PageParser()
    parser.feed(body)
    return parser


def route_url(path: str) -> str:
    if path == "index.html":
        return SITE_URL
    if path.endswith("/index.html"):
        return urljoin(SITE_URL, path.removesuffix("index.html"))
    return urljoin(SITE_URL, path)


def _response_errors(url: str, expected: str, response: Response) -> list[str]:
    errors: list[str] = []
    if response.status != OK:
        errors.append(f"{url}: status {response.status}, expected 200")
    if not response.content_type.lower().startswith("text/html"):
        errors.append(f"{url}: unexpected content type {response.content_type!r}")
    if response.final_url != url:
        errors.append(f"{url}: unexpected final URL {response.final_url}")
    if response.body != expected:
        errors.append(f"{url}: live body differs from the local deployed artifact")
    if any(marker in response.body for marker in GENERIC_GITHUB_ERRORS):
        errors.append(f"{url}: generic GitHub Pages error returned as content")
    if MIXED_CONTENT.search(response.body):
        errors.append(f"{url}: mixed-content URL found in response")
    if urlsplit(response.final_url).scheme != "https":
        errors.append(f"{url}: final URL is not HTTPS: {response.final_url}")
    return errors


def validate_route(
    url: str,
    expected_bodies: dict[str, str],
    responses: dict[str, Response],
) -> list[str]:
    expected = expected_bodies[url]
    response = responses[url]
    errors = _response_errors(url, expected, response)
    redirect = page_facts(response.body).redirect
    if redirect is None:
        return errors
    target = urljoin(url, redirect)
    if not target.startswith(SITE_URL):
        errors.append(f"{url}: redirect leaves the project site: {target}")
        return errors
    if target == url or target not in expected_bodies or target not in responses:
        errors.append(f"{url}: redirect loop or missing local target: {target}")
        return errors
    errors.extend(_response_errors(target, expected_bodies[target], responses[target]))
    if page_facts(responses[target].body).redirect is not None:
        errors.append(
            f"{url}: redirect loop or exceeds one compatibility hop: {target}"
        )
    return errors


def _fetch_once(url: str, timeout: float) -> Response:
    request = Request(url, headers={"User-Agent": USER_AGENT})  # noqa: S310
    try:
        response = urlopen(request, timeout=timeout)  # noqa: S310
    except HTTPError as error:
        response = error
    with response:
        return Response(
            status=response.status,
            final_url=response.geturl(),
            content_type=response.headers.get("Content-Type", ""),
            body=response.read().decode("utf-8", errors="replace"),
        )


def fetch(url: str, attempts: int, timeout: float) -> Response:
    for attempt in range(1, attempts + 1):
        try:
            response = _fetch_once(url, timeout)
        except URLError:
            if attempt == attempts:
                raise
        else:
            if (
                response.status != TOO_MANY_REQUESTS
                and response.status < SERVER_ERROR_MINIMUM
            ):
                return response
            if attempt == attempts:
                return response
        time.sleep(attempt)
    raise AssertionError("unreachable")


def _load_bodies(site: Path) -> tuple[dict[str, str], dict[str, str]]:
    paths: dict[str, str] = {}
    bodies: dict[str, str] = {}
    for path in sorted(site.rglob("*.html")):
        relative = path.relative_to(site).as_posix()
        url = route_url(relative)
        paths[relative] = url
        bodies[url] = path.read_text(errors="replace")
    return paths, bodies


def check_live(
    site: Path,
    manifest: dict[str, object],
    attempts: int,
    timeout: float,
    workers: int,
) -> tuple[list[str], list[str]]:
    paths, bodies = _load_bodies(site)
    responses: dict[str, Response] = {}
    errors: list[str] = []
    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {
            executor.submit(fetch, url, attempts, timeout): url for url in bodies
        }
        for future in as_completed(futures):
            url = futures[future]
            try:
                responses[url] = future.result()
            except URLError as error:
                errors.append(f"{url}: request failed: {error}")

    for url in sorted(responses):
        errors.extend(validate_route(url, bodies, responses))

    rows: list[str] = []
    anchors = manifest.get("anchors", {})
    for route in manifest.get("routes", []):
        relative = str(route["path"])
        url = paths.get(relative)
        if url is None:
            errors.append(f"baseline route missing from local artifact: {relative}")
            continue
        response = responses.get(url)
        if response is None:
            continue
        facts = page_facts(response.body)
        target = urljoin(url, facts.redirect) if facts.redirect else url
        target_response = responses.get(target)
        target_facts = page_facts(target_response.body) if target_response else facts
        missing = [
            anchor
            for anchor in anchors.get(relative, [])
            if anchor not in target_facts.ids
        ]
        errors.extend(f"{url}: missing live anchor #{anchor}" for anchor in missing)
        mode = "redirect" if facts.redirect else "direct"
        rows.append(
            f"| `{relative}` | {response.status} | `{response.final_url}` | "
            f"{mode}; {len(anchors.get(relative, [])) - len(missing)} anchors |"
        )
    return errors, rows


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Compare live Pages HTML with the locally built artifact"
    )
    parser.add_argument("--site-dir", type=Path, default=ROOT / "site")
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--attempts", type=int, default=3)
    parser.add_argument("--timeout", type=float, default=20)
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--summary", type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    arguments = _parser().parse_args(argv)
    manifest = json.loads(arguments.manifest.read_text())
    errors, rows = check_live(
        arguments.site_dir.resolve(),
        manifest,
        arguments.attempts,
        arguments.timeout,
        arguments.workers,
    )
    summary = "\n".join(
        [
            "## Live documentation route baseline",
            "",
            "| Baseline route | Status | Final URL | Marker result |",
            "| --- | ---: | --- | --- |",
            *rows,
            "",
            f"Checked {len(rows)} baseline routes against the local artifact.",
            "",
        ]
    )
    print(summary)
    summary_path = arguments.summary or (
        Path(value) if (value := os.environ.get("GITHUB_STEP_SUMMARY")) else None
    )
    if summary_path is not None:
        with summary_path.open("a") as output:
            output.write(summary)
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
