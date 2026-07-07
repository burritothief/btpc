from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).parents[2]
REPOSITORY = "https://github.com/burritothief/btpc"
DOCUMENTATION = "https://burritothief.github.io/btpc/"
OWNER_DRIFT_FILES = (
    "Cargo.toml",
    "pyproject.toml",
    "README.md",
    "CHANGELOG.md",
    "CONTRIBUTING.md",
    "book.toml",
)


def test_cargo_and_python_metadata_use_canonical_urls() -> None:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]
    assert cargo["repository"] == REPOSITORY
    assert cargo["homepage"] == DOCUMENTATION
    assert cargo["documentation"] == DOCUMENTATION

    project = tomllib.loads((ROOT / "pyproject.toml").read_text())["project"]
    assert project["urls"] == {
        "Documentation": DOCUMENTATION,
        "Homepage": DOCUMENTATION,
        "Issues": f"{REPOSITORY}/issues",
        "Repository": REPOSITORY,
        "Source": REPOSITORY,
    }

    metadata = json.loads(
        subprocess.run(  # noqa: S603
            [
                shutil.which("cargo") or "cargo",
                "metadata",
                "--no-deps",
                "--format-version",
                "1",
            ],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    for package in metadata["packages"]:
        assert package["repository"] == REPOSITORY
        assert package["homepage"] == DOCUMENTATION
        assert package["documentation"] == DOCUMENTATION

    for relative in OWNER_DRIFT_FILES:
        source = (ROOT / relative).read_text()
        assert "github.com/" + "btpc-dev/btpc" not in source


def test_readme_site_navigation_and_runbooks_are_discoverable() -> None:
    readme = (ROOT / "README.md").read_text()
    assert f"[Documentation]({DOCUMENTATION})" in readme

    config = tomllib.loads((ROOT / "book.toml").read_text())
    html = config["output"]["html"]
    assert html["site-url"] == "/btpc/"
    assert html["git-repository-url"] == REPOSITORY
    homepage = (ROOT / "docs/index.md").read_text()
    assert f"{REPOSITORY}/blob/main/LICENSE" in homepage

    contributing = (ROOT / "CONTRIBUTING.md").read_text()
    for topic in (
        "GitHub Actions",
        "github-pages",
        "workflow_dispatch",
        "project subpath",
        "404",
        "rollback",
        "known-good commit",
    ):
        assert topic.lower() in contributing.lower()
    release = (ROOT / "docs/release-checklist.md").read_text()
    assert DOCUMENTATION in release
    assert "current `main` branch" in release


def test_generated_site_has_canonical_sitemap_edit_and_license_links(
    tmp_path: Path,
) -> None:
    destination = tmp_path / "site"
    subprocess.run(  # noqa: S603
        [
            sys.executable,
            ROOT / "scripts/build_mdbook_site.py",
            "--site-dir",
            destination,
        ],
        cwd=tmp_path,
        check=True,
    )
    homepage = (destination / "index.html").read_text()
    assert f'<link rel="canonical" href="{DOCUMENTATION}">' in homepage
    assert f"{REPOSITORY}/edit/main/docs/index.md" in homepage
    assert f"{REPOSITORY}/blob/main/LICENSE" in homepage
    sitemap = (destination / "sitemap.xml").read_text()
    assert f"<loc>{DOCUMENTATION}</loc>" in sitemap
