from __future__ import annotations

import ast
import re
import shutil
import subprocess
import sys
from html.parser import HTMLParser
from pathlib import Path

import yaml
from griffe import GriffeLoader

ROOT = Path(__file__).parents[2]
PACKAGE = ROOT / "python/btpc"
BUILDER = ROOT / "scripts/build_docs_site.py"
PUBLIC_MODULES = ("creation", "metainfo", "verification", "types", "errors")


class TextCollector(HTMLParser):
    """Collect visible text from generated reference markup."""

    def __init__(self) -> None:
        """Initialize an empty text collection."""
        super().__init__()
        self.parts: list[str] = []

    def handle_data(self, data: str) -> None:
        """Record one visible text fragment."""
        self.parts.append(data)


def _visible_text(html: str) -> str:
    collector = TextCollector()
    collector.feed(html)
    return re.sub(r"\s+", " ", " ".join(collector.parts)).strip()


def _exports(module: str) -> list[str]:
    tree = ast.parse((PACKAGE / f"{module}.py").read_text())
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == "__all__"
            for target in node.targets
        ):
            return ast.literal_eval(node.value)
    raise AssertionError


def _root_exports() -> list[str]:
    tree = ast.parse((PACKAGE / "__init__.py").read_text())
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == "__all__"
            for target in node.targets
        ):
            return ast.literal_eval(node.value)
    raise AssertionError


def test_reference_pages_cover_the_static_public_inventory_once() -> None:
    defining_modules: dict[str, str] = {}
    for module in PUBLIC_MODULES:
        exports = _exports(module)
        page = ROOT / f"docs/python/reference/{module}.md"
        source = page.read_text()
        assert source.count(f"<!-- btpc-python-api: btpc.{module} -->") == 1
        for symbol in exports:
            assert symbol not in defining_modules
            defining_modules[symbol] = module

    root_exports = set(_root_exports())
    expected_root = set(defining_modules) | set(PUBLIC_MODULES) | {"__version__"}
    assert root_exports == expected_root

    overview = (ROOT / "docs/python/index.md").read_text()
    assert '<a id="package-version"></a>' in overview
    for symbol, module in defining_modules.items():
        target = f"reference/{module}.md#btpc.{module}.{symbol}"
        assert target in overview


def test_mkdocstrings_configuration_is_static_and_annotation_aware() -> None:
    config = yaml.safe_load((ROOT / "mkdocs.yml").read_text())
    handler = config["plugins"][1]["mkdocstrings"]["handlers"]["python"]
    assert handler["paths"] == ["python"]
    options = handler["options"]
    assert options["docstring_style"] == "google"
    assert options["members_order"] == "source"
    assert options["show_signature_annotations"] is True
    assert options["signature_crossrefs"] is True
    assert options["show_source"] is False


def test_griffe_collects_public_modules_without_the_native_extension(
    tmp_path: Path,
) -> None:
    package = tmp_path / "btpc"
    shutil.copytree(PACKAGE, package, ignore=shutil.ignore_patterns("*.so", "*.pyd"))
    loader = GriffeLoader(search_paths=[tmp_path], allow_inspection=False)
    for module in PUBLIC_MODULES:
        loaded = loader.load(f"btpc.{module}")
        for symbol in _exports(module):
            assert symbol in loaded.members


def test_generated_reference_has_one_canonical_anchor_per_symbol(
    tmp_path: Path,
) -> None:
    destination = tmp_path / "site"
    subprocess.run(  # noqa: S603
        [sys.executable, str(BUILDER), "--site-dir", str(destination)],
        cwd=tmp_path,
        check=True,
    )

    reference_pages = list((destination / "python/reference").rglob("*.html"))
    all_html = "\n".join(page.read_text() for page in reference_pages)
    assert 'id="btpc._native' not in all_html
    assert 'id="btpc._conversion' not in all_html
    for module in PUBLIC_MODULES:
        page = destination / f"python/reference/{module}/index.html"
        html = page.read_text()
        for symbol in _exports(module):
            anchor = f'id="btpc.{module}.{symbol}"'
            assert html.count(anchor) == 1
            assert all_html.count(anchor) == 1

    creation = (destination / "python/reference/creation/index.html").read_text()
    metainfo = (destination / "python/reference/metainfo/index.html").read_text()
    creation_text = _visible_text(creation)
    metainfo_text = _visible_text(metainfo)
    assert "Callable [[ int , int , int ], None ] | None" in creation_text
    assert "Raises:" in creation_text
    assert (
        "from_bytes ( data : object , * , options : ParseOptions | None = None )"
        " -> Metainfo"
    ) in metainfo_text
    assert 'href="../types/#btpc.types.ParseOptions"' in metainfo
