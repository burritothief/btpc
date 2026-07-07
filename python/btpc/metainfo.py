"""Parse, inspect, validate, edit, serialize, and verify torrent metainfo."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Never, Protocol, SupportsIndex, TypeAlias, cast

from . import _native
from ._conversion import (
    _convert_error,
    _node_bytes,
    _string_sequence_bytes,
    _text_bytes,
    _tracker_bytes,
)
from .errors import ResourceLimitError
from .types import (
    UNCHANGED,
    HashValue,
    ParseOptions,
    TorrentBytes,
    TorrentMode,
    TorrentPath,
    UnchangedType,
)
from .verification import PayloadVerificationReport, verify

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence
    from os import PathLike

    from ._native import _NativeMetainfo as _NativeMetainfoType
    from .creation import CancellationToken


class _Buffer(Protocol):
    def __buffer__(self, flags: int, /) -> memoryview: ...


@dataclass(frozen=True, slots=True)
class BencodeList:
    """Store an immutable ordered bencode list.

    Attributes:
        values: Recursive values in source order.
    """

    values: tuple[BencodeValue, ...]

    def __post_init__(self) -> None:
        """Validate and freeze recursive values."""
        object.__setattr__(
            self,
            "values",
            tuple(_normalize_bencode_value(value) for value in self.values),
        )


@dataclass(frozen=True, slots=True)
class BencodeDictionary:
    """Store an immutable bencode dictionary in canonical raw-key order.

    Attributes:
        items: Unique raw-byte keys and recursive values.
    """

    items: tuple[tuple[bytes, BencodeValue], ...]

    def __post_init__(self) -> None:
        """Validate keys, reject duplicates, and sort canonically."""
        normalized: list[tuple[bytes, BencodeValue]] = []
        seen: set[bytes] = set()
        for key, value in self.items:
            if not isinstance(key, bytes):
                message = "bencode dictionary keys must be bytes"
                raise TypeError(message)
            if key in seen:
                message = f"duplicate bencode dictionary key: {key!r}"
                raise ValueError(message)
            seen.add(key)
            normalized.append((key, _normalize_bencode_value(value)))
        object.__setattr__(self, "items", tuple(sorted(normalized)))


@dataclass(frozen=True, slots=True)
class UnknownField:
    """Describe one unknown top-level field and its exact source encoding.

    Attributes:
        key: Raw top-level dictionary key.
        value: Recursive semantic bencode value.
        encoded: Exact source bytes covering the encoded key and value.
        span: Half-open offsets for ``encoded`` within ``original_bytes``.
    """

    key: bytes
    value: BencodeValue
    encoded: bytes
    span: tuple[int, int]


BencodeValue: TypeAlias = int | bytes | BencodeList | BencodeDictionary


def _normalize_bencode_value(value: BencodeValue) -> BencodeValue:
    if isinstance(value, bool):
        message = "bencode values do not accept bool; use an integer"
        raise TypeError(message)
    if isinstance(value, (int, bytes)):
        return value
    if isinstance(value, BencodeList):
        return value
    if isinstance(value, BencodeDictionary):
        return value
    message = "bencode values must be int, bytes, BencodeList, or BencodeDictionary"
    raise TypeError(message)


def _public_bencode_value(value: object) -> BencodeValue:
    if isinstance(value, bool):
        message = "native bencode integer unexpectedly returned bool"
        raise TypeError(message)
    if isinstance(value, (int, bytes)):
        return value
    if isinstance(value, list):
        return BencodeList(tuple(_public_bencode_value(item) for item in value))
    if isinstance(value, dict):
        return BencodeDictionary(
            tuple((key, _public_bencode_value(item)) for key, item in value.items())
        )
    message = f"unexpected native bencode value: {type(value).__name__}"
    raise TypeError(message)


def _native_bencode_value(value: BencodeValue) -> object:
    value = _normalize_bencode_value(value)
    if isinstance(value, BencodeList):
        return [_native_bencode_value(item) for item in value.values]
    if isinstance(value, BencodeDictionary):
        return {key: _native_bencode_value(item) for key, item in value.items}
    return value


@dataclass(frozen=True, slots=True)
class TorrentFile:
    """Describe one file in the torrent's logical payload.

    Attributes:
        length: File length in bytes.
        path: Raw torrent path components. Components remain bytes because torrent
            paths are not required to be UTF-8.
        attributes: Raw BEP 47/BEP 52 file attribute bytes.
        pieces_root: v2 Merkle root when present.
        is_padding: Whether the entry is a padding file.
    """

    length: int
    path: tuple[bytes, ...]
    attributes: bytes
    pieces_root: HashValue | None
    is_padding: bool = False

    @property
    def path_text(self) -> tuple[str, ...] | None:
        """Return UTF-8 components when every component decodes."""
        try:
            return tuple(component.decode() for component in self.path)
        except UnicodeDecodeError:
            return None

    @property
    def torrent_path(self) -> TorrentPath:
        """Return the raw-identity typed torrent path."""
        return TorrentPath(tuple(TorrentBytes(component) for component in self.path))


@dataclass(frozen=True, slots=True)
class ValidationReport:
    """Summarize validation and canonical-encoding status.

    Attributes:
        warnings: Non-fatal protocol or interoperability warnings.
        canonical: Whether the original source used canonical bencoding.
        canonical_offset: Source offset of the first canonicalization issue.
        canonical_message: Human-readable canonicalization issue.
    """

    warnings: tuple[str, ...]
    canonical: bool
    canonical_offset: int | None = None
    canonical_message: str | None = None

    @property
    def is_valid(self) -> bool:
        """Return true because invalid metainfo fails construction."""
        return True


class Metainfo:
    """Represent validated metainfo while preserving its exact source identity.

    Parsing retains the original metainfo bytes and computes info hashes from the
    exact raw ``info`` dictionary slice, not from re-serialization. Equality also
    compares exact original bytes. Use :meth:`to_bytes` for canonical output and
    :attr:`original_bytes` when byte-for-byte source identity is required. Raw
    protocol strings and paths remain bytes; optional ``*_text`` views decode only
    valid UTF-8.

    Examples:
        >>> from btpc import Metainfo, TorrentMode
        >>> data = (
        ...     b"d4:infod6:lengthi1e4:name1:x12:piece lengthi16384e"
        ...     b"6:pieces20:00000000000000000000ee"
        ... )
        >>> torrent = Metainfo.from_bytes(data)
        >>> torrent.mode is TorrentMode.V1
        True
        >>> torrent.original_bytes == data
        True
    """

    __slots__ = (
        "_canonical_bytes_cache",
        "_files_cache",
        "_info_hash_v1_cache",
        "_info_hash_v2_cache",
        "_native",
        "_nodes_cache",
        "_original_bytes_cache",
        "_trackers_cache",
        "_unknown_fields_cache",
        "_validation_cache",
        "_web_seeds_cache",
    )
    _native: _NativeMetainfoType
    _nodes_cache: tuple[tuple[bytes, int], ...] | None
    _canonical_bytes_cache: bytes | None
    _files_cache: tuple[TorrentFile, ...] | None
    _info_hash_v1_cache: HashValue | bool | None
    _info_hash_v2_cache: HashValue | bool | None
    _original_bytes_cache: bytes | None
    _trackers_cache: tuple[tuple[bytes, ...], ...] | None
    _unknown_fields_cache: tuple[UnknownField, ...] | None
    _validation_cache: ValidationReport | None
    _web_seeds_cache: tuple[bytes, ...] | None

    def __init__(self, native: _NativeMetainfoType) -> None:
        """Own a validated native object; callers use parsing constructors."""
        object.__setattr__(self, "_native", native)
        for name in self.__slots__:
            if name != "_native":
                object.__setattr__(self, name, None)

    def __setattr__(self, _name: str, _value: object) -> Never:
        """Keep the public inspection object immutable."""
        message = "Metainfo is immutable"
        raise AttributeError(message)

    def __init_subclass__(cls, **_kwargs: object) -> Never:
        """Keep storage and invariants under BTPC control."""
        message = "Metainfo does not support subclassing"
        raise TypeError(message)

    def __reduce_ex__(self, _protocol: SupportsIndex) -> Never:
        """Reject pickling until a stable serialized-object policy exists."""
        message = "Metainfo objects are not picklable; serialize with to_bytes()"
        raise TypeError(message)

    @classmethod
    def from_bytes(
        cls, data: object, *, options: ParseOptions | None = None
    ) -> Metainfo:
        """Parse metainfo from a contiguous Python buffer.

        Args:
            data: ``bytes``, ``bytearray``, or another readable contiguous buffer.
                Non-contiguous memoryviews are rejected.
            options: Parser input, allocation, and integer-digit limits.

        Returns:
            A validated immutable metainfo object retaining exact source bytes.

        Raises:
            TypeError: If ``data`` does not provide a contiguous buffer.
            BencodeError: If bencode syntax is invalid.
            MetainfoError: If decoded fields violate torrent protocol rules.
            ResourceLimitError: If a configured parse limit is exceeded.

        Examples:
            >>> from btpc import Metainfo
            >>> data = (
            ...     b"d4:infod6:lengthi1e4:name1:x12:piece lengthi16384e"
            ...     b"6:pieces20:00000000000000000000ee"
            ... )
            >>> Metainfo.from_bytes(memoryview(data)).name
            b'x'
        """
        try:
            view = memoryview(cast("_Buffer", data))
        except TypeError as error:
            message = "data must support the contiguous buffer protocol"
            raise TypeError(message) from error
        options = options or ParseOptions()
        if (
            options.max_total_input is not None
            and view.nbytes > options.max_total_input
        ):
            ResourceLimitError.raise_exceeded(
                "total input", view.nbytes, options.max_total_input
            )
        try:
            return cls(
                _native.inspect_bytes(
                    view,
                    options.max_total_input,
                    options.max_owned_allocation,
                    options.max_integer_digits,
                )
            )
        except _native._NativeError as error:  # noqa: SLF001
            raise _convert_error(error) from None

    @classmethod
    def read(
        cls, path: str | PathLike[str], *, options: ParseOptions | None = None
    ) -> Metainfo:
        """Read and parse metainfo directly from a filesystem path.

        Args:
            path: Torrent file path.
            options: Parser input, allocation, and integer-digit limits.

        Returns:
            A validated immutable metainfo object.

        Raises:
            PathError: If the file cannot be opened or read.
            BencodeError: If bencode syntax is invalid.
            MetainfoError: If decoded fields violate torrent protocol rules.
            ResourceLimitError: If a configured parse limit is exceeded.

        Examples:
            >>> from pathlib import Path
            >>> from tempfile import TemporaryDirectory
            >>> from btpc import Metainfo
            >>> data = (
            ...     b"d4:infod6:lengthi1e4:name1:x12:piece lengthi16384e"
            ...     b"6:pieces20:00000000000000000000ee"
            ... )
            >>> with TemporaryDirectory() as directory:
            ...     path = Path(directory) / "sample.torrent"
            ...     _ = path.write_bytes(data)
            ...     assert Metainfo.read(path).original_bytes == data
        """
        try:
            options = options or ParseOptions()
            return cls(
                _native.inspect_path(
                    Path(path),
                    options.max_total_input,
                    options.max_owned_allocation,
                    options.max_integer_digits,
                )
            )
        except _native._NativeError as error:  # noqa: SLF001
            raise _convert_error(error) from None

    def verify(
        self,
        payload: str | PathLike[str],
        *,
        fail_fast: bool = False,
        extra_files: bool = False,
        progress: Callable[[int, int, int], None] | None = None,
        cancellation: CancellationToken | None = None,
    ) -> PayloadVerificationReport:
        r"""Verify payload files using every hash domain present in this torrent.

        ``payload`` is resolved as the torrent's payload root: a single-file torrent
        may point directly to its file, while a multi-file torrent points to the
        containing directory. v1 verifies the concatenated piece stream, v2 verifies
        per-file Merkle roots, and hybrid verifies both. Mismatches are returned in
        deterministic order; filesystem and policy failures raise exceptions.

        Args:
            payload: Payload file or root directory.
            fail_fast: Stop after the first mismatch when true.
            extra_files: Report files not represented by the torrent.
            progress: Optional callback receiving ``(completed_bytes, total_bytes,
                completed_pieces)``. Callback exceptions propagate unchanged.
            cancellation: Cooperative cancellation token.

        Returns:
            A report containing zero or more deterministic mismatches.

        Raises:
            PathError: If required payload paths cannot be read safely.
            VerificationError: If verification cannot be performed as requested.
            CancelledError: If ``cancellation`` is requested.
            Exception: Any exception raised by ``progress``.

        Examples:
            >>> from pathlib import Path
            >>> from tempfile import TemporaryDirectory
            >>> from btpc import CreateOptions, Metainfo, create_bytes
            >>> with TemporaryDirectory() as directory:
            ...     payload = Path(directory) / "hello.txt"
            ...     _ = payload.write_bytes(b"hello torrent\\n")
            ...     created = create_bytes(
            ...         payload,
            ...         options=CreateOptions(creation_date=0, threads=1),
            ...     )
            ...     assert Metainfo.from_bytes(created.bytes).verify(payload).is_valid
        """
        return verify(
            self,
            payload,
            fail_fast=fail_fast,
            extra_files=extra_files,
            progress=progress,
            cancellation=cancellation,
        )

    def magnet(
        self,
        *,
        display_name: bool = True,
        trackers: bool = True,
        web_seeds: bool = True,
    ) -> str:
        """Generate a deterministic magnet URI.

        Args:
            display_name: Include the decoded display name when valid UTF-8.
            trackers: Include tracker URLs.
            web_seeds: Include web seed URLs.

        Returns:
            A magnet URI containing every applicable v1 and v2 exact-topic hash.

        Examples:
            >>> from btpc import Metainfo
            >>> data = (
            ...     b"d4:infod6:lengthi1e4:name1:x12:piece lengthi16384e"
            ...     b"6:pieces20:00000000000000000000ee"
            ... )
            >>> magnet = Metainfo.from_bytes(data).magnet(trackers=False)
            >>> magnet.startswith("magnet:?xt=")
            True
        """
        return self._native.magnet(
            display_name,
            trackers,
            web_seeds,
        )

    def edit(  # noqa: PLR0913
        self,
        *,
        trackers: Sequence[Sequence[str]] | None | UnchangedType = UNCHANGED,
        web_seeds: Sequence[str] | None | UnchangedType = UNCHANGED,
        nodes: Sequence[tuple[str, int]] | None | UnchangedType = UNCHANGED,
        private: bool | None | UnchangedType = UNCHANGED,
        source: str | None | UnchangedType = UNCHANGED,
        comment: str | None | UnchangedType = UNCHANGED,
        created_by: str | None | UnchangedType = UNCHANGED,
        creation_date: int | None | UnchangedType = UNCHANGED,
        raw_top_level: Mapping[bytes, BencodeValue] | None = None,
        file_attributes: dict[tuple[bytes, ...], bytes] | None = None,
    ) -> Metainfo:
        """Return a validated copy with explicit preserve, remove, or set edits.

        Optional fields use three states: :data:`btpc.UNCHANGED` preserves the
        current value, ``None`` removes it, and a typed value replaces it. Trackers,
        web seeds, nodes, comments, creator, and creation date are top-level fields
        and do not affect info hashes. ``private``, ``source``, and file attributes
        edit the ``info`` dictionary and therefore change applicable info hashes.
        Top-level-only edits retain the exact original ``info`` bytes, including a
        noncanonical source encoding. Use :meth:`to_bytes` for an explicit fully
        canonical serialization.

        Args:
            trackers: Tracker tiers, ``None`` to remove, or ``UNCHANGED``.
            web_seeds: Web seeds, ``None`` to remove, or ``UNCHANGED``.
            nodes: DHT nodes, ``None`` to remove, or ``UNCHANGED``.
            private: Private flag, ``None`` to remove, or ``UNCHANGED``.
            source: Source string, ``None`` to remove, or ``UNCHANGED``.
            comment: Comment, ``None`` to remove, or ``UNCHANGED``.
            created_by: Creator text, ``None`` to remove, or ``UNCHANGED``.
            creation_date: Unix timestamp, ``None`` to remove, or ``UNCHANGED``.
            raw_top_level: Raw recursive bencode extension replacements.
            file_attributes: Raw attributes keyed by raw torrent path components.

        Returns:
            A newly validated immutable object retaining untouched ``info`` bytes.

        Raises:
            TypeError: If a textual field receives a non-string value.
            MetainfoError: If an edit would produce invalid metainfo.

        Examples:
            >>> from btpc import UNCHANGED, Metainfo
            >>> data = (
            ...     b"d4:infod6:lengthi1e4:name1:x12:piece lengthi16384e"
            ...     b"6:pieces20:00000000000000000000ee"
            ... )
            >>> torrent = Metainfo.from_bytes(data)
            >>> with_comment = torrent.edit(comment="reviewed")
            >>> preserved = with_comment.edit(comment=UNCHANGED)
            >>> removed = with_comment.edit(comment=None)
            >>> preserved.info_hash_v1 == removed.info_hash_v1 == torrent.info_hash_v1
            True
            >>> torrent.edit(source="release").info_hash_v1 != torrent.info_hash_v1
            True
        """
        try:
            value = self._native.edit(
                None if trackers is UNCHANGED else _tracker_bytes(trackers or ()),
                None
                if web_seeds is UNCHANGED
                else _string_sequence_bytes(web_seeds or (), "web_seeds"),
                None if nodes is UNCHANGED else _node_bytes(nodes or ()),
                None if private is UNCHANGED else private,
                private is not UNCHANGED,
                None
                if source is None or source is UNCHANGED
                else _text_bytes(source, "source"),
                source is not UNCHANGED,
                None
                if comment is None or comment is UNCHANGED
                else _text_bytes(comment, "comment"),
                comment is not UNCHANGED,
                _text_bytes(created_by, "created_by")
                if created_by is not None and created_by is not UNCHANGED
                else None,
                created_by is not UNCHANGED,
                None if creation_date is UNCHANGED else creation_date,
                creation_date is not UNCHANGED,
                [
                    (key, _native_bencode_value(item))
                    for key, item in (raw_top_level or {}).items()
                ],
                [
                    (list(path), attributes)
                    for path, attributes in (file_attributes or {}).items()
                ],
            )
        except _native._NativeError as error:  # noqa: SLF001
            raise _convert_error(error) from None
        return type(self)(value)

    @property
    def mode(self) -> TorrentMode:
        """Return the validated protocol representation."""
        return TorrentMode(self._native.mode)

    @property
    def name(self) -> bytes:
        """Return raw torrent name bytes."""
        return self._native.name

    @property
    def piece_length(self) -> int:
        """Return piece length in bytes."""
        return self._native.piece_length

    @property
    def total_length(self) -> int:
        """Return total payload bytes."""
        return self._native.total_length

    @property
    def piece_count(self) -> int:
        """Return logical piece count."""
        return self._native.piece_count

    @property
    def files(self) -> tuple[TorrentFile, ...]:
        """Return cached immutable payload file objects."""
        cached = self._files_cache
        if cached is None:
            cached = tuple(
                TorrentFile(
                    length=file.length,
                    path=file.path,
                    attributes=file.attributes,
                    pieces_root=(
                        HashValue(file.pieces_root)
                        if file.pieces_root is not None
                        else None
                    ),
                    is_padding=file.is_padding,
                )
                for file in self._native.files
            )
            object.__setattr__(self, "_files_cache", cached)
        return cached

    @property
    def trackers(self) -> tuple[tuple[bytes, ...], ...]:
        """Return cached immutable tracker tiers."""
        cached = self._trackers_cache
        if cached is None:
            cached = self._native.trackers
            object.__setattr__(self, "_trackers_cache", cached)
        return cached

    @property
    def web_seeds(self) -> tuple[bytes, ...]:
        """Return cached immutable web seeds."""
        cached = self._web_seeds_cache
        if cached is None:
            cached = self._native.web_seeds
            object.__setattr__(self, "_web_seeds_cache", cached)
        return cached

    @property
    def private(self) -> bool | None:
        """Return the explicit private flag."""
        return self._native.private

    @property
    def nodes(self) -> tuple[tuple[bytes, int], ...]:
        """Return cached immutable DHT bootstrap nodes as raw hosts and ports."""
        cached = self._nodes_cache
        if cached is None:
            cached = self._native.nodes
            object.__setattr__(self, "_nodes_cache", cached)
        return cached

    @property
    def source(self) -> bytes | None:
        """Return raw source bytes from the info dictionary."""
        return self._native.source

    @property
    def comment(self) -> bytes | None:
        """Return raw top-level comment bytes."""
        return self._native.comment

    @property
    def created_by(self) -> bytes | None:
        """Return raw top-level creator bytes."""
        return self._native.created_by

    @property
    def creation_date(self) -> int | None:
        """Return the non-negative Unix creation timestamp."""
        return self._native.creation_date

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
    def original_bytes(self) -> bytes:
        """Return exact source bytes, copied only on first request."""
        cached = self._original_bytes_cache
        if cached is None:
            cached = self._native.original_bytes
            object.__setattr__(self, "_original_bytes_cache", cached)
        return cached

    @property
    def unknown_fields(self) -> tuple[UnknownField, ...]:
        """Return cached unknown fields with values and exact source encodings."""
        cached = self._unknown_fields_cache
        if cached is None:
            cached = tuple(
                UnknownField(key, _public_bencode_value(value), encoded, span)
                for key, value, encoded, span in self._native.unknown_fields
            )
            object.__setattr__(self, "_unknown_fields_cache", cached)
        return cached

    @property
    def name_text(self) -> str | None:
        """Return the name as UTF-8 when valid."""
        try:
            return self.name.decode()
        except UnicodeDecodeError:
            return None

    def to_bytes(self, *, canonical: bool = True) -> bytes:
        """Serialize canonical metainfo or return the exact original bytes.

        Args:
            canonical: Sort dictionary keys by unsigned raw-byte order and normalize
                bencode when true. False returns :attr:`original_bytes` unchanged.

        Returns:
            Serialized metainfo bytes.

        Examples:
            >>> from btpc import Metainfo
            >>> data = (
            ...     b"d4:infod6:lengthi1e4:name1:x12:piece lengthi16384e"
            ...     b"6:pieces20:00000000000000000000ee"
            ... )
            >>> torrent = Metainfo.from_bytes(data)
            >>> torrent.to_bytes(canonical=False) == data
            True
            >>> reparsed = Metainfo.from_bytes(torrent.to_bytes())
            >>> reparsed.info_hash_v1 == torrent.info_hash_v1
            True
        """
        if not canonical:
            return self.original_bytes
        cached = self._canonical_bytes_cache
        if cached is None:
            cached = self._native.canonical_bytes
            object.__setattr__(self, "_canonical_bytes_cache", cached)
        return cached

    def validate(self) -> ValidationReport:
        """Return the construction-time validation report."""
        cached = self._validation_cache
        if cached is None:
            native = self._native.validation
            cached = ValidationReport(
                tuple(native.warnings),
                native.canonical,
                native.canonical_offset,
                native.canonical_message,
            )
            object.__setattr__(self, "_validation_cache", cached)
        return cached

    def __eq__(self, other: object) -> bool:
        """Compare validated objects by exact source bytes."""
        return isinstance(other, Metainfo) and self._native == other._native

    __hash__ = None  # type: ignore[assignment]

    def __repr__(self) -> str:
        """Return a compact representation without expanding files or bytes."""
        return (
            f"Metainfo(mode={self.mode.value!r}, name={self.name!r}, "
            f"file_count={self._native.file_count})"
        )


__all__ = [
    "BencodeDictionary",
    "BencodeList",
    "BencodeValue",
    "Metainfo",
    "TorrentFile",
    "UnknownField",
    "ValidationReport",
]
