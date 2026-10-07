from __future__ import annotations

from typing import TYPE_CHECKING

import btpc
import pytest

if TYPE_CHECKING:
    from pathlib import Path


@pytest.mark.parametrize("mode", list(btpc.TorrentMode))
def test_write_preserves_identity_and_explicit_canonicalization(
    tmp_path: Path, mode: btpc.TorrentMode
) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"example")
    created = btpc.create_bytes(
        payload,
        options=btpc.CreateOptions(mode=mode, piece_length=16_384),
    )
    source = created.bytes.replace(
        b"12:piece lengthi16384e", b"12:piece lengthi016384e", 1
    )
    original = btpc.Metainfo.from_bytes(source)
    edited = original.edit(comment="reviewed")
    destination = tmp_path / "nested" / "edited.torrent"

    edited.write(destination)
    saved = btpc.Metainfo.read(destination)
    assert saved.original_bytes == edited.original_bytes
    assert saved.info_hash_v1 == original.info_hash_v1
    assert saved.info_hash_v2 == original.info_hash_v2
    assert not saved.validation.canonical

    with pytest.raises(btpc.PathError):
        original.write(destination)
    assert destination.read_bytes() == edited.original_bytes
    edited.write(destination, canonical=True, overwrite=True, durable=True)
    canonical = btpc.Metainfo.read(destination)
    assert canonical.validation.canonical
    assert destination.read_bytes() == edited.to_bytes()
    assert (canonical.info_hash_v1, canonical.info_hash_v2) != (
        original.info_hash_v1,
        original.info_hash_v2,
    )
    assert not list(destination.parent.glob(".*.btpc-tmp-*"))


def test_write_failure_preserves_destination_and_cleans_temporary_files(
    tmp_path: Path,
) -> None:
    torrent = btpc.Metainfo.from_bytes(
        b"d4:infod6:lengthi0e4:name1:x12:piece lengthi16384e6:pieces0:ee"
    )
    destination = tmp_path / "directory"
    destination.mkdir()
    with pytest.raises(btpc.PathError):
        torrent.write(destination, overwrite=True)
    assert destination.is_dir()
    assert not list(tmp_path.glob(".*.btpc-tmp-*"))


@pytest.mark.parametrize("mode", list(btpc.TorrentMode))
def test_created_metainfo_is_cached_and_independent_of_payload(
    tmp_path: Path, mode: btpc.TorrentMode
) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"owned creation result")
    result = btpc.create_bytes(payload, options=btpc.CreateOptions(mode=mode))
    payload.unlink()

    assert result._bytes_cache is None  # noqa: SLF001
    torrent = result.metainfo
    assert torrent is result.metainfo
    assert result._bytes_cache is None  # noqa: SLF001
    assert torrent.mode is mode
    assert torrent.info_hash_v1 == result.info_hash_v1
    assert torrent.info_hash_v2 == result.info_hash_v2
    assert torrent.magnet().startswith("magnet:?xt=")
    assert torrent.original_bytes == result.bytes


def test_creation_options_own_immutable_sequences() -> None:
    trackers = [["https://tracker.example/announce"]]
    seeds = ["https://seed.example/file"]
    nodes = [("router.example", 6881)]
    options = btpc.CreateOptions(trackers=trackers, web_seeds=seeds, nodes=nodes)
    trackers[0].append("https://backup.example/announce")
    trackers.clear()
    seeds.clear()
    nodes.clear()

    assert options.trackers == (("https://tracker.example/announce",),)
    assert options.web_seeds == ("https://seed.example/file",)
    assert options.nodes == (("router.example", 6881),)
    assert options == btpc.CreateOptions(
        trackers=(("https://tracker.example/announce",),),
        web_seeds=("https://seed.example/file",),
        nodes=(("router.example", 6881),),
    )
    assert hash(options) == hash(options)


def test_decoded_metadata_views_can_be_used_for_text_edits(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"text")
    result = btpc.create_bytes(
        payload,
        options=btpc.CreateOptions(
            comment="reviewed π",
            source="source",
            created_by="creator",
            trackers=(("https://tracker.example/π",),),
            web_seeds=("https://seed.example/π",),
            nodes=(("router.example", 6881),),
        ),
    )
    torrent = result.metainfo
    assert torrent.comment_text == "reviewed π"
    assert torrent.source_text == "source"
    assert torrent.created_by_text == "creator"
    assert torrent.trackers_text == (("https://tracker.example/π",),)
    assert torrent.web_seeds_text == ("https://seed.example/π",)
    assert torrent.nodes_text == (("router.example", 6881),)
    copied = torrent.edit(
        comment=torrent.comment_text,
        source=torrent.source_text,
        created_by=torrent.created_by_text,
        trackers=torrent.trackers_text,
        web_seeds=torrent.web_seeds_text,
        nodes=torrent.nodes_text,
    )
    assert copied.original_bytes == torrent.original_bytes


def test_decoded_metadata_views_preserve_invalid_raw_bytes() -> None:
    data = (
        b"d8:announce1:\xff7:comment1:\xff10:created by1:\xff"
        b"4:infod6:lengthi0e4:name1:x12:piece lengthi16384e"
        b"6:pieces0:6:source1:\xffe5:nodesll1:\xffi1eee8:url-listl1:\xffee"
    )
    torrent = btpc.Metainfo.from_bytes(data)
    assert torrent.comment_text is None
    assert torrent.source_text is None
    assert torrent.created_by_text is None
    assert torrent.trackers_text is None
    assert torrent.web_seeds_text is None
    assert torrent.nodes_text is None
    assert torrent.comment == b"\xff"
    assert torrent.trackers == ((b"\xff",),)
    assert torrent.original_bytes == data

    empty = btpc.Metainfo.from_bytes(
        b"d4:infod6:lengthi0e4:name1:x12:piece lengthi16384e6:pieces0:ee"
    )
    assert empty.comment_text is None
    assert empty.source_text is None
    assert empty.created_by_text is None
    assert empty.trackers_text == ()
    assert empty.web_seeds_text == ()
    assert empty.nodes_text == ()


def test_validation_property_is_cached_and_validate_remains_compatible() -> None:
    torrent = btpc.Metainfo.from_bytes(
        b"d4:infod6:lengthi0e4:name1:x12:piece lengthi16384e6:pieces0:ee"
    )
    assert torrent.validation is torrent.validation
    assert torrent.validate() is torrent.validation
    assert torrent.validation.is_valid


def test_bencode_values_support_collection_operations() -> None:
    values = btpc.BencodeList((1, b"raw", 1))
    assert len(values) == len(values.values)
    assert list(values) == [1, b"raw", 1]
    assert values[-1] == 1
    assert values[1:] == btpc.BencodeList((b"raw", 1))
    assert values[::-1] == btpc.BencodeList((1, b"raw", 1))
    assert b"raw" in values
    assert values.count(1) == 2  # noqa: PLR2004
    assert values.index(b"raw") == 1
    with pytest.raises(IndexError):
        _ = values[len(values)]

    mapping = btpc.BencodeDictionary(((b"z", values), (b"a", 7)))
    assert len(mapping) == len(mapping.items)
    assert list(mapping) == [b"a", b"z"]
    assert mapping[b"z"] is values
    assert mapping.keys() == (b"a", b"z")
    assert mapping.values() == (7, values)
    assert mapping.get(b"missing") is None
    assert mapping.get(b"missing", b"fallback") == b"fallback"
    assert b"a" in mapping
    assert b"missing" not in mapping
    assert dict(mapping) == {b"a": 7, b"z": values}
    assert mapping.items == ((b"a", 7), (b"z", values))
    with pytest.raises(KeyError, match="missing"):
        _ = mapping[b"missing"]
