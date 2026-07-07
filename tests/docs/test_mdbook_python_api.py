from __future__ import annotations

import ast
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parents[2]
SCRIPT = ROOT / "scripts/mdbook_python_api.py"
PACKAGE = ROOT / "python/btpc"
MODULES = ("creation", "metainfo", "verification", "types", "errors")


def _exports(module: str) -> list[str]:
    tree = ast.parse((PACKAGE / f"{module}.py").read_text())
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == "__all__"
            for target in node.targets
        ):
            return ast.literal_eval(node.value)
    raise AssertionError(module)


def _protocol(module: str = "creation") -> list[object]:
    return [
        {"renderer": "html", "root": str(ROOT), "mdbook_version": "0.5.3"},
        {
            "items": [
                {
                    "Chapter": {
                        "name": "API",
                        "content": (
                            f"# API\n\n<!-- btpc-python-api: btpc.{module} -->\n"
                        ),
                        "number": [1],
                        "sub_items": [],
                        "path": f"python/reference/{module}.md",
                        "source_path": f"python/reference/{module}.md",
                        "parent_names": [],
                    }
                }
            ],
            "__non_exhaustive": None,
        },
    ]


def _run(payload: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run(  # noqa: S603
        [sys.executable, SCRIPT],
        input=json.dumps(payload),
        check=False,
        capture_output=True,
        text=True,
        cwd=ROOT,
    )


def test_preprocessor_supports_html_only_and_rejects_bad_input() -> None:
    supported = subprocess.run(  # noqa: S603
        [sys.executable, SCRIPT, "supports", "html"],
        check=False,
    )
    unsupported = subprocess.run(  # noqa: S603
        [sys.executable, SCRIPT, "supports", "not-html"],
        check=False,
    )
    malformed = subprocess.run(  # noqa: S603
        [sys.executable, SCRIPT],
        input="not-json",
        check=False,
        capture_output=True,
        text=True,
    )
    assert supported.returncode == 0
    assert unsupported.returncode == 1
    assert malformed.returncode == 1
    assert malformed.stdout == ""
    assert "invalid mdBook preprocessor JSON" in malformed.stderr


def test_preprocessor_is_deterministic_and_renders_representative_shapes() -> None:
    first = _run(_protocol("creation"))
    second = _run(_protocol("creation"))
    assert first.returncode == second.returncode == 0
    assert first.stdout == second.stdout
    content = json.loads(first.stdout)["items"][0]["Chapter"]["content"]
    assert '<a id="btpc.creation.CreateOptions"></a>' in content
    assert "class CreateOptions" in content
    assert "Create canonical metainfo without writing a torrent file." in content
    assert "Callable[[int, int, int], None] | None" in content
    assert "**Raises:**" in content
    assert "[`CancelledError`](errors.html#btpc.errors.CancelledError)" in content
    assert "```python" in content
    assert "_NativeCreateResultType" not in content

    metainfo = _run(_protocol("metainfo"))
    metainfo_content = json.loads(metainfo.stdout)["items"][0]["Chapter"]["content"]
    assert "Metainfo.from_bytes" in metainfo_content
    assert '<a id="btpc.metainfo.Metainfo.mode"></a>' in metainfo_content
    assert "ParseOptions | None = None" in metainfo_content

    types = _run(_protocol("types"))
    types_content = json.loads(types.stdout)["items"][0]["Chapter"]["content"]
    assert "class TorrentMode" in types_content
    assert '<a id="btpc.types.TorrentMode.V1"></a>' in types_content
    assert (
        "[`Metainfo.from_bytes`](metainfo.html#btpc.metainfo.Metainfo.from_bytes)"
        in types_content
    )

    errors = _run(_protocol("errors"))
    errors_content = json.loads(errors.stdout)["items"][0]["Chapter"]["content"]
    assert "class ResourceLimitError" in errors_content
    assert (
        '<a id="btpc.errors.ResourceLimitError.raise_exceeded"></a>' in errors_content
    )


def test_every_public_export_has_one_anchor_and_private_exports_have_none() -> None:
    all_content = []
    for module in MODULES:
        result = _run(_protocol(module))
        assert result.returncode == 0, result.stderr
        content = json.loads(result.stdout)["items"][0]["Chapter"]["content"]
        all_content.append(content)
        for symbol in _exports(module):
            assert content.count(f'<a id="btpc.{module}.{symbol}"></a>') == 1
    combined = "\n".join(all_content)
    assert "btpc._native" not in combined
    assert "btpc._conversion" not in combined
    assert '<a id="btpc.metainfo.Metainfo.from_bytes"></a>' in combined
    assert '<a id="btpc.metainfo.Metainfo.mode"></a>' in combined
    assert '<a id="btpc.verification.MismatchKind"></a>' in combined
    assert '<a id="btpc.errors.ResourceLimitError"></a>' in combined


def test_unknown_marker_and_duplicate_module_fail_without_stdout() -> None:
    unknown = _protocol()
    unknown[1]["items"][0]["Chapter"]["content"] = (
        "# API\n\n<!-- btpc-python-api: btpc.unknown -->\n"
    )
    duplicate = _protocol()
    duplicate[1]["items"].append(duplicate[1]["items"][0])
    for payload, message in [
        (unknown, "unknown BTPC Python API module"),
        (duplicate, "duplicate BTPC Python API module"),
    ]:
        result = _run(payload)
        assert result.returncode == 1
        assert result.stdout == ""
        assert message in result.stderr


def test_invalid_protocol_context_fails_without_stdout() -> None:
    payload = _protocol()
    del payload[0]["mdbook_version"]
    result = _run(payload)
    assert result.returncode == 1
    assert result.stdout == ""
    assert "context lacks mdbook_version" in result.stderr
