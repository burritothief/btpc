from __future__ import annotations

import argparse
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).parents[1]


def _strip_frontmatter(path: Path) -> None:
    text = path.read_text()
    if not text.startswith("---\n"):
        return
    end = text.find("\n---\n", 4)
    if end < 0:
        raise RuntimeError(f"unterminated Markdown front matter: {path}")
    path.write_text(text[end + 5 :])


def _prepare_book_root(parent: Path) -> Path:
    book_root = parent / "book"
    book_root.mkdir()
    shutil.copy2(ROOT / "book.toml", book_root / "book.toml")
    shutil.copytree(ROOT / "docs", book_root / "docs")
    for path in book_root.joinpath("docs").rglob("*.md"):
        _strip_frontmatter(path)
    return book_root


def _prepare_preprocessor(parent: Path) -> tuple[Path, dict[str, str]]:
    executable_dir = parent / "bin"
    executable_dir.mkdir()
    executable = executable_dir / "btpc-mdbook-python-api"
    executable.write_text(
        "#!/bin/sh\n"
        f"exec {shlex.quote(sys.executable)} "
        f'{shlex.quote(str(ROOT / "scripts/mdbook_python_api.py"))} "$@"\n'
    )
    executable.chmod(0o755)
    environment = dict(os.environ)
    environment["PATH"] = f"{executable_dir}{os.pathsep}{environment['PATH']}"
    return executable, environment


def main() -> int:
    parser = argparse.ArgumentParser(description="Build the side-by-side mdBook site")
    parser.add_argument("--site-dir", type=Path, required=True)
    arguments = parser.parse_args()
    subprocess.run(
        [sys.executable, ROOT / "scripts/check_mdbook.py"], cwd=ROOT, check=True
    )
    destination = (ROOT / arguments.site_dir).resolve()
    shutil.rmtree(destination, ignore_errors=True)
    temporary_parent = ROOT / ".tmp"
    temporary_parent.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix="mdbook-source-", dir=temporary_parent
    ) as temporary:
        book_root = _prepare_book_root(Path(temporary))
        _, environment = _prepare_preprocessor(Path(temporary))
        subprocess.run(
            ["mdbook", "build", str(book_root), "--dest-dir", str(destination)],
            cwd=ROOT,
            env=environment,
            check=True,
        )
    subprocess.run(
        [
            sys.executable,
            ROOT / "scripts/postprocess_mdbook.py",
            "--site-dir",
            destination,
        ],
        cwd=ROOT,
        check=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
