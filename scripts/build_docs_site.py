from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SITE_DIR = ROOT / "site"
STAGING_DIR = ROOT / ".tmp/docs-site"
STAGES = ("cli", "mkdocs", "rustdoc", "validate")


def _run(command: tuple[str, ...]) -> None:
    subprocess.run(command, cwd=ROOT, check=True)


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


def _stage_rustdoc() -> None:
    required = ROOT / "docs/rust/index.md"
    if not required.is_file():
        message = f"missing Rust documentation landing page: {required}"
        raise RuntimeError(message)


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
    _stage_rustdoc()
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
