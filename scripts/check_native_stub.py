from __future__ import annotations

import argparse
import ast
import importlib
import inspect
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


def declared_parameters(node: ast.FunctionDef) -> tuple[tuple[str, str, bool], ...]:
    positional = node.args.posonlyargs + node.args.args
    required = len(positional) - len(node.args.defaults)
    result = [
        (
            argument.arg,
            "positional_only" if index < len(node.args.posonlyargs) else "positional",
            index < required,
        )
        for index, argument in enumerate(positional)
        if argument.arg not in {"self", "cls"}
    ]
    result.extend(
        (argument.arg, "keyword", default is None)
        for argument, default in zip(
            node.args.kwonlyargs, node.args.kw_defaults, strict=True
        )
    )
    if node.args.vararg is not None:
        result.append((node.args.vararg.arg, "variadic", False))
    if node.args.kwarg is not None:
        result.append((node.args.kwarg.arg, "keywords", False))
    return tuple(result)


def check_signature(node: ast.FunctionDef, runtime: object, name: str) -> list[str]:
    signature = inspect.signature(runtime)
    kinds = {
        inspect.Parameter.POSITIONAL_ONLY: "positional_only",
        inspect.Parameter.POSITIONAL_OR_KEYWORD: "positional",
        inspect.Parameter.KEYWORD_ONLY: "keyword",
        inspect.Parameter.VAR_POSITIONAL: "variadic",
        inspect.Parameter.VAR_KEYWORD: "keywords",
    }
    actual = tuple(
        (
            parameter.name,
            kinds[parameter.kind],
            parameter.kind
            not in {inspect.Parameter.VAR_POSITIONAL, inspect.Parameter.VAR_KEYWORD}
            and parameter.default is inspect.Parameter.empty,
        )
        for parameter in signature.parameters.values()
        if parameter.name not in {"self", "cls"}
    )
    expected = declared_parameters(node)
    if actual != expected:
        return [f"stub signature mismatch for {name}: {expected!r} != {actual!r}"]
    return []


def check_class(node: ast.ClassDef, runtime: type[object]) -> list[str]:
    members = {
        item.name: item for item in node.body if isinstance(item, ast.FunctionDef)
    }
    runtime_members = {name for name in vars(runtime) if not name.startswith("_")}
    errors = [
        f"stub missing runtime member: {node.name}.{name}"
        for name in sorted(runtime_members - members.keys())
    ]
    for name, member in members.items():
        if name.startswith("_"):
            continue
        if name not in runtime_members:
            errors.append(f"stub has stale member: {node.name}.{name}")
            continue
        descriptor = inspect.getattr_static(runtime, name)
        is_property = any(
            isinstance(item, ast.Name) and item.id == "property"
            for item in member.decorator_list
        )
        runtime_property = inspect.isgetsetdescriptor(descriptor) or isinstance(
            descriptor, property
        )
        if is_property != runtime_property:
            errors.append(f"stub property mismatch: {node.name}.{name}")
        elif not is_property:
            errors.extend(check_signature(member, descriptor, f"{node.name}.{name}"))
    return errors


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
    for node in ast.parse(stub.read_text()).body:
        value = getattr(runtime, getattr(node, "name", ""), None)
        if value is None:
            continue
        if isinstance(node, ast.FunctionDef):
            errors.extend(check_signature(node, value, node.name))
        elif isinstance(node, ast.ClassDef):
            errors.extend(check_class(node, value))
    return errors


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stub", type=Path)
    arguments = parser.parse_args()
    runtime = importlib.import_module("btpc._native")
    stub = arguments.stub or Path(runtime.__file__).with_name("_native.pyi")
    errors = validate(stub)
    if errors:
        raise SystemExit("\n".join(errors))
    print("native stub exports and signatures match runtime")


if __name__ == "__main__":
    main()
