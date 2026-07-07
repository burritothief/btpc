from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from pathlib import Path

import yaml

ROOT = Path(__file__).parents[2]
BUILDER = ROOT / "scripts/build_docs_site.py"
CONFIG = ROOT / "mkdocs.yml"


def test_documentation_dependencies_are_locked_outside_runtime() -> None:
    project = tomllib.loads((ROOT / "pyproject.toml").read_text())
    docs = project["dependency-groups"]["docs"]
    assert "griffelib==2.1.0" in docs
    assert any(requirement.startswith("mkdocs-material") for requirement in docs)
    assert any(requirement.startswith("mkdocstrings-python") for requirement in docs)
    runtime = project["project"].get("dependencies", [])
    assert all("mkdocs" not in requirement for requirement in runtime)


def test_mkdocs_configuration_uses_canonical_project_site() -> None:
    config = yaml.safe_load(CONFIG.read_text())
    assert config["site_url"] == "https://burritothief.github.io/btpc/"
    assert config["repo_url"] == "https://github.com/burritothief/btpc"
    assert config["edit_uri"] == "edit/main/docs/"
    assert config["theme"]["name"] == "material"
    assert "search" in config["plugins"]
    assert config["nav"][0] == {"Home": "index.md"}
    assert config["use_directory_urls"] is True


def test_minimal_site_has_required_root_sources() -> None:
    assert (ROOT / "docs/index.md").is_file()
    assert (ROOT / "docs/404.md").is_file()
    assert "site/" in (ROOT / ".gitignore").read_text().splitlines()


def test_builder_exposes_reserved_stages() -> None:
    result = subprocess.run(  # noqa: S603
        [sys.executable, str(BUILDER), "--list-stages"],
        cwd=ROOT.parent,
        check=True,
        capture_output=True,
        text=True,
    )
    assert result.stdout.splitlines() == ["cli", "mkdocs", "rustdoc", "validate"]


def test_strict_builder_cleans_output_and_is_project_subpath_safe(
    tmp_path: Path,
) -> None:
    destination = tmp_path / "published"
    destination.mkdir()
    (destination / "stale.txt").write_text("stale")

    subprocess.run(  # noqa: S603
        [sys.executable, str(BUILDER), "--site-dir", str(destination)],
        cwd=tmp_path,
        check=True,
    )

    assert (destination / "index.html").is_file()
    assert (destination / "404.html").is_file()
    assert not (destination / "stale.txt").exists()
    html = (destination / "index.html").read_text()
    assert not re.search(r'(?:href|src)="/(?!/)', html)
