"""Verify payload files against v1 pieces and v2 Merkle roots."""

from __future__ import annotations

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
    """Describe one deterministic payload mismatch.

    Attributes:
        kind: Stable mismatch category.
        path: Payload-relative filesystem path.
        piece: Zero-based v1 piece index when the mismatch is piece-specific.
    """

    kind: MismatchKind
    path: Path
    piece: int | None


@dataclass(frozen=True, slots=True)
class PayloadVerificationReport:
    """Collect deterministic mismatches from a completed verification.

    Attributes:
        mismatches: Mismatches in deterministic path and piece order.
    """

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
    r"""Verify payload content against all hashes represented by metainfo.

    ``payload`` points directly to a single-file payload or to the root directory of
    a multi-file torrent. v1 verification hashes the logical concatenated file
    stream; v2 verification checks each file's Merkle root; hybrid torrents check
    both. Content mismatches are returned, while unsafe paths and operational I/O
    failures raise exceptions.

    Args:
        metainfo: Parsed torrent describing the expected payload.
        payload: Payload file or root directory.
        fail_fast: Stop after the first mismatch when true.
        extra_files: Include files absent from the torrent as mismatches.
        progress: Optional callback receiving ``(completed_bytes, total_bytes,
            completed_pieces)``. Callback exceptions propagate unchanged.
        cancellation: Cooperative cancellation token.

    Returns:
        A report containing zero or more deterministic mismatches.

    Raises:
        PathError: If a required payload path cannot be read safely.
        VerificationError: If verification cannot be completed under the selected
            policy.
        CancelledError: If ``cancellation`` is requested.
        Exception: Any exception raised by ``progress``.

    Examples:
        >>> from pathlib import Path
        >>> from tempfile import TemporaryDirectory
        >>> from btpc import CreateOptions, Metainfo, create_bytes, verify
        >>> with TemporaryDirectory() as directory:
        ...     payload = Path(directory) / "hello.txt"
        ...     _ = payload.write_bytes(b"hello torrent\\n")
        ...     created = create_bytes(
        ...         payload,
        ...         options=CreateOptions(creation_date=0, threads=1),
        ...     )
        ...     torrent = Metainfo.from_bytes(created.bytes)
        ...     assert verify(torrent, payload).is_valid
    """
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
                path=Path(item.path),
                piece=item.piece,
            )
            for item in value.mismatches
        )
    )


__all__ = ["MismatchKind", "PayloadMismatch", "PayloadVerificationReport", "verify"]
