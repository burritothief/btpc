from __future__ import annotations

import pickle
from concurrent.futures import ThreadPoolExecutor
from typing import TYPE_CHECKING

import btpc

# Spec: PYAPI-TYPES-001
import pytest
from btpc import _native
from btpc._conversion import _convert_error

if TYPE_CHECKING:
    from pathlib import Path

PIECE_LENGTH = 16
SHA1_LENGTH = 20


def torrent_bytes() -> bytes:
    return b"d4:infod6:lengthi0e4:name5:empty12:piece lengthi16e6:pieces0:ee"


def canonical_cached(native: btpc._native._NativeMetainfo) -> bool:
    return native.canonical_cached


def cache_is_set(torrent: btpc.Metainfo, name: str) -> bool:
    return getattr(torrent, name) is not None


@pytest.mark.parametrize(
    "data",
    [torrent_bytes(), bytearray(torrent_bytes()), memoryview(torrent_bytes())],
)
def test_from_bytes_accepts_contiguous_buffers(data: object) -> None:
    torrent = btpc.Metainfo.from_bytes(data)
    assert torrent.mode is btpc.TorrentMode.V1
    assert torrent.name == b"empty"
    assert torrent.name_text == "empty"
    assert torrent.piece_length == PIECE_LENGTH
    assert torrent.total_length == 0
    assert torrent.original_bytes == torrent_bytes()
    assert torrent.to_bytes() == torrent_bytes()
    assert torrent.info_hash_v1 is not None
    assert len(torrent.info_hash_v1.bytes) == SHA1_LENGTH
    assert torrent.info_hash_v2 is None
    assert torrent.validate().is_valid


def test_from_bytes_rejects_noncontiguous_buffers() -> None:
    with pytest.raises(TypeError, match="contiguous byte buffer"):
        btpc.Metainfo.from_bytes(memoryview(torrent_bytes())[::2])


def test_read_pathlike_files_and_private_native_surface(tmp_path: Path) -> None:
    path = tmp_path / "empty.torrent"
    path.write_bytes(torrent_bytes())
    torrent = btpc.Metainfo.read(path)
    assert torrent.files[0].path == (b"empty",)
    assert torrent.files[0].path_text == ("empty",)
    assert not hasattr(btpc, "_NativeMetainfo")


def test_native_metainfo_is_lazy_cached_immutable_and_thread_safe() -> None:
    native = btpc._native.inspect_bytes(torrent_bytes())  # noqa: SLF001
    assert type(native).__name__ == "_NativeMetainfo"
    assert not canonical_cached(native)
    assert native.files is native.files
    assert native.files[0].path is native.files[0].path
    assert native.files[0].attributes is native.files[0].attributes
    assert native.validation is native.validation
    assert not canonical_cached(native)
    assert native.canonical_bytes == torrent_bytes()
    assert native.canonical_bytes is native.canonical_bytes
    assert native.original_bytes is native.original_bytes
    assert canonical_cached(native)
    assert "empty" in repr(native)
    with pytest.raises(TypeError):
        pickle.dumps(native)
    with pytest.raises(TypeError):
        type("Child", (type(native),), {})

    with ThreadPoolExecutor(max_workers=4) as executor:
        names = list(executor.map(lambda _index: native.name, range(32)))
    assert names == [b"empty"] * 32


def test_public_metainfo_caches_expensive_properties_and_defines_object_policy() -> (
    None
):
    first = btpc.Metainfo.from_bytes(torrent_bytes())
    second = btpc.Metainfo.from_bytes(torrent_bytes())
    assert not cache_is_set(first, "_original_bytes_cache")
    assert not cache_is_set(first, "_canonical_bytes_cache")
    assert not cache_is_set(first, "_files_cache")
    assert first == second
    assert not cache_is_set(first, "_original_bytes_cache")
    assert first.files is first.files
    assert cache_is_set(first, "_files_cache")
    assert first.trackers is first.trackers
    assert first.validate() is first.validate()
    assert not cache_is_set(first, "_canonical_bytes_cache")
    assert first.to_bytes() == torrent_bytes()
    assert cache_is_set(first, "_canonical_bytes_cache")
    assert "empty" in repr(first)
    with pytest.raises(TypeError):
        pickle.dumps(first)
    with pytest.raises(TypeError):
        type("Child", (btpc.Metainfo,), {})
    with pytest.raises(TypeError, match="unhashable"):
        hash(first)


def test_metainfo_exact_native_equality_preserves_source_identity() -> None:
    canonical = torrent_bytes()
    noncanonical = canonical.replace(b"i16e", b"i016e")
    first = btpc.Metainfo.from_bytes(canonical)
    second = btpc.Metainfo.from_bytes(canonical)
    different_encoding = btpc.Metainfo.from_bytes(noncanonical)
    assert first == second
    assert first != different_encoding
    assert first != object()
    assert not cache_is_set(first, "_original_bytes_cache")
    assert not cache_is_set(second, "_original_bytes_cache")


def test_owned_operations_do_not_materialize_original_python_bytes(
    tmp_path: Path,
) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"owned native operations")
    metainfo = btpc.Metainfo.from_bytes(btpc.create_bytes(payload).bytes)
    assert not cache_is_set(metainfo, "_original_bytes_cache")

    assert metainfo.magnet().startswith("magnet:?xt=")
    assert metainfo.edit(comment="owned").name == metainfo.name
    assert metainfo.verify(payload).is_valid

    assert not cache_is_set(metainfo, "_original_bytes_cache")


def test_parse_options_apply_to_bytes_and_paths(tmp_path: Path) -> None:
    data = torrent_bytes()
    exact = btpc.ParseOptions(max_total_input=len(data), max_owned_allocation=1_000_000)
    assert btpc.Metainfo.from_bytes(data, options=exact).name == b"empty"
    path = tmp_path / "empty.torrent"
    path.write_bytes(data)
    assert btpc.Metainfo.read(path, options=exact).name == b"empty"

    too_small = btpc.ParseOptions(max_total_input=len(data) - 1)
    with pytest.raises(btpc.ResourceLimitError, match="total input") as raised:
        btpc.Metainfo.from_bytes(data, options=too_small)
    assert raised.value.limit == "total input"
    assert raised.value.actual == len(data)
    assert raised.value.maximum == len(data) - 1
    with pytest.raises(btpc.ResourceLimitError, match="total input"):
        btpc.Metainfo.read(path, options=too_small)

    integer_limited = btpc.ParseOptions(max_integer_digits=1)
    with pytest.raises(btpc.ResourceLimitError, match="integer digits"):
        btpc.Metainfo.from_bytes(data, options=integer_limited)


def test_structured_exception_hierarchy() -> None:
    with pytest.raises(btpc.BencodeError) as raised:
        btpc.Metainfo.from_bytes(b"not bencode")
    assert raised.value.offset == 0
    assert raised.value.field is None
    assert raised.value.path is None

    with pytest.raises(btpc.MetainfoError) as raised_meta:
        btpc.Metainfo.from_bytes(b"d4:infodee")
    assert raised_meta.value.field == "info.name"


def test_unknown_native_exception_mapping_fails_loudly() -> None:
    with pytest.raises(RuntimeError, match="unrecognized native BTPC exception"):
        _convert_error(_native._NativeError("unknown"))  # noqa: SLF001


def test_validation_report_distinguishes_canonicality() -> None:
    payload = (
        b"d4:infod6:lengthi1e4:name1:x12:piece lengthi016384e"
        b"6:pieces20:00000000000000000000ee"
    )
    metainfo = btpc.Metainfo.from_bytes(payload)
    report = metainfo.validate()
    assert report.is_valid
    assert not report.canonical
    assert report.canonical_offset is not None
    assert report.canonical_message is not None
    assert btpc.Metainfo.from_bytes(metainfo.to_bytes()).validate().canonical


def test_torrent_bytes_and_paths_use_raw_identity() -> None:
    composed = btpc.TorrentBytes("é".encode())
    decomposed = btpc.TorrentBytes("e\N{COMBINING ACUTE ACCENT}".encode())
    invalid = btpc.TorrentBytes(b"\xff")
    assert composed.text == "é"
    assert invalid.text is None
    assert composed != decomposed
    assert sorted([composed, decomposed]) == [decomposed, composed]
    assert "raw=b'\\xff'" in repr(invalid)
    assert pickle.loads(pickle.dumps(invalid)) == invalid  # noqa: S301

    path = btpc.TorrentPath((btpc.TorrentBytes(b"dir"), invalid))
    assert path.text is None
    assert path.to_path() is None
    assert pickle.loads(pickle.dumps(path)) == path  # noqa: S301
    assert btpc.TorrentPath((btpc.TorrentBytes(b"dir"),)).to_path() is not None

    for unsafe in [b"", b".", b"..", b"a/b", b"a\\b", b"a\0b"]:
        with pytest.raises(ValueError, match="unsafe torrent path component"):
            btpc.TorrentPath((btpc.TorrentBytes(unsafe),))
    with pytest.raises(ValueError, match="at least one component"):
        btpc.TorrentPath(())
    with pytest.raises(TypeError, match="raw must be bytes"):
        btpc.TorrentBytes("not bytes")  # type: ignore[arg-type]

    torrent = btpc.Metainfo.from_bytes(torrent_bytes())
    assert torrent.files[0].torrent_path.components == (btpc.TorrentBytes(b"empty"),)


def _bencode(value: object) -> bytes:
    if isinstance(value, bytes):
        return str(len(value)).encode() + b":" + value
    if isinstance(value, int):
        return b"i" + str(value).encode() + b"e"
    if isinstance(value, list):
        return b"l" + b"".join(_bencode(item) for item in value) + b"e"
    if isinstance(value, dict):
        return (
            b"d"
            + b"".join(
                _bencode(key) + _bencode(item) for key, item in sorted(value.items())
            )
            + b"e"
        )
    raise TypeError(type(value))


def test_optional_metadata_properties_are_lossless_and_immutable() -> None:
    data = _bencode(
        {
            b"announce": b"tracker",
            b"announce-list": [[b"tracker", b"tracker"]],
            b"comment": b"\xfecomment",
            b"created by": b"\xfdcreator",
            b"creation date": 0,
            b"nodes": [[b"\xfchost", 1], [b"host", 65_535]],
            b"url-list": [b"seed", b"seed"],
            b"info": {
                b"length": 0,
                b"name": b"x",
                b"piece length": 16_384,
                b"pieces": b"",
                b"source": b"\xfbsource",
            },
        }
    )
    torrent = btpc.Metainfo.from_bytes(data)
    assert torrent.trackers == ((b"tracker", b"tracker"),)
    assert torrent.web_seeds == (b"seed", b"seed")
    assert torrent.nodes == ((b"\xfchost", 1), (b"host", 65_535))
    assert torrent.nodes is torrent.nodes
    assert torrent.source == b"\xfbsource"
    assert torrent.comment == b"\xfecomment"
    assert torrent.created_by == b"\xfdcreator"
    assert torrent.creation_date == 0


def test_optional_metadata_warning_and_rejection_policy(tmp_path: Path) -> None:
    fallback = _bencode(
        {
            b"announce": b"primary",
            b"announce-list": [],
            b"info": {
                b"length": 0,
                b"name": b"x",
                b"piece length": 16_384,
                b"pieces": b"",
            },
        }
    )
    torrent = btpc.Metainfo.from_bytes(fallback)
    assert torrent.trackers == ((b"primary",),)
    assert torrent.validate().warnings == (
        "empty announce-list ignored in favor of announce",
    )

    malformed = _bencode(
        {
            b"comment": 1,
            b"info": {
                b"length": 0,
                b"name": b"x",
                b"piece length": 16_384,
                b"pieces": b"",
            },
        }
    )
    with pytest.raises(btpc.MetainfoError, match="comment"):
        btpc.Metainfo.from_bytes(malformed)

    payload = tmp_path / "payload"
    payload.write_bytes(b"data")
    for options in [
        btpc.CreateOptions(nodes=(("host", 0),)),
        btpc.CreateOptions(creation_date=-1),
        btpc.CreateOptions(trackers=((),)),
    ]:
        with pytest.raises(btpc.MetainfoError):
            btpc.create_bytes(payload, options=options)
