"""Shared immutable BTPC value types."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import cast


class UnchangedType(Enum):
    """Singleton edit marker preserving the current field value."""

    VALUE = "UNCHANGED"

    def __repr__(self) -> str:
        """Return the public sentinel spelling."""
        return "UNCHANGED"


UNCHANGED = UnchangedType.VALUE


class TorrentMode(Enum):
    """Validated BitTorrent protocol representation."""

    V1 = "v1"
    V2 = "v2"
    HYBRID = "hybrid"


@dataclass(frozen=True, slots=True)
class ParseOptions:
    """Advanced metainfo loading limits; omitted values use safe core defaults."""

    max_total_input: int | None = None
    max_owned_allocation: int | None = None
    max_integer_digits: int | None = None


@dataclass(frozen=True, slots=True)
class HashValue:
    """Immutable raw hash value."""

    bytes: bytes

    @property
    def hex(self) -> str:
        """Return lowercase hexadecimal text."""
        return self.bytes.hex()

    def __str__(self) -> str:
        """Return lowercase hexadecimal text."""
        return self.hex


@dataclass(frozen=True, order=True, slots=True)
class TorrentBytes:
    """Immutable torrent byte string with raw-byte identity."""

    raw: bytes

    def __post_init__(self) -> None:
        """Reject non-bytes values instead of coercing identity."""
        raw = cast("object", self.raw)
        if not isinstance(raw, bytes):
            message = "raw must be bytes"
            raise TypeError(message)

    @property
    def text(self) -> str | None:
        """Return UTF-8 text only when decoding is lossless."""
        try:
            return self.raw.decode()
        except UnicodeDecodeError:
            return None


@dataclass(frozen=True, order=True, slots=True)
class TorrentPath:
    """Immutable torrent path ordered and compared by raw components."""

    components: tuple[TorrentBytes, ...]

    def __post_init__(self) -> None:
        """Reject components unsafe in torrent paths."""
        if not self.components:
            message = "torrent path must contain at least one component"
            raise ValueError(message)
        for component in self.components:
            raw = component.raw
            if (
                not raw
                or raw in {b".", b".."}
                or b"/" in raw
                or b"\\" in raw
                or b"\0" in raw
            ):
                message = "unsafe torrent path component"
                raise ValueError(message)

    @property
    def text(self) -> tuple[str, ...] | None:
        """Return decoded components only when every component is UTF-8."""
        decoded = tuple(component.text for component in self.components)
        if any(component is None for component in decoded):
            return None
        return cast("tuple[str, ...]", decoded)

    def to_path(self) -> Path | None:
        """Return a platform path only when every component decodes safely."""
        decoded = self.text
        if decoded is None:
            return None
        return Path(*decoded)


__all__ = [
    "UNCHANGED",
    "HashValue",
    "ParseOptions",
    "TorrentBytes",
    "TorrentMode",
    "TorrentPath",
]
