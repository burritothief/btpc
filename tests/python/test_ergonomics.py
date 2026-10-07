from __future__ import annotations

import btpc
import pytest


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
