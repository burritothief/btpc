from __future__ import annotations

from typing import TYPE_CHECKING

import btpc
import pytest

if TYPE_CHECKING:
    from pathlib import Path


def test_python_editor_preserves_or_changes_hashes_by_field_scope(
    tmp_path: Path,
) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"data")
    original = btpc.Metainfo.from_bytes(btpc.create_bytes(payload).bytes)
    top_level = original.edit(
        trackers=(("https://one", "https://two"),),
        web_seeds=("https://seed",),
        nodes=(("router.example", 6881),),
        comment="comment",
        created_by="python-editor",
        creation_date=123,
    )
    assert top_level.info_hash_v1 == original.info_hash_v1
    assert top_level.trackers == ((b"https://one", b"https://two"),)
    assert top_level.web_seeds == (b"https://seed",)

    info_edit = top_level.edit(private=True, source="source")
    assert info_edit.info_hash_v1 != original.info_hash_v1
    assert info_edit.private is True
    with pytest.raises(TypeError, match="comment"):
        original.edit(comment=b"bytes")  # type: ignore[arg-type]


def test_python_editor_three_state_optional_fields(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"data")
    original = btpc.Metainfo.from_bytes(btpc.create_bytes(payload).bytes)
    set_values = original.edit(
        trackers=(("https://tracker",),),
        web_seeds=("https://seed",),
        nodes=(("router.example", 1),),
        private=False,
        source="",
        comment="",
        created_by="",
        creation_date=0,
    )
    preserved = set_values.edit(
        trackers=btpc.UNCHANGED,
        web_seeds=btpc.UNCHANGED,
        nodes=btpc.UNCHANGED,
        private=btpc.UNCHANGED,
        source=btpc.UNCHANGED,
        comment=btpc.UNCHANGED,
        created_by=btpc.UNCHANGED,
        creation_date=btpc.UNCHANGED,
    )
    assert preserved.to_bytes() == set_values.to_bytes()
    removed = set_values.edit(
        trackers=None,
        web_seeds=None,
        nodes=None,
        private=None,
        source=None,
        comment=None,
        created_by=None,
        creation_date=None,
    )
    assert removed.trackers == ()
    assert removed.web_seeds == ()
    assert removed.private is None


def test_python_editor_raw_fields_and_attributes(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"data")
    original = btpc.Metainfo.from_bytes(btpc.create_bytes(payload).bytes)
    edited = original.edit(
        raw_top_level={b"x-custom": 7},
        file_attributes={(b"payload",): b"x"},
    )
    assert b"x-custom" in edited.unknown_fields
    assert edited.files[0].attributes == b"x"
    with pytest.raises(btpc.MetainfoError, match="reserved"):
        original.edit(raw_top_level={b"announce": b"bad"})


def test_python_top_level_edit_preserves_noncanonical_info_bytes() -> None:
    data = b"d4:infod6:pieces0:12:piece lengthi16384e4:name7:payload6:lengthi0eee"
    original_info = data[len(b"d4:info") : -1]
    original = btpc.Metainfo.from_bytes(data)

    edited = original.edit(comment="updated", raw_top_level={b"x-custom": 7})

    assert edited.info_hash_v1 == original.info_hash_v1
    assert original_info in edited.original_bytes
    assert edited.to_bytes() != edited.original_bytes
    assert b"x-custom" in edited.unknown_fields


def test_python_hybrid_attributes_update_both_representations(
    tmp_path: Path,
) -> None:
    hybrid_representation_count = 2
    payload = tmp_path / "payload"
    payload.write_bytes(b"data")
    original = btpc.Metainfo.from_bytes(
        btpc.create_bytes(
            payload,
            options=btpc.CreateOptions(mode=btpc.TorrentMode.HYBRID),
        ).bytes
    )

    edited = original.edit(file_attributes={(b"payload",): b"x"})

    assert edited.original_bytes.count(b"4:attr1:x") == hybrid_representation_count
