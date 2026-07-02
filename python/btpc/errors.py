"""Catch structured errors from parsing, creation, editing, and verification."""

from __future__ import annotations

from typing import TYPE_CHECKING, Never

if TYPE_CHECKING:
    from pathlib import Path


class BtpcError(Exception):
    """Base class for stable BTPC error categories.

    Inspect the optional attributes instead of parsing the human-readable message.
    Only attributes relevant to a particular failure are populated.

    Attributes:
        offset: Encoded byte offset for syntax or canonicalization failures.
        field: Protocol field associated with a validation failure.
        path: Filesystem path associated with an I/O or safety failure.
        limit: Resource-limit name.
        actual: Observed resource value.
        maximum: Configured maximum resource value.
    """

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
    """Report malformed bencode syntax or an invalid canonical encoding."""


class MetainfoError(BtpcError):
    """Report torrent fields that violate v1, v2, or hybrid protocol rules."""


class PathError(BtpcError):
    """Report filesystem I/O, unsafe path, traversal, or destination failures."""


class VerificationError(BtpcError):
    """Report an operational failure that prevents payload verification."""


class UnsupportedError(BtpcError):
    """Report a requested feature or policy that BTPC does not support."""


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
    """Report cooperative cancellation requested through a cancellation token."""


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
