"""Create v1, v2, and hybrid torrent metainfo from filesystem payloads."""

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
    """Configure deterministic torrent creation.

    Attributes:
        mode: Protocol representation to create. The default is
            :attr:`TorrentMode.V1`.
        piece_length: Explicit piece length in bytes. ``None`` selects the core's
            deterministic automatic policy; v2 and hybrid values must satisfy BEP
            52 constraints.
        threads: Maximum hashing worker count. ``None`` lets the core choose from
            available parallelism.
        trackers: Tracker tiers as a sequence of URL-string sequences. Text is
            encoded as strict UTF-8 at the native boundary.
        web_seeds: Web seed URLs encoded as strict UTF-8.
        nodes: DHT bootstrap nodes as ``(host, port)`` pairs.
        private: Optional private flag stored in the info dictionary. Changing it
            changes the info hash.
        source: Optional source string stored in the info dictionary. Changing it
            changes the info hash.
        comment: Optional top-level comment. Changing it does not change the info
            hash.
        created_by: Explicit top-level creator text. When omitted, BTPC writes
            ``btpc/<version>`` unless ``omit_created_by`` is true.
        omit_created_by: Suppress the default creator field. It cannot be combined
            with an explicit ``created_by`` value.
        creation_date: Unix timestamp stored at the top level. Set a fixed value,
            commonly ``0``, for reproducible output.

    Examples:
        >>> from btpc import CreateOptions, TorrentMode
        >>> options = CreateOptions(
        ...     mode=TorrentMode.HYBRID,
        ...     piece_length=16_384,
        ...     creation_date=0,
        ...     threads=1,
        ... )
        >>> options.mode is TorrentMode.HYBRID
        True
    """

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
    """Record elapsed time for creation phases.

    Attributes:
        scan_ms: Filesystem traversal time in milliseconds.
        hash_ms: Payload hashing time in milliseconds.
        serialize_ms: Canonical metainfo serialization time in milliseconds.
    """

    scan_ms: float
    hash_ms: float
    serialize_ms: float


class CreateResult:
    """Expose generated metainfo, hashes, and metrics without eager copies."""

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
    """Request cooperative cancellation of creation or verification.

    The same token may be passed to one or more operations. Calling :meth:`cancel`
    is thread-safe; active operations stop at a core cancellation checkpoint and
    raise :class:`btpc.CancelledError`.

    Examples:
        >>> from btpc import CancellationToken
        >>> token = CancellationToken()
        >>> token.cancel()
        >>> token.cancelled
        True
    """

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
    r"""Create canonical metainfo without writing a torrent file.

    Args:
        path: Payload file or directory to traverse deterministically.
        options: Creation settings. ``None`` uses :class:`CreateOptions` defaults.
        progress: Optional callback receiving ``(completed_bytes, total_bytes,
            completed_pieces)``. Exceptions raised by the callback propagate to the
            caller and stop creation.
        cancellation: Cooperative cancellation token checked during traversal and
            hashing.

    Returns:
        The immutable result, including generated bytes, applicable info hashes,
        selected piece length, and phase metrics.

    Raises:
        PathError: If the payload cannot be traversed or read.
        MetainfoError: If options or payload structure violate protocol rules.
        CancelledError: If ``cancellation`` is requested.
        Exception: Any exception raised by ``progress``.

    Examples:
        >>> from pathlib import Path
        >>> from tempfile import TemporaryDirectory
        >>> from btpc import CreateOptions, create_bytes
        >>> with TemporaryDirectory() as directory:
        ...     payload = Path(directory) / "hello.txt"
        ...     _ = payload.write_bytes(b"hello torrent\\n")
        ...     result = create_bytes(
        ...         payload,
        ...         options=CreateOptions(creation_date=0, threads=1),
        ...     )
        ...     assert result.file_count == 1
    """
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
    r"""Create canonical metainfo and atomically write a torrent file.

    The destination is written through a temporary file and renamed into place.
    Existing files are rejected unless ``overwrite`` is true. With ``durable``
    enabled, BTPC also requests file and parent-directory synchronization before
    returning.

    Args:
        path: Payload file or directory to traverse deterministically.
        destination: Torrent file to create atomically.
        options: Creation settings. ``None`` uses :class:`CreateOptions` defaults.
        overwrite: Replace an existing destination when true.
        durable: Request durable file and directory synchronization.
        progress: Optional callback receiving ``(completed_bytes, total_bytes,
            completed_pieces)``. Callback exceptions propagate unchanged.
        cancellation: Cooperative cancellation token.

    Returns:
        The same result data returned by :func:`create_bytes`.

    Raises:
        PathError: If payload or destination I/O fails, including a disallowed
            overwrite.
        MetainfoError: If options or payload structure violate protocol rules.
        CancelledError: If ``cancellation`` is requested.
        Exception: Any exception raised by ``progress``.

    Examples:
        >>> from pathlib import Path
        >>> from tempfile import TemporaryDirectory
        >>> from btpc import CreateOptions, create
        >>> with TemporaryDirectory() as directory:
        ...     root = Path(directory)
        ...     payload = root / "hello.txt"
        ...     destination = root / "hello.torrent"
        ...     _ = payload.write_bytes(b"hello torrent\\n")
        ...     result = create(
        ...         payload,
        ...         destination,
        ...         options=CreateOptions(creation_date=0, threads=1),
        ...     )
        ...     assert destination.read_bytes() == result.bytes
    """
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
