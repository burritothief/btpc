from __future__ import annotations

from typing import TYPE_CHECKING

from btpc import CreateOptions, Metainfo, TorrentMode, create

if TYPE_CHECKING:
    from pathlib import Path


# Spec: PYAPI-DOC-001
def test_python_guide_example_runs_for_every_mode(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.mkdir()
    (payload / "hello.txt").write_bytes(b"hello torrent\n")

    for mode in TorrentMode:
        destination = tmp_path / f"payload-{mode.value}.torrent"
        result = create(
            payload,
            destination,
            options=CreateOptions(
                mode=mode,
                piece_length=16_384 if mode is not TorrentMode.V1 else None,
                creation_date=0,
                threads=1,
            ),
        )
        torrent = Metainfo.from_bytes(result.bytes)
        assert torrent.verify(payload).is_valid
        assert torrent.magnet().startswith("magnet:?xt=")
        assert Metainfo.read(destination).mode is mode
