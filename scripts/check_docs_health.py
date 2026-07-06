from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import Request, urlopen

ROOT = Path(__file__).parents[1]
DEFAULT_MANIFEST = ROOT / ".github/docs-health.json"
USER_AGENT = "btpc-documentation-health/1"
GENERIC_GITHUB_ERRORS = (
    "Site not found · GitHub Pages",
    "There isn't a GitHub Pages site here.",
)
MIXED_CONTENT = re.compile(r"(?:href|src|action)=[\"']http://", re.IGNORECASE)
TOO_MANY_REQUESTS = 429
SERVER_ERROR_MINIMUM = 500


@dataclass(frozen=True)
class Check:
    """One expected production response."""

    name: str
    url: str
    status: int
    content_type: str
    marker: str


@dataclass(frozen=True)
class Response:
    """Relevant fields from one HTTP response."""

    status: int
    final_url: str
    content_type: str
    body: str


def validate(check: Check, response: Response) -> list[str]:
    """Return actionable errors for a response that violates its contract."""
    errors: list[str] = []
    prefix = f"{check.name} ({check.url})"
    if response.status != check.status:
        errors.append(f"{prefix}: status {response.status}, expected {check.status}")
    if not response.content_type.lower().startswith(check.content_type.lower()):
        errors.append(
            f"{prefix}: content type {response.content_type!r}, "
            f"expected {check.content_type!r}"
        )
    if check.marker not in response.body:
        errors.append(f"{prefix}: missing marker {check.marker!r}")
    if any(marker in response.body for marker in GENERIC_GITHUB_ERRORS):
        errors.append(f"{prefix}: generic GitHub Pages error returned as content")
    if MIXED_CONTENT.search(response.body):
        errors.append(f"{prefix}: mixed-content URL found in response")
    if not response.final_url.startswith("https://"):
        errors.append(f"{prefix}: final URL is not HTTPS: {response.final_url}")
    return errors


def _fetch_once(check: Check, timeout: float) -> Response:
    if urlsplit(check.url).scheme != "https":
        raise ValueError(f"health check URL must use HTTPS: {check.url}")
    request = Request(  # noqa: S310
        check.url,
        headers={"User-Agent": USER_AGENT},
    )
    try:
        response = urlopen(request, timeout=timeout)  # noqa: S310
    except HTTPError as error:
        response = error
    with response:
        body = response.read().decode("utf-8", errors="replace")
        return Response(
            status=response.status,
            final_url=response.geturl(),
            content_type=response.headers.get("Content-Type", ""),
            body=body,
        )


def fetch(check: Check, *, attempts: int, timeout: float) -> Response:
    """Fetch a check, retrying transient transport and server failures."""
    if attempts < 1:
        raise ValueError("attempts must be at least one")
    last_error: URLError | None = None
    for attempt in range(1, attempts + 1):
        try:
            response = _fetch_once(check, timeout)
        except URLError as error:
            last_error = error
        else:
            transient = (
                response.status == TOO_MANY_REQUESTS
                or response.status >= SERVER_ERROR_MINIMUM
            )
            if not transient or response.status == check.status or attempt == attempts:
                return response
        if attempt < attempts:
            time.sleep(2 ** (attempt - 1))
    if last_error is None:
        raise RuntimeError("health request exhausted retries without a response")
    raise last_error


def load_checks(path: Path) -> list[Check]:
    """Load and validate the checked-in production contract."""
    raw = json.loads(path.read_text())
    if not isinstance(raw, list) or not raw:
        raise ValueError("health manifest must contain a non-empty list")
    checks = [Check(**item) for item in raw]
    names = [check.name for check in checks]
    if len(names) != len(set(names)):
        raise ValueError("health manifest check names must be unique")
    return checks


def _summary(results: list[tuple[Check, Response]], errors: list[str]) -> str:
    lines = ["## Documentation health", ""]
    for check, response in results:
        state = "PASS" if not validate(check, response) else "FAIL"
        lines.append(
            f"- **{state}** `{check.name}`: {response.status} "
            f"`{response.content_type}` → {response.final_url}"
        )
    if errors:
        lines.extend(("", "### Failures", ""))
        lines.extend(f"- {error}" for error in errors)
    return "\n".join(lines) + "\n"


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate the live BTPC documentation contract"
    )
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--attempts", type=int, default=3)
    parser.add_argument("--timeout", type=float, default=20)
    parser.add_argument("--summary", type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    """Run all live checks and optionally append a GitHub Actions summary."""
    arguments = _parser().parse_args(argv)
    results: list[tuple[Check, Response]] = []
    errors: list[str] = []
    for check in load_checks(arguments.manifest):
        try:
            response = fetch(
                check,
                attempts=arguments.attempts,
                timeout=arguments.timeout,
            )
        except URLError as error:
            errors.append(f"{check.name} ({check.url}): request failed: {error}")
            continue
        results.append((check, response))
        errors.extend(validate(check, response))

    summary = _summary(results, errors)
    print(summary, end="")
    summary_path = arguments.summary or (
        Path(value) if (value := os.environ.get("GITHUB_STEP_SUMMARY")) else None
    )
    if summary_path is not None:
        with summary_path.open("a") as output:
            output.write(summary)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
