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
CLI_REFERENCE_DIR = ROOT / "docs/cli/reference"
RUSTDOC_TARGET_DIR = ROOT / ".tmp/mdbook-rustdoc-target"
MDBOOK_TEST_TARGET_DIR = ROOT / ".tmp/mdbook-test-target"
STAGES = (
    "tool",
    "cli",
    "python",
    "mdbook",
    "mdbook-test",
    "rustdoc",
    "postprocess",
    "validate",
    "publish",
)
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


def _run(
    command: list[str | Path], *, environment: dict[str, str] | None = None
) -> None:
    subprocess.run(
        [str(argument) for argument in command],
        cwd=ROOT,
        env=environment,
        check=True,
    )


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


def _prepare_preprocessor(parent: Path) -> dict[str, str]:
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
    return environment


def _check_tool() -> None:
    _run([sys.executable, ROOT / "scripts/check_mdbook.py"])


def _check_cli(parent: Path) -> None:
    _run(["cargo", "build", "-p", "btpc-cli"])
    binary = ROOT / "target/debug/btpc"
    generated: list[Path] = []
    for index in range(2):
        destination = parent / f"cli-reference-{index}"
        _run([binary, "__generate-markdown", destination])
        generated.append(destination)
    expected_names = sorted(path.name for path in CLI_REFERENCE_DIR.glob("*.md"))
    for destination in generated:
        names = sorted(path.name for path in destination.glob("*.md"))
        if names != expected_names:
            raise RuntimeError("generated CLI website reference file set is stale")
    for name in expected_names:
        committed = (CLI_REFERENCE_DIR / name).read_bytes()
        first = (generated[0] / name).read_bytes()
        second = (generated[1] / name).read_bytes()
        if first != second:
            raise RuntimeError(f"CLI reference generation is nondeterministic: {name}")
        if committed != first:
            raise RuntimeError(f"generated CLI website reference is stale: {name}")


def _check_python_preprocessor() -> None:
    _run([sys.executable, ROOT / "scripts/mdbook_python_api.py", "supports", "html"])
    result = subprocess.run(
        [
            sys.executable,
            ROOT / "scripts/mdbook_python_api.py",
            "supports",
            "unsupported",
        ],
        cwd=ROOT,
        check=False,
    )
    if result.returncode != 1:
        raise RuntimeError("Python API preprocessor accepted an unsupported renderer")


def _build_book(book_root: Path, output: Path, environment: dict[str, str]) -> None:
    shutil.rmtree(output, ignore_errors=True)
    _run(
        ["mdbook", "build", book_root, "--dest-dir", output],
        environment=environment,
    )


def _test_book(book_root: Path, environment: dict[str, str]) -> None:
    shutil.rmtree(MDBOOK_TEST_TARGET_DIR, ignore_errors=True)
    cargo_environment = dict(os.environ)
    cargo_environment["CARGO_TARGET_DIR"] = str(MDBOOK_TEST_TARGET_DIR)
    _run(["cargo", "build", "-p", "btpc-core"], environment=cargo_environment)
    library_paths = ",".join(
        (
            str(MDBOOK_TEST_TARGET_DIR / "debug"),
            str(MDBOOK_TEST_TARGET_DIR / "debug/deps"),
        )
    )
    _run(
        ["mdbook", "test", book_root, "-L", library_paths],
        environment=environment,
    )


def _build_rustdoc(output: Path) -> None:
    shutil.rmtree(RUSTDOC_TARGET_DIR, ignore_errors=True)
    environment = dict(os.environ)
    environment["CARGO_TARGET_DIR"] = str(RUSTDOC_TARGET_DIR)
    environment["RUSTDOCFLAGS"] = "-D warnings"
    _run(
        ["cargo", "doc", "-p", "btpc-core", "--all-features", "--no-deps"],
        environment=environment,
    )
    rustdoc_root = RUSTDOC_TARGET_DIR / "doc"
    rust_output = output / "rust"
    rust_output.mkdir(exist_ok=True)
    for entry in RUSTDOC_ENTRIES:
        source = rustdoc_root / entry
        if not source.exists():
            raise RuntimeError(f"generated rustdoc is missing {source}")
        destination = rust_output / entry
        if source.is_dir():
            shutil.copytree(source, destination)
        else:
            shutil.copy2(source, destination)
    source_tree = rustdoc_root / "src/btpc_core"
    if not source_tree.is_dir():
        raise RuntimeError(f"generated rustdoc is missing {source_tree}")
    shutil.copytree(source_tree, rust_output / "src/btpc_core")


def _postprocess(output: Path) -> None:
    _run([sys.executable, ROOT / "scripts/postprocess_mdbook.py", "--site-dir", output])


def _validate(output: Path) -> None:
    _run([sys.executable, ROOT / "scripts/check_docs_site.py", output])


def _publish(staged: Path, destination: Path) -> None:
    backup = destination.with_name(f".{destination.name}.previous")
    shutil.rmtree(backup, ignore_errors=True)
    if destination.exists():
        destination.rename(backup)
    try:
        staged.rename(destination)
    except BaseException:
        if backup.exists():
            backup.rename(destination)
        raise
    shutil.rmtree(backup, ignore_errors=True)


def build_site(destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    (ROOT / ".tmp").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix="btpc-mdbook-source-", dir=ROOT / ".tmp"
    ) as source_temporary:
        source_parent = Path(source_temporary)
        book_root = _prepare_book_root(source_parent)
        environment = _prepare_preprocessor(source_parent)
        with tempfile.TemporaryDirectory(
            prefix=f".{destination.name}.staging-", dir=destination.parent
        ) as output_temporary:
            output = Path(output_temporary) / "site"
            _check_tool()
            _check_cli(source_parent)
            _check_python_preprocessor()
            _build_book(book_root, output, environment)
            _test_book(book_root, environment)
            _build_rustdoc(output)
            _postprocess(output)
            _validate(output)
            staged = Path(output_temporary) / "publication"
            output.rename(staged)
            _publish(staged, destination)


def main() -> int:
    parser = argparse.ArgumentParser(description="Build the complete mdBook site")
    parser.add_argument("--site-dir", type=Path)
    parser.add_argument("--list-stages", action="store_true")
    arguments = parser.parse_args()
    if arguments.list_stages:
        print("\n".join(STAGES))
        return 0
    if arguments.site_dir is None:
        parser.error("--site-dir is required")
    build_site(arguments.site_dir.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
