from pathlib import Path
from typing import assert_type

import btpc
from btpc.creation import CreateOptions
from btpc.metainfo import Metainfo
from btpc.types import TorrentMode
from btpc.verification import PayloadVerificationReport

# Spec: TEST-PY-TYPING-001

payload = Path("payload")
options = btpc.CreateOptions(
    mode=btpc.TorrentMode.HYBRID,
    trackers=(("https://tracker",),),
    web_seeds=("https://seed",),
    nodes=(("router.example", 6881),),
    source="source",
    comment="comment",
    created_by="consumer",
)
assert_type(CreateOptions, type[btpc.CreateOptions])
assert_type(Metainfo, type[btpc.Metainfo])
assert_type(TorrentMode, type[btpc.TorrentMode])
assert_type(PayloadVerificationReport, type[btpc.PayloadVerificationReport])
result = btpc.create_bytes(payload, options=options)
assert_type(result, btpc.CreateResult)
assert_type(result.bytes, bytes)
assert_type(result.info_hash_v1, btpc.HashValue | None)
assert_type(result.metrics, btpc.CreateMetrics)
metainfo = btpc.Metainfo.from_bytes(result.bytes)
assert_type(metainfo.mode, btpc.TorrentMode)
assert_type(metainfo.name, bytes)
assert_type(metainfo.files, tuple[btpc.TorrentFile, ...])
assert_type(metainfo.trackers, tuple[tuple[bytes, ...], ...])
assert_type(metainfo.web_seeds, tuple[bytes, ...])
assert_type(metainfo.nodes, tuple[tuple[bytes, int], ...])
assert_type(metainfo.source, bytes | None)
assert_type(metainfo.comment, bytes | None)
assert_type(metainfo.created_by, bytes | None)
assert_type(metainfo.creation_date, int | None)
assert_type(metainfo.unknown_fields, tuple[btpc.UnknownField, ...])
unknown_value: btpc.BencodeValue = btpc.BencodeDictionary(
    ((b"nested", btpc.BencodeList((1, b"value"))),)
)
assert_type(metainfo.edit(raw_top_level={b"extension": unknown_value}), btpc.Metainfo)
assert_type(metainfo.magnet(), str)
assert_type(metainfo.edit(comment="edited"), btpc.Metainfo)
assert_type(metainfo.edit(comment=btpc.UNCHANGED), btpc.Metainfo)
token = btpc.CancellationToken()
assert_type(token.cancelled, bool)
assert_type(
    metainfo.verify(payload, cancellation=token), btpc.PayloadVerificationReport
)
