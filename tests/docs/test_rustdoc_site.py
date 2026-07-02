from __future__ import annotations

import re
import subprocess
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).parents[2]
BUILDER = ROOT / "scripts/build_docs_site.py"
RUSTDOC_TARGET = ROOT / ".tmp/docs-rustdoc-target/doc"
ALLOWED_RUSTDOC_ENTRIES = {
    "btpc_core",
    "crates.js",
    "help.html",
    "search.index",
    "settings.html",
    "src",
    "src-files.js",
    "static.files",
    "trait.impl",
    "type.impl",
}


class RuntimeAssetInspector(HTMLParser):
    """Collect script and stylesheet URLs from rustdoc HTML."""

    def __init__(self) -> None:
        """Initialize an empty URL collection."""
        super().__init__()
        self.urls: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        """Record runtime script and stylesheet targets."""
        values = dict(attrs)
        if tag == "script" and values.get("src"):
            self.urls.append(values["src"])
        if tag == "link" and values.get("rel") == "stylesheet" and values.get("href"):
            self.urls.append(values["href"])


def _build(destination: Path) -> None:
    subprocess.run(  # noqa: S603
        [sys.executable, str(BUILDER), "--site-dir", str(destination)],
        cwd=destination.parent,
        check=True,
    )


def test_rust_landing_links_main_rustdoc_source_and_examples() -> None:
    landing = (ROOT / "docs/rust/index.md").read_text()
    assert "main" in landing
    assert "btpc_core/index.html" in landing
    assert "https://github.com/burritothief/btpc/tree/main/crates/btpc-core" in landing
    assert "https://docs.rs" not in landing
    assert "```rust" in landing


def test_builder_embeds_only_fresh_btpc_core_rustdoc(tmp_path: Path) -> None:
    first = tmp_path / "first"
    _build(first)
    sentinel = RUSTDOC_TARGET / "btpc_core/sentinel.html"
    sentinel.write_text("stale rustdoc")

    second = tmp_path / "second"
    _build(second)

    rust_root = second / "rust"
    assert (rust_root / "btpc_core/index.html").is_file()
    assert not sentinel.exists()
    assert not (rust_root / "btpc_core/sentinel.html").exists()
    assert {entry.name for entry in rust_root.iterdir()} == ALLOWED_RUSTDOC_ENTRIES | {
        "index.html"
    }
    assert not (rust_root / "file_id").exists()
    assert not (rust_root / "tempfile").exists()


def test_embedded_rustdoc_runtime_assets_are_same_origin(tmp_path: Path) -> None:
    destination = tmp_path / "site"
    _build(destination)
    rustdoc = destination / "rust/btpc_core/index.html"
    html = rustdoc.read_text()
    inspector = RuntimeAssetInspector()
    inspector.feed(html)
    assert inspector.urls
    assert all(not urlparse(url).scheme for url in inspector.urls)
    assert not re.search(r'(?:href|src)="/(?!/)', html)
    for target in inspector.urls:
        asset = (rustdoc.parent / target).resolve()
        assert asset.is_file(), target
