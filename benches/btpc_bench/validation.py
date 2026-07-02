"""BTPC-backed and independent benchmark torrent validation."""

from __future__ import annotations

import hashlib
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path
from typing import TYPE_CHECKING, TypeAlias

import btpc

if TYPE_CHECKING:
    from .models import DatasetFingerprint

BValue: TypeAlias = int | bytes | list["BValue"] | dict[bytes, "BValue"]


@dataclass(frozen=True, slots=True)
class ExpectedTorrent:
    """Exact v1 semantics and piece checksums expected from every tool."""

    name: bytes
    length: int
    tracker: bytes
    piece_length: int
    piece_sha1: tuple[bytes, ...]

    @classmethod
    def from_fingerprint(
        cls, fingerprint: DatasetFingerprint, *, tracker: str
    ) -> ExpectedTorrent:
        """Build expectations from the streaming preflight oracle."""
        return cls(
            name=Path(fingerprint.path).name.encode(),
            length=fingerprint.size_bytes,
            tracker=tracker.encode(),
            piece_length=fingerprint.piece_length,
            piece_sha1=tuple(bytes.fromhex(item) for item in fingerprint.piece_sha1),
        )

    @classmethod
    def from_payload(
        cls,
        *,
        payload_path: Path,
        payload: bytes,
        tracker: str,
        piece_length: int,
    ) -> ExpectedTorrent:
        """Build a compact test oracle from in-memory payload bytes."""
        return cls(
            name=payload_path.name.encode(),
            length=len(payload),
            tracker=tracker.encode(),
            piece_length=piece_length,
            piece_sha1=tuple(
                hashlib.sha1(payload[offset : offset + piece_length]).digest()
                for offset in range(0, len(payload), piece_length)
            ),
        )


@dataclass(frozen=True, slots=True)
class ValidationResult:
    """Independent and BTPC validation outcome."""

    valid: bool
    reason: str
    info_hash_v1: str = ""
    failure: ValidationFailure | None = None
    creation_date: int | None = None
    created_by: bytes | None = None


class ValidationFailure(StrEnum):
    """Stable failure category for benchmark result diagnostics."""

    PARSE = "parse"
    PROFILE = "profile"
    PAYLOAD = "payload"
    INFO_HASH = "info_hash"
    BTPC_CROSS_CHECK = "btpc_cross_check"


def validate_torrent(
    path: Path,
    expected: ExpectedTorrent,
    *,
    require_btpc: bool = True,
    expected_info_hash: str | None = None,
) -> ValidationResult:
    """Validate exact semantics, raw info hash, and every v1 piece digest."""
    try:
        raw = path.read_bytes()
        value, end, info_span = _decode_root(raw)
        if end != len(raw) or not isinstance(value, dict):
            return ValidationResult(
                valid=False,
                reason="trailing bytes or non-dictionary root",
                failure=ValidationFailure.PARSE,
            )
        info = value.get(b"info")
        if not isinstance(info, dict):
            return ValidationResult(
                valid=False,
                reason="missing info dictionary",
                failure=ValidationFailure.PARSE,
            )
        checks = (
            (value.get(b"announce") == expected.tracker, "tracker mismatch"),
            (info.get(b"name") == expected.name, "name mismatch"),
            (info.get(b"length") == expected.length, "length mismatch"),
            (
                info.get(b"piece length") == expected.piece_length,
                "piece length mismatch",
            ),
            (info.get(b"private") == 1, "private flag mismatch"),
            (b"files" not in info, "not single-file v1"),
            (b"meta version" not in info, "not v1 mode"),
            (
                set(info)
                == {b"length", b"name", b"piece length", b"pieces", b"private"},
                "unexpected info fields",
            ),
        )
        for passed, reason in checks:
            if not passed:
                return ValidationResult(
                    valid=False,
                    reason=reason,
                    failure=ValidationFailure.PROFILE,
                )
        pieces = info.get(b"pieces")
        expected_pieces = b"".join(expected.piece_sha1)
        if pieces != expected_pieces:
            return ValidationResult(
                valid=False,
                reason="piece digest mismatch",
                failure=ValidationFailure.PAYLOAD,
            )
        if info_span is None:
            return ValidationResult(
                valid=False,
                reason="raw info span unavailable",
                failure=ValidationFailure.PARSE,
            )
        info_hash = hashlib.sha1(raw[slice(*info_span)]).hexdigest()
        if expected_info_hash is not None and info_hash != expected_info_hash:
            return ValidationResult(
                valid=False,
                reason="info hash mismatch",
                info_hash_v1=info_hash,
                failure=ValidationFailure.INFO_HASH,
            )
        if require_btpc:
            btpc_error = _validate_with_btpc(path, expected, info_hash)
            if btpc_error:
                return ValidationResult(
                    valid=False,
                    reason=btpc_error,
                    info_hash_v1=info_hash,
                    failure=ValidationFailure.BTPC_CROSS_CHECK,
                )
        creation_date = value.get(b"creation date")
        created_by = value.get(b"created by")
        return ValidationResult(
            valid=True,
            reason="",
            info_hash_v1=info_hash,
            creation_date=creation_date if isinstance(creation_date, int) else None,
            created_by=created_by if isinstance(created_by, bytes) else None,
        )
    except (OSError, ValueError, TypeError) as error:
        return ValidationResult(
            valid=False,
            reason=f"parse failure: {error}",
            failure=ValidationFailure.PARSE,
        )


def _validate_with_btpc(path: Path, expected: ExpectedTorrent, info_hash: str) -> str:
    try:
        torrent = btpc.Metainfo.read(path)
    except (ImportError, OSError, ValueError) as error:
        return f"BTPC validation failed: {error}"
    if torrent.mode.value != "v1":
        return "BTPC mode mismatch"
    if torrent.name != expected.name or torrent.total_length != expected.length:
        return "BTPC payload metadata mismatch"
    if torrent.piece_length != expected.piece_length or torrent.private is not True:
        return "BTPC piece/private mismatch"
    if torrent.info_hash_v1 is None or torrent.info_hash_v1.hex != info_hash:
        return "BTPC raw info hash mismatch"
    return ""


def _decode_root(raw: bytes) -> tuple[BValue, int, tuple[int, int] | None]:
    if not raw.startswith(b"d"):
        value, end = _decode(raw, 0)
        return value, end, None
    index = 1
    output: dict[bytes, BValue] = {}
    info_span: tuple[int, int] | None = None
    while raw[index : index + 1] != b"e":
        key_value, index = _decode(raw, index)
        if not isinstance(key_value, bytes):
            msg = "dictionary key is not bytes"
            raise TypeError(msg)
        start = index
        item, index = _decode(raw, index)
        output[key_value] = item
        if key_value == b"info":
            info_span = (start, index)
    return output, index + 1, info_span


def _decode(raw: bytes, index: int) -> tuple[BValue, int]:
    marker = raw[index : index + 1]
    if marker == b"i":
        end = raw.index(b"e", index + 1)
        return int(raw[index + 1 : end]), end + 1
    if marker == b"l":
        items: list[BValue] = []
        index += 1
        while raw[index : index + 1] != b"e":
            item, index = _decode(raw, index)
            items.append(item)
        return items, index + 1
    if marker == b"d":
        items_dict: dict[bytes, BValue] = {}
        index += 1
        while raw[index : index + 1] != b"e":
            key, index = _decode(raw, index)
            if not isinstance(key, bytes):
                msg = "dictionary key is not bytes"
                raise TypeError(msg)
            item, index = _decode(raw, index)
            items_dict[key] = item
        return items_dict, index + 1
    colon = raw.index(b":", index)
    length = int(raw[index:colon])
    start = colon + 1
    end = start + length
    if end > len(raw):
        msg = "truncated byte string"
        raise ValueError(msg)
    return raw[start:end], end
