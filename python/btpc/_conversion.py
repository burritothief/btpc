"""Private Python/native boundary conversions."""

# ruff: noqa: SLF001

from __future__ import annotations

from collections import abc
from pathlib import Path

from . import _native
from .errors import (
    BencodeError,
    BtpcError,
    CancelledError,
    MetainfoError,
    PathError,
    ResourceLimitError,
    UnsupportedError,
    VerificationError,
)


def _text_bytes(value: object, parameter: str) -> bytes:
    if not isinstance(value, str):
        message = f"{parameter} must be str, not {type(value).__name__}"
        raise TypeError(message)
    return value.encode("utf-8")


def _tracker_bytes(trackers: object, parameter: str = "trackers") -> list[list[bytes]]:
    if not isinstance(trackers, abc.Sequence) or isinstance(trackers, (str, bytes)):
        message = f"{parameter} must be a sequence of string sequences"
        raise TypeError(message)
    result: list[list[bytes]] = []
    for tier_index, tier in enumerate(trackers):
        if not isinstance(tier, abc.Sequence) or isinstance(tier, (str, bytes)):
            message = f"{parameter}[{tier_index}] must be a sequence of str"
            raise TypeError(message)
        result.append(
            [_text_bytes(value, f"{parameter}[{tier_index}]") for value in tier]
        )
    return result


def _string_sequence_bytes(values: object, parameter: str) -> list[bytes]:
    if not isinstance(values, abc.Sequence) or isinstance(values, (str, bytes)):
        message = f"{parameter} must be a sequence of str"
        raise TypeError(message)
    return [_text_bytes(value, parameter) for value in values]


def _node_bytes(nodes: object, parameter: str = "nodes") -> list[tuple[bytes, int]]:
    if not isinstance(nodes, abc.Sequence) or isinstance(nodes, (str, bytes)):
        message = f"{parameter} must be a sequence of (str, int) tuples"
        raise TypeError(message)
    result: list[tuple[bytes, int]] = []
    node_arity = 2
    for index, node in enumerate(nodes):
        if not isinstance(node, tuple) or len(node) != node_arity:
            message = f"{parameter}[{index}] must be a (str, int) tuple"
            raise TypeError(message)
        host, port = node
        maximum_port = 65_535
        if (
            not isinstance(port, int)
            or isinstance(port, bool)
            or not 0 <= port <= maximum_port
        ):
            message = f"{parameter}[{index}] port must be an integer from 0 to 65535"
            raise TypeError(message)
        result.append((_text_bytes(host, f"{parameter}[{index}] host"), port))
    return result


def _convert_error(error: _native._NativeError) -> BtpcError:
    mappings: tuple[tuple[type[_native._NativeError], type[BtpcError]], ...] = (
        (_native._NativeBencodeSyntaxError, BencodeError),
        (_native._NativeBencodeCanonicalError, BencodeError),
        (_native._NativeResourceLimitError, ResourceLimitError),
        (_native._NativeMetainfoError, MetainfoError),
        (_native._NativeIoError, PathError),
        (_native._NativeVerificationError, VerificationError),
        (_native._NativeUnsupportedError, UnsupportedError),
        (_native._NativeCancelledError, CancelledError),
    )
    error_type = next(
        (public for native, public in mappings if isinstance(error, native)), None
    )
    if error_type is None:
        message = f"unrecognized native BTPC exception: {type(error).__name__}"
        raise RuntimeError(message) from error
    native_path = getattr(error, "path", None)
    return error_type(
        str(error),
        offset=getattr(error, "offset", None),
        field=getattr(error, "field", None),
        path=Path(native_path) if native_path is not None else None,
        limit=getattr(error, "limit", None),
        actual=getattr(error, "actual", None),
        maximum=getattr(error, "maximum", None),
    )
