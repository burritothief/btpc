from __future__ import annotations

import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).parents[2]
BUILDER = ROOT / "scripts/build_mdbook_site.py"


def _joined(*parts: str) -> str:
    return "".join(parts)


REMOVED_PATHS = (
    _joined("mk", "docs.yml"),
    _joined("docs/", "overrides"),
    _joined("docs/stylesheets/", "extra.css"),
    _joined("scripts/build_", "docs_site.py"),
)
FORBIDDEN_DEPENDENCIES = (
    _joined("mk", "docs"),
    _joined("mat", "erial"),
    _joined("mkdoc", "strings"),
    _joined("pym", "down"),
)


def test_documentation_dependencies_are_locked_outside_runtime() -> None:
    project = tomllib.loads((ROOT / "pyproject.toml").read_text())
    docs = project["dependency-groups"]["docs"]
    assert docs == ["griffelib==2.1.0", "pyyaml==6.0.3"]
    assert not any(
        forbidden in requirement.lower()
        for requirement in docs
        for forbidden in FORBIDDEN_DEPENDENCIES
    )
    runtime = project["project"].get("dependencies", [])
    assert not any("griffe" in requirement.lower() for requirement in runtime)


def test_mdbook_configuration_uses_canonical_project_site() -> None:
    config = tomllib.loads((ROOT / "book.toml").read_text())
    html = config["output"]["html"]
    assert config["book"]["title"] == "BTPC Documentation"
    assert config["book"]["src"] == "docs"
    assert html["site-url"] == "/btpc/"
    assert html["git-repository-url"] == "https://github.com/burritothief/btpc"
    assert html["edit-url-template"].endswith("/edit/main/{path}")
    assert html["additional-css"] == ["docs/stylesheets/mdbook.css"]
    assert html["search"]["enable"] is True


def test_removed_renderer_stack_is_absent() -> None:
    for relative in REMOVED_PATHS:
        assert not (ROOT / relative).exists(), relative


def test_minimal_site_has_required_root_sources() -> None:
    assert (ROOT / "docs/index.md").is_file()
    assert (ROOT / "docs/404.md").is_file()
    assert (ROOT / "docs/SUMMARY.md").is_file()
    assert "site/" in (ROOT / ".gitignore").read_text().splitlines()


def test_canonical_commands_use_shared_mdbook_builder() -> None:
    makefile = (ROOT / "Makefile").read_text()
    assert "python scripts/build_mdbook_site.py --site-dir site" in makefile
    assert "python scripts/serve_mdbook.py" in makefile
    assert _joined("scripts/build_", "docs_site.py") not in makefile
    result = subprocess.run(  # noqa: S603
        [sys.executable, BUILDER, "--list-stages"],
        cwd=ROOT.parent,
        check=True,
        capture_output=True,
        text=True,
    )
    assert result.stdout.splitlines()[-1] == "publish"
    serve = (ROOT / "scripts/serve_mdbook.py").read_text()
    assert "build_site(destination)" in serve
    assert "ThreadingHTTPServer" in serve
    assert 'preview_root / "btpc"' in serve
    config = tomllib.loads((ROOT / "book.toml").read_text())
    assert config["build"]["extra-watch-dirs"] == [
        "python/btpc",
        "crates/btpc-cli/src",
        "crates/btpc-core/src",
    ]
