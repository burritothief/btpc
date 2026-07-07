"""Lossless cross-platform filesystem path JSON helpers."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

PATH_SCHEMA = "btpc.filesystem-path.v2"


def _safe_display(value: str) -> str:
    return "".join(
        character.encode("unicode_escape").decode("ascii")
        if character.isprintable() is False
        else character
        for character in value
    )


def filesystem_path_document(path: str | os.PathLike[str]) -> dict[str, Any]:
    """Encode a filesystem path without losing Unix bytes or Windows UTF-16 units."""
    value = os.fspath(path)
    if os.name == "nt":
        encoded = value.encode("utf-16-le", errors="surrogatepass")
        units = [
            int.from_bytes(encoded[index : index + 2], "little")
            for index in range(0, len(encoded), 2)
        ]
        return {
            "schema": PATH_SCHEMA,
            "display": _safe_display(value),
            "encoding": "windows-utf16",
            "value": units,
        }
    return {
        "schema": PATH_SCHEMA,
        "display": _safe_display(value),
        "encoding": "unix-bytes-hex",
        "value": os.fsencode(value).hex(),
    }


def filesystem_path_from_document(document: object) -> str:
    """Decode a v2 path object, accepting legacy display strings unchanged."""
    if isinstance(document, str):
        return document
    if not isinstance(document, dict) or document.get("schema") != PATH_SCHEMA:
        msg = "invalid filesystem path document"
        raise ValueError(msg)
    encoding = document.get("encoding")
    value = document.get("value")
    if encoding == "unix-bytes-hex" and isinstance(value, str):
        return os.fsdecode(bytes.fromhex(value))
    if encoding == "windows-utf16" and isinstance(value, list):
        encoded = b"".join(int(unit).to_bytes(2, "little") for unit in value)
        return encoded.decode("utf-16-le", errors="surrogatepass")
    msg = f"unsupported filesystem path encoding: {encoding!r}"
    raise ValueError(msg)


def path_name(path: str) -> str:
    """Return the final component without requiring the path to exist."""
    return Path(path).name
