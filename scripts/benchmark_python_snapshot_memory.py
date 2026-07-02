#!/usr/bin/env python3
"""Measure lazy versus fully materialized Python metainfo peak RSS."""

from __future__ import annotations

import argparse
import json
import resource

import btpc


def bencode_bytes(value: bytes) -> bytes:
    return str(len(value)).encode() + b":" + value


def fixture(file_count: int, name_width: int, blob_bytes: int) -> bytes:
    files = bytearray(b"l")
    for index in range(file_count):
        name = f"file-{index:0{name_width}d}".encode()
        files.extend(b"d6:lengthi0e4:pathl")
        files.extend(bencode_bytes(name))
        files.extend(b"ee")
    files.extend(b"e")
    info = (
        b"d5:files" + bytes(files) + b"4:name7:payload12:piece lengthi16384e6:pieces0:e"
    )
    blob = b"4:blob" + bencode_bytes(b"x" * blob_bytes) if blob_bytes else b""
    return b"d" + blob + b"4:info" + info + b"e"


def peak_rss_bytes() -> int:
    maximum = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return maximum if __import__("sys").platform == "darwin" else maximum * 1024


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=("lazy", "materialized"), required=True)
    parser.add_argument("--files", type=int, default=20_000)
    parser.add_argument("--name-width", type=int, default=24)
    parser.add_argument("--blob-bytes", type=int, default=0)
    arguments = parser.parse_args()
    data = fixture(arguments.files, arguments.name_width, arguments.blob_bytes)
    torrent = btpc.Metainfo.from_bytes(data)
    if arguments.mode == "materialized":
        _ = torrent.original_bytes
        _ = torrent.to_bytes()
        _ = torrent.files
        _ = torrent.trackers
        _ = torrent.web_seeds
        _ = torrent.unknown_fields
        _ = torrent.validate()
    print(
        json.dumps(
            {
                "mode": arguments.mode,
                "files": arguments.files,
                "input_bytes": len(data),
                "peak_rss_bytes": peak_rss_bytes(),
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
