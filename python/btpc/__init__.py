"""Create, parse, inspect, edit, and verify BitTorrent metainfo."""

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
from .metainfo import (
    BencodeDictionary,
    BencodeList,
    BencodeValue,
    Metainfo,
    TorrentFile,
    UnknownField,
    ValidationReport,
)
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
    "BencodeDictionary",
    "BencodeError",
    "BencodeList",
    "BencodeValue",
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
    "UnknownField",
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
