from __future__ import annotations

import os
import sys
import threading
from typing import TYPE_CHECKING

import btpc
import pytest

if TYPE_CHECKING:
    from pathlib import Path

GIL_PROGRESS_MINIMUM = 100


def test_metainfo_and_top_level_verify_all_modes(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"verification payload")
    for mode in btpc.TorrentMode:
        result = btpc.create_bytes(payload, options=btpc.CreateOptions(mode=mode))
        metainfo = btpc.Metainfo.from_bytes(result.bytes)
        report = metainfo.verify(payload)
        assert report.is_valid
        assert not report.mismatches
        assert btpc.verify(metainfo, payload) == report

        payload.write_bytes(b"verification payloae")
        mismatch = metainfo.verify(payload)
        assert not mismatch.is_valid
        expected = (
            btpc.MismatchKind.V2_HASH
            if mode is btpc.TorrentMode.V2
            else btpc.MismatchKind.V1_HASH
        )
        assert mismatch.mismatches[0].kind is expected
        payload.write_bytes(b"verification payload")


def test_verify_reports_structure_extra_and_fail_fast(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.mkdir()
    (payload / "a").write_bytes(b"a")
    (payload / "b").write_bytes(b"b")
    metainfo = btpc.Metainfo.from_bytes(btpc.create_bytes(payload).bytes)
    (payload / "a").unlink()
    (payload / "b").unlink()
    (payload / "extra").write_bytes(b"extra")

    report = metainfo.verify(payload, extra_files=True)
    kinds = {mismatch.kind for mismatch in report.mismatches}
    assert btpc.MismatchKind.MISSING in kinds
    assert btpc.MismatchKind.EXTRA in kinds
    fail_fast = metainfo.verify(payload, fail_fast=True, extra_files=True)
    assert len(fail_fast.mismatches) == 1


def test_verify_callbacks_exceptions_cancellation_and_gil(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"x" * (16 * 1024 * 1024))
    metainfo = btpc.Metainfo.from_bytes(
        btpc.create_bytes(
            payload, options=btpc.CreateOptions(mode=btpc.TorrentMode.V2)
        ).bytes
    )
    events: list[tuple[int, int, int]] = []
    assert metainfo.verify(
        payload, progress=lambda *event: events.append(event)
    ).is_valid
    assert events[-1][0] == payload.stat().st_size

    def fail(*_event: int) -> None:
        message = "callback failed"
        raise RuntimeError(message)

    with pytest.raises(RuntimeError, match="callback failed"):
        metainfo.verify(payload, progress=fail)

    cancellation = btpc.CancellationToken()

    def cancel(*_event: int) -> None:
        cancellation.cancel()

    with pytest.raises(btpc.CancelledError):
        metainfo.verify(payload, progress=cancel, cancellation=cancellation)

    started = threading.Event()
    finished = threading.Event()
    counter = 0

    def worker() -> None:
        nonlocal counter
        started.set()
        while not finished.is_set():
            counter += 1

    thread = threading.Thread(target=worker)
    thread.start()
    started.wait()
    metainfo.verify(payload)
    finished.set()
    thread.join()
    assert counter > GIL_PROGRESS_MINIMUM


@pytest.mark.skipif(sys.platform == "win32", reason="Unix byte-path semantics")
def test_mismatch_paths_round_trip_non_utf8_bytes(tmp_path: Path) -> None:
    raw_name = b"bad-\xff"
    missing = tmp_path / os.fsdecode(raw_name)
    with pytest.raises(btpc.PathError) as raised:
        btpc.Metainfo.read(missing)
    assert raised.value.path is not None
    assert isinstance(raised.value.path, type(tmp_path))
    assert os.fsencode(raised.value.path.name) == raw_name


@pytest.mark.skipif(sys.platform != "win32", reason="Windows UTF-16 path semantics")
def test_error_paths_round_trip_unpaired_utf16_surrogates(tmp_path: Path) -> None:
    missing = tmp_path / "bad-\ud800"
    with pytest.raises(btpc.PathError) as raised:
        btpc.Metainfo.read(missing)
    assert raised.value.path == missing
