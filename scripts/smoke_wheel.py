"""Exercise the public API of an installed wheel on each release platform."""

from __future__ import annotations

import os
import sys
from itertools import pairwise
from pathlib import Path
from typing import TYPE_CHECKING

import btpc

if TYPE_CHECKING:
    from collections.abc import Callable

PIECE_LENGTH = 16_384


def require(condition: bool, message: str) -> None:  # noqa: FBT001
    if not condition:
        raise RuntimeError(message)


def expect_error(kind: type[Exception], action: Callable[[], object]) -> None:
    try:
        action()
    except kind:
        return
    raise RuntimeError(f"expected {kind.__name__}")


def check_mode(root: Path, mode: btpc.TorrentMode) -> None:
    payload = Path("payload")
    destination = root / f"payload-{mode.value}.torrent"
    events: list[tuple[int, int, int]] = []
    options = btpc.CreateOptions(
        mode=mode,
        piece_length=PIECE_LENGTH,
        creation_date=0,
        threads=2,
    )
    result = btpc.create(
        payload,
        destination,
        options=options,
        progress=lambda *event: events.append(event),
    )
    sequential = btpc.create_bytes(
        payload,
        options=btpc.CreateOptions(
            mode=mode,
            piece_length=PIECE_LENGTH,
            creation_date=0,
            threads=1,
        ),
    )
    require(result.bytes == sequential.bytes, f"{mode.value}: thread parity")
    require(bool(events), f"{mode.value}: missing creation progress")
    require(events[-1][0] == result.payload_bytes, "incomplete creation progress")
    require(all(a[0] <= b[0] for a, b in pairwise(events)), "unordered progress")
    torrent = btpc.Metainfo.read(destination)
    require(torrent.mode is mode, "mode mismatch")
    require(torrent.magnet().startswith("magnet:?xt="), "invalid magnet")
    events.clear()
    require(
        torrent.verify(payload, progress=lambda *event: events.append(event)).is_valid,
        f"{mode.value}: relative directory verification",
    )
    require(bool(events), "missing verification progress")
    require(events[-1][0] == result.payload_bytes, "incomplete verification progress")
    require(
        btpc.Metainfo.from_bytes(result.bytes).to_bytes() == result.bytes,
        "canonical round trip changed bytes",
    )
    extension = btpc.BencodeDictionary(((b"nested", btpc.BencodeList((1, b"raw"))),))
    edited = torrent.edit(comment="edited", raw_top_level={b"x-extension": extension})
    require(edited.info_hash_v1 == torrent.info_hash_v1, "v1 top-level edit identity")
    require(edited.info_hash_v2 == torrent.info_hash_v2, "v2 top-level edit identity")
    require(
        any(
            field.key == b"x-extension" and field.value == extension
            for field in edited.unknown_fields
        ),
        "extension conversion",
    )
    require(edited.comment == b"edited", "text conversion")
    expect_error(
        btpc.PathError, lambda: btpc.create(payload, destination, options=options)
    )
    require(destination.read_bytes() == result.bytes, "no-clobber changed output")
    check_mismatches(torrent, payload, mode)
    check_callbacks(torrent, payload, options)
    single = Path("single")
    single.write_bytes(b"single file")
    single_torrent = btpc.Metainfo.from_bytes(
        btpc.create_bytes(single, options=options).bytes
    )
    require(single_torrent.verify(single).is_valid, "relative single-file verification")
    expect_error(
        btpc.ResourceLimitError,
        lambda: btpc.Metainfo.from_bytes(
            result.bytes, options=btpc.ParseOptions(max_total_input=1)
        ),
    )


def check_mismatches(
    torrent: btpc.Metainfo, payload: Path, mode: btpc.TorrentMode
) -> None:

    # Alter content across several pieces without changing the file size.
    target = payload / "nested" / "large"
    original = target.read_bytes()
    target.write_bytes(b"z" + original[1:])
    report = torrent.verify(payload)
    kinds = {mismatch.kind for mismatch in report.mismatches}
    if mode is not btpc.TorrentMode.V2:
        require(btpc.MismatchKind.V1_HASH in kinds, "missing v1 mismatch")
    if mode is not btpc.TorrentMode.V1:
        require(btpc.MismatchKind.V2_HASH in kinds, "missing v2 mismatch")
    require(
        len(torrent.verify(payload, fail_fast=True).mismatches) == 1,
        "fail-fast returned more than one mismatch",
    )
    target.write_bytes(original)
    extra = payload / "extra"
    extra.write_bytes(b"extra")
    require(torrent.verify(payload).is_valid, "extra-file ignore policy")
    require(
        any(
            item.kind is btpc.MismatchKind.EXTRA
            for item in torrent.verify(payload, extra_files=True).mismatches
        ),
        "extra-file reporting policy",
    )
    extra.unlink()
    target.unlink()
    require(
        any(
            item.kind is btpc.MismatchKind.MISSING
            for item in torrent.verify(payload).mismatches
        ),
        "missing-file report",
    )
    target.write_bytes(original + b"!")
    require(
        any(
            item.kind is btpc.MismatchKind.WRONG_SIZE
            for item in torrent.verify(payload).mismatches
        ),
        "wrong-size report",
    )
    target.write_bytes(original)


def check_callbacks(
    torrent: btpc.Metainfo, payload: Path, options: btpc.CreateOptions
) -> None:
    token = btpc.CancellationToken()
    token.cancel()
    expect_error(
        btpc.CancelledError, lambda: torrent.verify(payload, cancellation=token)
    )
    expect_error(
        btpc.CancelledError,
        lambda: btpc.create_bytes(payload, options=options, cancellation=token),
    )

    def fail(*_event: int) -> None:
        raise RuntimeError("wheel callback failure")

    expect_error(RuntimeError, lambda: torrent.verify(payload, progress=fail))
    expect_error(
        RuntimeError, lambda: btpc.create_bytes(payload, options=options, progress=fail)
    )
    token = btpc.CancellationToken()

    def cancel(*_event: int) -> None:
        token.cancel()

    expect_error(
        btpc.CancelledError,
        lambda: torrent.verify(payload, progress=cancel, cancellation=token),
    )


def check_bencode_collections() -> None:
    values = btpc.BencodeList((1, b"raw"))
    require(list(values) == [1, b"raw"], "bencode list iteration")
    require(values[1:] == btpc.BencodeList((b"raw",)), "bencode list slicing")
    mapping = btpc.BencodeDictionary(((b"value", values),))
    require(dict(mapping) == {b"value": values}, "bencode dictionary conversion")
    require(mapping.get(b"missing") is None, "bencode dictionary default")


def main() -> None:
    root = Path(sys.argv[1]).resolve()
    root.mkdir(parents=True)
    payload = root / "payload"
    (payload / "nested").mkdir(parents=True)
    for length in [0, PIECE_LENGTH - 1, PIECE_LENGTH, PIECE_LENGTH + 1]:
        (payload / f"boundary-{length}").write_bytes(b"x" * length)
    (payload / "nested" / "large").write_bytes(b"a" * (PIECE_LENGTH * 3 + 7))
    for index in range(100):
        (payload / f"small-{index:03}").write_bytes(b"small")
    previous = Path.cwd()
    try:
        os.chdir(root)
        for mode in btpc.TorrentMode:
            check_mode(root, mode)
        check_bencode_collections()
        expect_error(btpc.BencodeError, lambda: btpc.Metainfo.from_bytes(b"invalid"))
    finally:
        os.chdir(previous)


if __name__ == "__main__":
    main()
