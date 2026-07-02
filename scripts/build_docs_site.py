from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SITE_DIR = ROOT / "site"
STAGING_DIR = ROOT / ".tmp/docs-site"
RUSTDOC_TARGET_DIR = ROOT / ".tmp/docs-rustdoc-target"
STAGES = ("cli", "mkdocs", "rustdoc", "validate")
RUSTDOC_ENTRIES = (
    "btpc_core",
    "crates.js",
    "help.html",
    "search.index",
    "settings.html",
    "src-files.js",
    "static.files",
    "trait.impl",
    "type.impl",
)


def _run(command: tuple[str, ...], *, env: dict[str, str] | None = None) -> None:
    subprocess.run(command, cwd=ROOT, env=env, check=True)


def _prepare_output(site_dir: Path) -> Path:
    shutil.rmtree(STAGING_DIR, ignore_errors=True)
    shutil.rmtree(site_dir, ignore_errors=True)
    mkdocs_output = STAGING_DIR / "mkdocs"
    mkdocs_output.parent.mkdir(parents=True, exist_ok=True)
    return mkdocs_output


def _stage_cli() -> None:
    required = ROOT / "docs/reference/btpc.txt"
    if not required.is_file():
        message = f"missing generated CLI reference: {required}"
        raise RuntimeError(message)


def _stage_mkdocs(output: Path) -> None:
    _run(
        (
            sys.executable,
            "-m",
            "mkdocs",
            "build",
            "--strict",
            "--config-file",
            str(ROOT / "mkdocs.yml"),
            "--site-dir",
            str(output),
        )
    )


def _stage_rustdoc(output: Path) -> None:
    required = ROOT / "docs/rust/index.md"
    if not required.is_file():
        message = f"missing Rust documentation landing page: {required}"
        raise RuntimeError(message)

    rustdoc_root = RUSTDOC_TARGET_DIR / "doc"
    shutil.rmtree(rustdoc_root, ignore_errors=True)
    environment = dict(os.environ)
    environment["CARGO_TARGET_DIR"] = str(RUSTDOC_TARGET_DIR)
    environment["RUSTDOCFLAGS"] = "-D warnings"
    _run(
        (
            "cargo",
            "doc",
            "-p",
            "btpc-core",
            "--all-features",
            "--no-deps",
        ),
        env=environment,
    )

    rust_output = output / "rust"
    for entry in RUSTDOC_ENTRIES:
        source = rustdoc_root / entry
        if not source.exists():
            message = f"generated rustdoc is missing {source}"
            raise RuntimeError(message)
        destination = rust_output / entry
        if source.is_dir():
            shutil.copytree(source, destination)
        else:
            shutil.copy2(source, destination)

    source_tree = rustdoc_root / "src/btpc_core"
    if not source_tree.is_dir():
        message = f"generated rustdoc is missing {source_tree}"
        raise RuntimeError(message)
    shutil.copytree(source_tree, rust_output / "src/btpc_core")


def _stage_validate(output: Path) -> None:
    custom_not_found = output / "404/index.html"
    if custom_not_found.is_file():
        shutil.copyfile(custom_not_found, output / "404.html")
    for relative in ("index.html", "404.html"):
        if not (output / relative).is_file():
            message = f"generated documentation is missing {relative}"
            raise RuntimeError(message)


def build_site(site_dir: Path) -> None:
    output = _prepare_output(site_dir)
    _stage_cli()
    _stage_mkdocs(output)
    _stage_rustdoc(output)
    _stage_validate(output)
    shutil.copytree(output, site_dir)
    shutil.rmtree(STAGING_DIR)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Build the complete BTPC site")
    parser.add_argument(
        "--site-dir",
        type=Path,
        default=DEFAULT_SITE_DIR,
        help="explicit generated-site destination",
    )
    parser.add_argument(
        "--list-stages",
        action="store_true",
        help="print the reserved build stages and exit",
    )
    return parser


def main() -> int:
    arguments = _parser().parse_args()
    if arguments.list_stages:
        print("\n".join(STAGES))
        return 0
    build_site(arguments.site_dir.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
