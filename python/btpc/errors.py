"""Public BTPC exception hierarchy."""

from __future__ import annotations

from typing import TYPE_CHECKING, Never

if TYPE_CHECKING:
    from pathlib import Path


class BtpcError(Exception):
    """Base error with structured core context."""

    def __init__(  # noqa: PLR0913
        self,
        message: str,
        *,
        offset: int | None = None,
        field: str | None = None,
        path: Path | None = None,
        limit: str | None = None,
        actual: int | None = None,
        maximum: int | None = None,
    ) -> None:
        """Initialize structured error context."""
        super().__init__(message)
        self.offset = offset
        self.field = field
        self.path = path
        self.limit = limit
        self.actual = actual
        self.maximum = maximum


class BencodeError(BtpcError):
    """Invalid bencode syntax or canonical form."""


class MetainfoError(BtpcError):
    """Invalid metainfo protocol fields."""


class PathError(BtpcError):
    """Filesystem or path failure."""


class VerificationError(BtpcError):
    """Payload verification mismatch."""


class UnsupportedError(BtpcError):
    """Unsupported feature or policy."""


class ResourceLimitError(BtpcError):
    """Configured parser or ownership resource limit was exceeded."""

    @classmethod
    def raise_exceeded(cls, limit: str, actual: int, maximum: int) -> Never:
        """Raise a structured limit error matching the native diagnostic."""
        message = f"resource limit exceeded for {limit}: {actual} > {maximum}"
        raise cls(
            message,
            limit=limit,
            actual=actual,
            maximum=maximum,
        )


class CancelledError(BtpcError):
    """Operation cancelled by the caller."""


__all__ = [
    "BencodeError",
    "BtpcError",
    "CancelledError",
    "MetainfoError",
    "PathError",
    "ResourceLimitError",
    "UnsupportedError",
    "VerificationError",
]
