"""Torrent creation options, results, cancellation, and functions."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Never, SupportsIndex, cast

from . import _native
from ._conversion import (
    _convert_error,
    _node_bytes,
    _string_sequence_bytes,
    _text_bytes,
    _tracker_bytes,
)
from .types import HashValue, TorrentMode

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence
    from os import PathLike

    from ._native import _NativeCreateResult as _NativeCreateResultType


@dataclass(frozen=True, slots=True)
class CreateOptions:
    """Typed creation options matching core defaults."""

    # Spec: PYAPI-TYPE-COMPLETE-001

    mode: TorrentMode = TorrentMode.V1
    piece_length: int | None = None
    threads: int | None = None
    trackers: Sequence[Sequence[str]] = ()
    web_seeds: Sequence[str] = ()
    nodes: Sequence[tuple[str, int]] = ()
    private: bool | None = None
    source: str | None = None
    comment: str | None = None
    created_by: str | None = None
    omit_created_by: bool = False
    creation_date: int | None = None


@dataclass(frozen=True, slots=True)
class CreateMetrics:
    """Per-phase elapsed milliseconds."""

    scan_ms: float
    hash_ms: float
    serialize_ms: float


class CreateResult:
    """Immutable lazy facade over a native creation result."""

    __slots__ = (
        "_bytes_cache",
        "_info_hash_v1_cache",
        "_info_hash_v2_cache",
        "_metrics_cache",
        "_native",
    )
    _native: _NativeCreateResultType
    _bytes_cache: bytes | None
    _info_hash_v1_cache: HashValue | bool | None
    _info_hash_v2_cache: HashValue | bool | None
    _metrics_cache: CreateMetrics | None

    def __init__(self, native: _NativeCreateResultType) -> None:
        """Own the native result without copying generated metainfo bytes."""
        object.__setattr__(self, "_native", native)
        object.__setattr__(self, "_bytes_cache", None)
        object.__setattr__(self, "_info_hash_v1_cache", None)
        object.__setattr__(self, "_info_hash_v2_cache", None)
        object.__setattr__(self, "_metrics_cache", None)

    def __setattr__(self, _name: str, _value: object) -> Never:
        """Keep creation results immutable."""
        message = "CreateResult is immutable"
        raise AttributeError(message)

    def __init_subclass__(cls, **_kwargs: object) -> Never:
        """Keep storage and invariants under BTPC control."""
        message = "CreateResult does not support subclassing"
        raise TypeError(message)

    def __reduce_ex__(self, _protocol: SupportsIndex) -> Never:
        """Reject pickling until a stable serialized-object policy exists."""
        message = "CreateResult objects are not picklable; serialize the bytes property"
        raise TypeError(message)

    @property
    def bytes(self) -> bytes:
        """Return generated metainfo bytes, copied on first request."""
        cached = self._bytes_cache
        if cached is None:
            cached = self._native.bytes
            object.__setattr__(self, "_bytes_cache", cached)
        return cached

    @property
    def mode(self) -> TorrentMode:
        """Return the created protocol mode."""
        return TorrentMode(self._native.mode)

    @property
    def info_hash_v1(self) -> HashValue | None:
        """Return the cached v1 info hash when applicable."""
        cached = self._info_hash_v1_cache
        if cached is None:
            value = self._native.info_hash_v1
            cached = HashValue(value) if value is not None else False
            object.__setattr__(self, "_info_hash_v1_cache", cached)
        return None if cached is False else cast("HashValue", cached)

    @property
    def info_hash_v2(self) -> HashValue | None:
        """Return the cached v2 info hash when applicable."""
        cached = self._info_hash_v2_cache
        if cached is None:
            value = self._native.info_hash_v2
            cached = HashValue(value) if value is not None else False
            object.__setattr__(self, "_info_hash_v2_cache", cached)
        return None if cached is False else cast("HashValue", cached)

    @property
    def file_count(self) -> int:
        """Return payload file count."""
        return self._native.file_count

    @property
    def payload_bytes(self) -> int:
        """Return total payload bytes."""
        return self._native.payload_bytes

    @property
    def piece_count(self) -> int:
        """Return created piece count."""
        return self._native.piece_count

    @property
    def piece_length(self) -> int:
        """Return selected piece length."""
        return self._native.piece_length

    @property
    def piece_length_policy(self) -> str | None:
        """Return automatic policy identifier for automatic selection."""
        return self._native.piece_length_policy

    @property
    def metrics(self) -> CreateMetrics:
        """Return cached phase timing metrics."""
        cached = self._metrics_cache
        if cached is None:
            cached = CreateMetrics(
                scan_ms=self._native.scan_ms,
                hash_ms=self._native.hash_ms,
                serialize_ms=self._native.serialize_ms,
            )
            object.__setattr__(self, "_metrics_cache", cached)
        return cached

    def __repr__(self) -> str:
        """Return a compact representation without copying output bytes."""
        return (
            f"CreateResult(mode={self.mode.value!r}, file_count={self.file_count}, "
            f"payload_bytes={self.payload_bytes})"
        )


class CancellationToken:
    """Cooperative cancellation handle for Python creation calls."""

    def __init__(self) -> None:
        """Create an active token."""
        self._native = _native._CancellationToken()  # noqa: SLF001

    def cancel(self) -> None:
        """Request cancellation."""
        self._native.cancel()

    @property
    def cancelled(self) -> bool:
        """Return whether cancellation was requested."""
        return self._native.cancelled


def create_bytes(
    path: str | PathLike[str],
    *,
    options: CreateOptions | None = None,
    progress: Callable[[int, int, int], None] | None = None,
    cancellation: CancellationToken | None = None,
) -> CreateResult:
    """Create canonical metainfo bytes without writing a destination."""
    return _create_impl(
        path,
        destination=None,
        overwrite=False,
        durable=False,
        options=options,
        progress=progress,
        cancellation=cancellation,
    )


def create(  # noqa: PLR0913
    path: str | PathLike[str],
    destination: str | PathLike[str],
    *,
    options: CreateOptions | None = None,
    overwrite: bool = False,
    durable: bool = False,
    progress: Callable[[int, int, int], None] | None = None,
    cancellation: CancellationToken | None = None,
) -> CreateResult:
    """Create canonical metainfo and atomically write it."""
    return _create_impl(
        path,
        destination=destination,
        overwrite=overwrite,
        durable=durable,
        options=options,
        progress=progress,
        cancellation=cancellation,
    )


def _create_impl(  # noqa: PLR0913
    path: str | PathLike[str],
    *,
    destination: str | PathLike[str] | None,
    overwrite: bool,
    durable: bool,
    options: CreateOptions | None,
    progress: Callable[[int, int, int], None] | None,
    cancellation: CancellationToken | None,
) -> CreateResult:
    options = options or CreateOptions()
    if options.omit_created_by and options.created_by is not None:
        message = "omit_created_by cannot be true when created_by is set"
        raise TypeError(message)
    try:
        value = _native.create_v1(
            Path(path),
            options.mode.value,
            Path(destination) if destination is not None else None,
            overwrite,
            durable,
            options.piece_length,
            options.threads,
            _tracker_bytes(options.trackers),
            _string_sequence_bytes(options.web_seeds, "web_seeds"),
            _node_bytes(options.nodes),
            options.private,
            _text_bytes(options.source, "source")
            if options.source is not None
            else None,
            _text_bytes(options.comment, "comment")
            if options.comment is not None
            else None,
            _text_bytes(options.created_by, "created_by")
            if options.created_by is not None
            else None,
            options.omit_created_by,
            options.creation_date,
            progress,
            cancellation._native if cancellation is not None else None,  # noqa: SLF001
        )
    except _native._NativeError as error:  # noqa: SLF001
        raise _convert_error(error) from None
    return CreateResult(value)


__all__ = [
    "CancellationToken",
    "CreateMetrics",
    "CreateOptions",
    "CreateResult",
    "create",
    "create_bytes",
]
