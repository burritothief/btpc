from __future__ import annotations

import os

from benches.btpc_bench.paths import (
    filesystem_path_document,
    filesystem_path_from_document,
)


def test_unix_path_document_distinguishes_colliding_lossy_names() -> None:
    if os.name == "nt":
        return
    first = os.fsdecode(b"name-\xff")
    second = os.fsdecode(b"name-\xfe")
    first_document = filesystem_path_document(first)
    second_document = filesystem_path_document(second)
    assert first_document["value"] != second_document["value"]
    assert "\\udcff" in first_document["display"]
    assert "\\udcfe" in second_document["display"]
    assert filesystem_path_from_document(first_document) == first
    assert filesystem_path_from_document(second_document) == second


def test_windows_utf16_document_decodes_edge_units() -> None:
    document = {
        "schema": "btpc.filesystem-path.v2",
        "display": "bad-\\ud800",
        "encoding": "windows-utf16",
        "value": [98, 97, 100, 45, 0xD800],
    }
    assert filesystem_path_from_document(document) == "bad-\ud800"


def test_path_display_escapes_control_characters() -> None:
    assert filesystem_path_document("line\nbreak")["display"] == "line\\nbreak"
