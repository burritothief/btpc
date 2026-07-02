"""Typed Python interface for BTPC."""

from typing import Final

from . import creation, errors, metainfo, types, verification
from ._native import __version__ as _native_version
from .creation import (
    CancellationToken,
    CreateMetrics,
    CreateOptions,
    CreateResult,
    create,
    create_bytes,
)
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
from .metainfo import Metainfo, TorrentFile, ValidationReport
from .types import (
    UNCHANGED,
    HashValue,
    ParseOptions,
    TorrentBytes,
    TorrentMode,
    TorrentPath,
)
from .verification import (
    MismatchKind,
    PayloadMismatch,
    PayloadVerificationReport,
    verify,
)

__version__: Final[str] = _native_version

__all__ = [
    "UNCHANGED",
    "BencodeError",
    "BtpcError",
    "CancellationToken",
    "CancelledError",
    "CreateMetrics",
    "CreateOptions",
    "CreateResult",
    "HashValue",
    "Metainfo",
    "MetainfoError",
    "MismatchKind",
    "ParseOptions",
    "PathError",
    "PayloadMismatch",
    "PayloadVerificationReport",
    "ResourceLimitError",
    "TorrentBytes",
    "TorrentFile",
    "TorrentMode",
    "TorrentPath",
    "UnsupportedError",
    "ValidationReport",
    "VerificationError",
    "__version__",
    "create",
    "create_bytes",
    "creation",
    "errors",
    "metainfo",
    "types",
    "verification",
    "verify",
]
