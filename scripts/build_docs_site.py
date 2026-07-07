from __future__ import annotations

import argparse
import ast
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SITE_DIR = ROOT / "site"
STAGING_DIR = ROOT / ".tmp/docs-site"
RUSTDOC_TARGET_DIR = ROOT / ".tmp/docs-rustdoc-target"
CLI_REFERENCE_DIR = ROOT / "docs/cli/reference"
PYTHON_PACKAGE_DIR = ROOT / "python/btpc"
STAGES = ("cli", "mkdocs", "rustdoc", "validate")
PAGES_ROOT = "/btpc/"
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
    _run(("cargo", "build", "-p", "btpc-cli"))
    generated = STAGING_DIR / "cli-reference"
    _run(
        (
            str(ROOT / "target/debug/btpc"),
            "__generate-markdown",
            str(generated),
        )
    )
    expected_files = sorted(path.name for path in CLI_REFERENCE_DIR.glob("*.md"))
    generated_files = sorted(path.name for path in generated.glob("*.md"))
    if generated_files != expected_files:
        message = "generated CLI website reference file set is stale"
        raise RuntimeError(message)
    for name in expected_files:
        if (CLI_REFERENCE_DIR / name).read_bytes() != (generated / name).read_bytes():
            message = f"generated CLI website reference is stale: {name}"
            raise RuntimeError(message)


def _stage_mkdocs(output: Path) -> None:
    source = STAGING_DIR / "mkdocs-source"
    shutil.copytree(ROOT / "docs", source)
    for page in source.joinpath("python/reference").glob("*.md"):
        module = page.stem
        tree = ast.parse((PYTHON_PACKAGE_DIR / f"{module}.py").read_text())
        exports: list[str] | None = None
        for node in tree.body:
            if isinstance(node, ast.Assign) and any(
                isinstance(target, ast.Name) and target.id == "__all__"
                for target in node.targets
            ):
                exports = ast.literal_eval(node.value)
                break
        if exports is None:
            raise RuntimeError(f"missing __all__ for btpc.{module}")
        marker = f"<!-- btpc-python-api: btpc.{module} -->"
        directive = "\n".join(
            [f"::: btpc.{module}", "    options:", "      members:"]
            + [f"        - {symbol}" for symbol in exports]
        )
        content = page.read_text()
        if content.count(marker) != 1:
            raise RuntimeError(f"missing unique BTPC Python API marker in {page}")
        page.write_text(content.replace(marker, directive))
    config = STAGING_DIR / "mkdocs.yml"
    values = yaml.safe_load((ROOT / "mkdocs.yml").read_text())
    values["docs_dir"] = str(source)
    values["theme"]["custom_dir"] = str(source / "overrides")
    values["plugins"][1]["mkdocstrings"]["handlers"]["python"]["paths"] = [
        str(ROOT / "python")
    ]
    config.write_text(yaml.safe_dump(values, sort_keys=False))
    _run(
        (
            sys.executable,
            "-m",
            "mkdocs",
            "build",
            "--strict",
            "--config-file",
            str(config),
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
        content = custom_not_found.read_text()
        content = re.sub(
            r'(?P<attribute>href|src|action)="\.\./',
            rf'\g<attribute>="{PAGES_ROOT}',
            content,
        )
        content = re.sub(
            r'(?P<attribute>href|src|action)="\.\."',
            rf'\g<attribute>="{PAGES_ROOT}"',
            content,
        )
        content = content.replace(
            'new URL("..",location)', f'new URL("{PAGES_ROOT}",location)'
        )
        (output / "404.html").write_text(content)
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
