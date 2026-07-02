"""Payload verification values and functions."""

from __future__ import annotations

import os
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import TYPE_CHECKING

from . import _native
from ._conversion import _convert_error

if TYPE_CHECKING:
    from collections.abc import Callable
    from os import PathLike

    from .creation import CancellationToken
    from .metainfo import Metainfo


class MismatchKind(Enum):
    """Stable payload verification mismatch category."""

    MISSING = "missing"
    WRONG_SIZE = "wrong_size"
    EXTRA = "extra"
    UNSAFE_PATH = "unsafe_path"
    V1_HASH = "v1_hash"
    V2_HASH = "v2_hash"


@dataclass(frozen=True, slots=True)
class PayloadMismatch:
    """One deterministic payload mismatch."""

    kind: MismatchKind
    path: Path
    piece: int | None


@dataclass(frozen=True, slots=True)
class PayloadVerificationReport:
    """Completed payload verification report."""

    mismatches: tuple[PayloadMismatch, ...]

    @property
    def is_valid(self) -> bool:
        """Return whether every enabled check passed."""
        return not self.mismatches


def verify(  # noqa: PLR0913
    metainfo: Metainfo,
    payload: str | PathLike[str],
    *,
    fail_fast: bool = False,
    extra_files: bool = False,
    progress: Callable[[int, int, int], None] | None = None,
    cancellation: CancellationToken | None = None,
) -> PayloadVerificationReport:
    """Verify a payload using the native core verifier."""
    try:
        value = metainfo._native.verify(  # noqa: SLF001
            Path(payload),
            fail_fast,
            extra_files,
            progress,
            cancellation._native if cancellation is not None else None,  # noqa: SLF001
        )
    except _native._NativeError as error:  # noqa: SLF001
        raise _convert_error(error) from None
    return PayloadVerificationReport(
        mismatches=tuple(
            PayloadMismatch(
                kind=MismatchKind(item.kind),
                path=Path(os.fsdecode(item.path)),
                piece=item.piece,
            )
            for item in value.mismatches
        )
    )


__all__ = ["MismatchKind", "PayloadMismatch", "PayloadVerificationReport", "verify"]
