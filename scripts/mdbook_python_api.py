from __future__ import annotations

import ast
import json
import re
import sys
from pathlib import Path
from typing import Any

from griffe import Alias, GriffeLoader, Object

ROOT = Path(__file__).parents[1]
PACKAGE = ROOT / "python/btpc"
MODULES = ("creation", "metainfo", "verification", "types", "errors")
MARKER = re.compile(r"<!--\s*btpc-python-api:\s*btpc\.([a-z_]+)\s*-->")
ROLE = re.compile(r":(?:class|func|meth|attr|exc):`([^`]+)`")
SECTIONS = {"Args", "Returns", "Raises", "Attributes", "Examples", "Notes"}
PROTOCOL_PARTS = 2
SUPPORTS_ARGUMENTS = 3


class PreprocessorError(RuntimeError):
    """Report an actionable static API generation error."""


def _exports(module: str) -> list[str]:
    tree = ast.parse((PACKAGE / f"{module}.py").read_text())
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == "__all__"
            for target in node.targets
        ):
            value = ast.literal_eval(node.value)
            if not isinstance(value, list) or not all(
                isinstance(item, str) for item in value
            ):
                raise PreprocessorError(f"invalid __all__ for btpc.{module}")
            if len(value) != len(set(value)):
                raise PreprocessorError(f"duplicate symbols in btpc.{module}.__all__")
            return value
    raise PreprocessorError(f"missing __all__ for btpc.{module}")


def _inventory() -> tuple[dict[str, str], dict[str, list[str]]]:
    definitions: dict[str, str] = {}
    exports: dict[str, list[str]] = {}
    for module in MODULES:
        exports[module] = _exports(module)
        for symbol in exports[module]:
            if symbol in definitions:
                raise PreprocessorError(f"duplicate public symbol: {symbol}")
            definitions[symbol] = module
    return definitions, exports


def _target(
    reference: str, *, module: str, owner: str | None, definitions: dict[str, str]
) -> tuple[str, str]:
    reference = reference.removeprefix("btpc.")
    parts = reference.split(".")
    symbol = parts[0]
    if symbol in definitions:
        target_module = definitions[symbol]
        anchor = f"btpc.{target_module}.{reference}"
    elif owner is not None and len(parts) == 1:
        target_module = module
        anchor = f"{owner}.{reference}"
    else:
        raise PreprocessorError(
            f"unresolved public BTPC cross-reference {reference!r} in btpc.{module}"
        )
    href = f"#{anchor}" if target_module == module else f"{target_module}.html#{anchor}"
    return reference, href


def _inline(
    text: str, *, module: str, owner: str | None, definitions: dict[str, str]
) -> str:
    def replace(match: re.Match[str]) -> str:
        label, href = _target(
            match.group(1), module=module, owner=owner, definitions=definitions
        )
        return f"[`{label}`]({href})"

    text = ROLE.sub(replace, text)
    return text.replace("``", "`")


def _docstring(
    value: str | None, *, module: str, owner: str | None, definitions: dict[str, str]
) -> str:
    if not value:
        return ""
    lines = value.strip().splitlines()
    output: list[str] = []
    in_examples = False
    for line in lines:
        stripped = line.strip()
        if (
            stripped.endswith(":")
            and stripped[:-1] in SECTIONS
            and not line.startswith(" ")
        ):
            if in_examples:
                output.append("```")
                in_examples = False
            section = stripped[:-1]
            output.extend(["", f"**{section}:**", ""])
            if section == "Examples":
                output.append("```python")
                in_examples = True
            continue
        converted = _inline(
            stripped, module=module, owner=owner, definitions=definitions
        )
        if in_examples:
            output.append(converted)
        elif line.startswith("        ") and output:
            output.append(f"  {converted}")
        elif line.startswith("    ") and ":" in stripped:
            name, description = stripped.split(":", 1)
            output.append(f"- **{name}:**{description}")
        else:
            output.append(converted)
    if in_examples:
        output.append("```")
    return "\n".join(output).strip()


def _signature(obj: Object | Alias, name: str) -> str:
    if not hasattr(obj, "signature"):
        return name
    try:
        signature = str(obj.signature())
    except (AttributeError, TypeError, ValueError):
        return name
    if not signature or "_Native" in signature or "_native" in signature:
        return name
    return signature


def _kind(obj: Object | Alias) -> str:
    return str(getattr(getattr(obj, "kind", None), "value", "object"))


def _render_member(
    obj: Object | Alias,
    *,
    module: str,
    path: str,
    level: int,
    definitions: dict[str, str],
) -> list[str]:
    name = path.rsplit(".", 1)[-1]
    anchor = f"btpc.{module}.{path}"
    lines = [f'<a id="{anchor}"></a>', f"{'#' * level} `{name}`", ""]
    kind = _kind(obj)
    if kind == "class":
        owner = anchor
    elif "." in path:
        owner = anchor.rsplit(".", 1)[0]
    else:
        owner = None
    if kind in {"function", "class"}:
        prefix = "class " if kind == "class" else ""
        lines.extend(["```python", f"{prefix}{_signature(obj, name)}", "```", ""])
    elif annotation := getattr(obj, "annotation", None):
        lines.extend(["```python", f"{name}: {annotation}", "```", ""])
    documentation = _docstring(
        obj.docstring.value if getattr(obj, "docstring", None) else None,
        module=module,
        owner=owner,
        definitions=definitions,
    )
    if documentation:
        lines.extend([documentation, ""])
    if kind == "class":
        for member_name, member in obj.members.items():
            if member_name.startswith("_"):
                continue
            member_kind = _kind(member)
            if member_kind not in {"function", "attribute"}:
                raise PreprocessorError(
                    f"unsupported public object shape {member_kind!r} for {anchor}.{member_name}"
                )
            lines.extend(
                _render_member(
                    member,
                    module=module,
                    path=f"{path}.{member_name}",
                    level=min(level + 1, 6),
                    definitions=definitions,
                )
            )
    return lines


def render_module(module: str) -> str:
    if module not in MODULES:
        raise PreprocessorError(f"unknown BTPC Python API module: btpc.{module}")
    definitions, exports = _inventory()
    loaded = GriffeLoader(search_paths=[PACKAGE.parent], allow_inspection=False).load(
        f"btpc.{module}"
    )
    lines = [f'<a id="btpc.{module}"></a>', f"## `btpc.{module}`", ""]
    module_doc = _docstring(
        loaded.docstring.value if loaded.docstring else None,
        module=module,
        owner=None,
        definitions=definitions,
    )
    if module_doc:
        lines.extend([module_doc, ""])
    for symbol in exports[module]:
        if symbol not in loaded.members:
            raise PreprocessorError(
                f"Griffe did not find public symbol btpc.{module}.{symbol}"
            )
        lines.extend(
            _render_member(
                loaded.members[symbol],
                module=module,
                path=symbol,
                level=3,
                definitions=definitions,
            )
        )
    rendered = "\n".join(lines).rstrip() + "\n"
    if any(
        value in rendered for value in ("btpc._native", "btpc._conversion", str(ROOT))
    ):
        raise PreprocessorError(
            f"private implementation detail leaked from btpc.{module}"
        )
    return rendered


def _chapters(items: list[dict[str, Any]]) -> list[dict[str, Any]]:
    chapters: list[dict[str, Any]] = []
    for item in items:
        chapter = item.get("Chapter")
        if chapter is None:
            continue
        chapters.append(chapter)
        chapters.extend(_chapters(chapter.get("sub_items", [])))
    return chapters


def preprocess(payload: object) -> dict[str, Any]:
    if not isinstance(payload, list) or len(payload) != PROTOCOL_PARTS:
        raise PreprocessorError("mdBook preprocessor input must be [context, book]")
    context, book = payload
    if not isinstance(context, dict) or not isinstance(
        context.get("mdbook_version"), str
    ):
        raise PreprocessorError("mdBook preprocessor context lacks mdbook_version")
    if context.get("renderer") != "html":
        raise PreprocessorError(
            "BTPC Python API preprocessor supports only the html renderer"
        )
    if not isinstance(book, dict):
        raise PreprocessorError("mdBook preprocessor input lacks book items")
    items = book.get("items", book.get("sections"))
    if not isinstance(items, list):
        raise PreprocessorError("mdBook preprocessor input lacks book items")
    seen: set[str] = set()
    for chapter in _chapters(items):
        content = chapter.get("content")
        if not isinstance(content, str):
            raise PreprocessorError("mdBook chapter content must be text")
        matches = MARKER.findall(content)
        if len(matches) > 1:
            raise PreprocessorError("chapter contains multiple BTPC Python API markers")
        if not matches:
            continue
        module = matches[0]
        if module not in MODULES:
            raise PreprocessorError(f"unknown BTPC Python API module: btpc.{module}")
        if module in seen:
            raise PreprocessorError(f"duplicate BTPC Python API module: btpc.{module}")
        seen.add(module)
        chapter["content"] = MARKER.sub(render_module(module), content)
    return book


def main() -> int:
    if len(sys.argv) == SUPPORTS_ARGUMENTS and sys.argv[1] == "supports":
        return 0 if sys.argv[2] == "html" else 1
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        print(f"invalid mdBook preprocessor JSON: {error}", file=sys.stderr)
        return 1
    try:
        result = preprocess(payload)
    except PreprocessorError as error:
        print(str(error), file=sys.stderr)
        return 1
    json.dump(result, sys.stdout, separators=(",", ":"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
