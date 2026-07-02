"""Metainfo parsing, inspection, validation, and editing."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Never, Protocol, SupportsIndex, cast

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
    from collections.abc import Callable, Sequence
    from os import PathLike

    from ._native import _NativeMetainfo as _NativeMetainfoType
    from .creation import CancellationToken


class _Buffer(Protocol):
    def __buffer__(self, flags: int, /) -> memoryview: ...


@dataclass(frozen=True, slots=True)
class TorrentFile:
    """Immutable payload file metadata."""

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
    """Successful metainfo validation report."""

    warnings: tuple[str, ...]
    canonical: bool
    canonical_offset: int | None = None
    canonical_message: str | None = None

    @property
    def is_valid(self) -> bool:
        """Return true because invalid metainfo fails construction."""
        return True


class Metainfo:
    """Immutable lazy facade over an owned native metainfo object."""

    __slots__ = (
        "_canonical_bytes_cache",
        "_files_cache",
        "_info_hash_v1_cache",
        "_info_hash_v2_cache",
        "_native",
        "_original_bytes_cache",
        "_trackers_cache",
        "_unknown_fields_cache",
        "_validation_cache",
        "_web_seeds_cache",
    )
    _native: _NativeMetainfoType
    _canonical_bytes_cache: bytes | None
    _files_cache: tuple[TorrentFile, ...] | None
    _info_hash_v1_cache: HashValue | bool | None
    _info_hash_v2_cache: HashValue | bool | None
    _original_bytes_cache: bytes | None
    _trackers_cache: tuple[tuple[bytes, ...], ...] | None
    _unknown_fields_cache: tuple[bytes, ...] | None
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
        """Parse bytes, bytearray, or any contiguous buffer."""
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
        """Read and parse a metainfo path."""
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
        """Verify payload files using all hash domains applicable to this torrent."""
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
        """Generate a deterministic magnet URI."""
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
        raw_top_level: dict[bytes, int | bytes] | None = None,
        file_attributes: dict[tuple[bytes, ...], bytes] | None = None,
    ) -> Metainfo:
        """Return a validated canonical copy with selected metadata edits."""
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
                list((raw_top_level or {}).items()),
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
    def unknown_fields(self) -> tuple[bytes, ...]:
        """Return cached unknown top-level field keys."""
        cached = self._unknown_fields_cache
        if cached is None:
            cached = self._native.unknown_fields
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
        """Return canonical bytes by default or exact original bytes."""
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


__all__ = ["Metainfo", "TorrentFile", "ValidationReport"]
