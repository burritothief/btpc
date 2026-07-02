from __future__ import annotations

import shutil
from collections.abc import Callable
from pathlib import Path

import pytest

from scripts.check_docs_site import validate_site

CANONICAL = "https://burritothief.github.io/btpc/"
EXPECTED_HTML_FILES = 7


def _page(title: str, canonical: str, body: str = "") -> str:
    return f"""<!doctype html>
<html lang="en"><head><title>{title}</title>
<link rel="canonical" href="{canonical}">
<link rel="stylesheet" href="/btpc/assets/app.css"></head>
<body><h1 id="top">{title}</h1>{body}</body></html>
"""


def _valid_site(root: Path) -> None:
    pages = {
        "index.html": _page("Home", CANONICAL, '<a href="cli/reference/#top">CLI</a>'),
        "404.html": _page("Not found", f"{CANONICAL}404/"),
        "python/index.html": _page("Python", f"{CANONICAL}python/"),
        "python/reference/creation/index.html": _page(
            "Python creation", f"{CANONICAL}python/reference/creation/"
        ),
        "cli/reference/index.html": _page(
            "CLI reference", f"{CANONICAL}cli/reference/"
        ),
        "rust/index.html": _page("Rust", f"{CANONICAL}rust/"),
        "rust/btpc_core/index.html": (
            "<!doctype html><title>btpc_core</title><h1>crate</h1>"
        ),
    }
    for relative, content in pages.items():
        destination = root / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(content)
    (root / "assets").mkdir()
    (root / "assets/app.css").write_text("body { color: black; }")


def _broken_link(root: Path) -> None:
    (root / "index.html").write_text(
        _page("Home", CANONICAL, '<a href="missing/">Missing</a>')
    )


def _missing_anchor(root: Path) -> None:
    (root / "index.html").write_text(
        _page("Home", CANONICAL, '<a href="cli/reference/#missing">CLI</a>')
    )


def _missing_asset(root: Path) -> None:
    (root / "assets/app.css").unlink()


def _escaping_root_link(root: Path) -> None:
    (root / "index.html").write_text(
        _page("Home", CANONICAL, '<a href="/outside/">Outside</a>')
    )


def _forbidden_scheme(root: Path) -> None:
    (root / "index.html").write_text(
        _page("Home", CANONICAL, '<a href="file:///tmp/private">Private</a>')
    )


def _localhost_url(root: Path) -> None:
    (root / "index.html").write_text(
        _page("Home", CANONICAL, '<a href="http://localhost:8000/">Local</a>')
    )


def _checkout_path(root: Path) -> None:
    (root / "index.html").write_text(_page("Home", CANONICAL, str(Path.cwd())))


def _missing_canonical(root: Path) -> None:
    content = (root / "index.html").read_text()
    canonical = f'<link rel="canonical" href="{CANONICAL}">'
    (root / "index.html").write_text(content.replace(canonical, ""))


def _duplicate_title(root: Path) -> None:
    content = (root / "python/index.html").read_text().replace("Python", "Home")
    (root / "python/index.html").write_text(content)


def _missing_entry(root: Path) -> None:
    shutil.rmtree(root / "cli")


def _private_api(root: Path) -> None:
    path = root / "python/reference/creation/index.html"
    content = path.read_text().replace("<body>", '<body><h2 id="btpc._native.Secret">')
    path.write_text(content)


Mutation = Callable[[Path], None]


@pytest.mark.parametrize(
    ("mutation", "message"),
    [
        (_broken_link, "missing internal target"),
        (_missing_anchor, "missing anchor"),
        (_missing_asset, "missing internal target"),
        (_escaping_root_link, "escapes /btpc/"),
        (_forbidden_scheme, "forbidden URL scheme"),
        (_localhost_url, "localhost URL"),
        (_checkout_path, "checkout path"),
        (_missing_canonical, "missing canonical URL"),
        (_duplicate_title, "duplicate page title"),
        (_missing_entry, "missing required entry point"),
        (_private_api, "private Python API"),
    ],
)
def test_site_quality_rejects_broken_artifacts(
    tmp_path: Path, mutation: Mutation, message: str
) -> None:
    _valid_site(tmp_path)
    mutation(tmp_path)
    errors, _ = validate_site(tmp_path)
    assert any(message in error for error in errors)


def test_site_quality_enforces_size_budgets(tmp_path: Path) -> None:
    _valid_site(tmp_path)
    errors, _ = validate_site(tmp_path, max_uncompressed=1, max_compressed=1)
    assert any("uncompressed size budget" in error for error in errors)
    assert any("compressed size budget" in error for error in errors)


def test_site_quality_accepts_complete_fixture(tmp_path: Path) -> None:
    _valid_site(tmp_path)
    errors, report = validate_site(tmp_path)
    assert not errors
    assert report.html_files == EXPECTED_HTML_FILES
