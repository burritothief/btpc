from __future__ import annotations

import sys
from pathlib import Path

from btpc import CreateOptions, Metainfo, TorrentMode, create


def main() -> None:
    root = Path(sys.argv[1]).resolve()
    payload = root / "payload"
    payload.mkdir(parents=True)
    (payload / "hello.txt").write_bytes(b"hello torrent\n")
    for mode in TorrentMode:
        destination = root / f"payload-{mode.value}.torrent"
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
        torrent = Metainfo.read(destination)
        if torrent.mode is not mode:
            msg = f"mode mismatch: expected {mode.value}, got {torrent.mode.value}"
            raise RuntimeError(msg)
        if not torrent.magnet().startswith("magnet:?xt="):
            msg = f"invalid {mode.value} magnet"
            raise RuntimeError(msg)
        if not torrent.verify(payload).is_valid:
            msg = f"{mode.value} payload verification failed"
            raise RuntimeError(msg)
        if Metainfo.from_bytes(result.bytes).to_bytes() != result.bytes:
            msg = f"{mode.value} canonical bytes changed after parse"
            raise RuntimeError(msg)


if __name__ == "__main__":
    main()
