"""Thin executable adapter for timing the BTPC Python API."""

from __future__ import annotations

import sys
from pathlib import Path

import btpc

EXPECTED_ARGUMENT_COUNT = 5


def main() -> int:
    """Create one deterministic v1 torrent using Python bindings."""
    if len(sys.argv) != EXPECTED_ARGUMENT_COUNT:
        return 2
    payload, output, tracker, piece_length = sys.argv[1:]
    options = btpc.CreateOptions(
        mode=btpc.TorrentMode.V1,
        piece_length=int(piece_length),
        trackers=((tracker.encode(),),),
        private=True,
    )
    btpc.create(Path(payload), Path(output), options=options, overwrite=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
