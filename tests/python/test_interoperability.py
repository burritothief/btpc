from __future__ import annotations

from pathlib import Path

import btpc
import pytest

FIXTURE_ROOT = Path(__file__).parents[1] / "fixtures" / "interoperability"
MINIMUM_FIXTURE_COUNT = 10


def fixture_rows() -> list[list[str]]:
    return [
        line.split("\t")
        for line in (FIXTURE_ROOT / "manifest.tsv").read_text().splitlines()
        if line and not line.startswith("#")
    ]


def test_python_reads_the_documented_fixture_corpus() -> None:
    rows = fixture_rows()
    assert len(rows) >= MINIMUM_FIXTURE_COUNT
    for row in rows:
        path = FIXTURE_ROOT / row[0]
        if row[4] == "reject":
            with pytest.raises(btpc.BtpcError):
                btpc.Metainfo.read(path)
            continue

        torrent = btpc.Metainfo.read(path)
        assert torrent.mode.value == row[3]
        assert torrent.original_bytes == path.read_bytes()
        assert torrent.validate().is_valid
        btpc.Metainfo.from_bytes(torrent.to_bytes())
