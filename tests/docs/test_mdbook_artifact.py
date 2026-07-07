from __future__ import annotations

import hashlib
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parents[2]
BUILDER = ROOT / "scripts/build_mdbook_site.py"
CLI_REFERENCE = ROOT / "docs/cli/reference"
RUSTDOC_TARGET = ROOT / ".tmp/mdbook-rustdoc-target/doc"
STAGES = [
    "tool",
    "cli",
    "python",
    "mdbook",
    "mdbook-test",
    "rustdoc",
    "postprocess",
    "validate",
    "publish",
]


def _tree_digest(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        digest.update(path.relative_to(root).as_posix().encode())
        digest.update(path.read_bytes())
    return digest.hexdigest()


def _build(destination: Path, *, cwd: Path) -> None:
    subprocess.run(  # noqa: S603
        [sys.executable, BUILDER, "--site-dir", destination],
        cwd=cwd,
        check=True,
    )


def test_mdbook_builder_declares_complete_atomic_pipeline() -> None:
    result = subprocess.run(  # noqa: S603
        [sys.executable, BUILDER, "--list-stages"],
        cwd=ROOT.parent,
        check=True,
        capture_output=True,
        text=True,
    )
    assert result.stdout.splitlines() == STAGES


def test_summary_contains_every_generated_cli_chapter_in_hierarchy() -> None:
    summary = (ROOT / "docs/SUMMARY.md").read_text()
    positions = []
    for page in sorted(CLI_REFERENCE.glob("*.md")):
        target = f"cli/reference/{page.name}"
        assert summary.count(f"({target})") == 1
        positions.append(summary.index(f"({target})"))
    assert summary.index("(cli/reference/config.md)") < summary.index(
        "(cli/reference/config-path.md)"
    )
    assert summary.index("(cli/reference/completion.md)") < summary.index(
        "(cli/reference/completion-generate.md)"
    )
    assert len(positions) == len(set(positions))


def test_complete_artifact_is_reproducible_fresh_and_cwd_independent(
    tmp_path: Path,
) -> None:
    RUSTDOC_TARGET.mkdir(parents=True, exist_ok=True)
    rust_sentinel = RUSTDOC_TARGET / "sentinel.html"
    rust_sentinel.write_text("stale")
    first = tmp_path / "first"
    first.mkdir()
    (first / "stale.html").write_text("stale mdBook")
    _build(first, cwd=tmp_path)
    assert not rust_sentinel.exists()
    assert not (first / "stale.html").exists()
    assert (first / "rust/btpc_core/index.html").is_file()
    assert (first / "sitemap.xml").is_file()
    assert '<link rel="canonical"' in (first / "cli/reference/create.html").read_text()
    assert 'id="arguments"' in (first / "cli/reference/create.html").read_text()
    initial_digest = _tree_digest(first)
    subprocess.run(  # noqa: S603
        [sys.executable, ROOT / "scripts/postprocess_mdbook.py", "--site-dir", first],
        cwd=tmp_path,
        check=True,
    )
    assert _tree_digest(first) == initial_digest

    second = tmp_path / "second"
    _build(second, cwd=ROOT)
    assert _tree_digest(first) == _tree_digest(second)


def test_failed_build_preserves_preexisting_publication(tmp_path: Path) -> None:
    destination = tmp_path / "site"
    destination.mkdir()
    sentinel = destination / "sentinel.txt"
    sentinel.write_text("published")
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()
    fake = fake_bin / "mdbook"
    version = (ROOT / ".mdbook-version").read_text().strip()
    fake.write_text(
        "#!/bin/sh\n"
        f'if [ "$1" = "--version" ]; then echo "mdbook v{version}"; exit 0; fi\n'
        "exit 19\n"
    )
    fake.chmod(0o755)
    environment = dict(os.environ)
    environment["PATH"] = f"{fake_bin}{os.pathsep}{environment['PATH']}"
    result = subprocess.run(  # noqa: S603
        [sys.executable, BUILDER, "--site-dir", destination],
        cwd=tmp_path,
        env=environment,
        check=False,
    )
    assert result.returncode != 0
    assert sentinel.read_text() == "published"
