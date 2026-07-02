from __future__ import annotations

import argparse
import ast
import importlib
from pathlib import Path

IGNORED_RUNTIME = {
    "__all__",
    "__builtins__",
    "__cached__",
    "__doc__",
    "__file__",
    "__loader__",
    "__name__",
    "__package__",
    "__spec__",
}
IGNORED_STUB = {
    "__gil_required__",
    "__subinterpreters_supported__",
    "__version__",
}


def stub_exports(path: Path) -> set[str]:
    tree = ast.parse(path.read_text())
    return {
        node.name
        for node in tree.body
        if isinstance(node, (ast.ClassDef, ast.FunctionDef))
    } | {
        target.id
        for node in tree.body
        if isinstance(node, ast.AnnAssign)
        and isinstance((target := node.target), ast.Name)
    }


def validate(stub: Path) -> list[str]:
    runtime = importlib.import_module("btpc._native")
    runtime_names = {
        name for name in dir(runtime) if not name.startswith("__")
    } - IGNORED_RUNTIME
    declared = stub_exports(stub)
    missing = sorted(runtime_names - declared)
    stale = sorted(declared - runtime_names - IGNORED_STUB)
    errors = []
    if missing:
        errors.append(f"stub missing runtime exports: {missing}")
    if stale:
        errors.append(f"stub has stale exports: {stale}")
    return errors


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stub", type=Path, default=Path("python/btpc/_native.pyi"))
    arguments = parser.parse_args()
    errors = validate(arguments.stub)
    if errors:
        raise SystemExit("\n".join(errors))
    print("native stub exports match runtime")


if __name__ == "__main__":
    main()
