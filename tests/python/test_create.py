from __future__ import annotations

import pickle
import shutil
import subprocess
import threading
from itertools import pairwise
from typing import TYPE_CHECKING

import btpc
import pytest

if TYPE_CHECKING:
    from pathlib import Path

# Spec: PYAPI-PARITY-001
# Spec: PYAPI-GIL-001
# Spec: PYAPI-TEXT-001

PAYLOAD_BYTES = 10
PIECE_LENGTH = 8192
GIL_PROGRESS_MINIMUM = 100


def test_create_bytes_result_options_and_cli_parity(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"abcdefghij")
    options = btpc.CreateOptions(
        piece_length=PIECE_LENGTH,
        trackers=(("https://tracker",),),
        web_seeds=("https://seed",),
        nodes=(("router.example", 6881),),
        private=True,
        source="source",
        comment="comment",
        created_by="python-test",
    )
    result = btpc.create_bytes(payload, options=options)
    assert result.bytes.startswith(b"d")
    assert result.file_count == 1
    assert result.payload_bytes == PAYLOAD_BYTES
    assert result.piece_length == PIECE_LENGTH
    assert result.piece_length_policy is None
    assert btpc.Metainfo.from_bytes(result.bytes).info_hash_v1 == result.info_hash_v1

    output = tmp_path / "cli.torrent"
    cargo = shutil.which("cargo")
    assert cargo is not None
    subprocess.run(  # noqa: S603
        [
            cargo,
            "run",
            "-q",
            "-p",
            "btpc-cli",
            "--",
            "create",
            str(payload),
            "-o",
            str(output),
            "--piece-length",
            str(PIECE_LENGTH),
            "--tracker",
            "https://tracker",
            "--web-seed",
            "https://seed",
            "--node",
            "router.example:6881",
            "--private",
            "--source",
            "source",
            "--comment",
            "comment",
            "--created-by",
            "python-test",
            "--quiet",
        ],
        check=True,
    )
    assert output.read_bytes() == result.bytes


def test_text_creation_inputs_accept_sequences_and_reject_bytes(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"text")
    options = btpc.CreateOptions(
        trackers=(("https://例え.invalid/announce",),),
        web_seeds=["https://seed.invalid/π"],
        nodes=[("router.invalid", 6881)],
        source="源",
        comment="コメント",
        created_by="作成者",
    )
    parsed = btpc.Metainfo.from_bytes(btpc.create_bytes(payload, options=options).bytes)
    assert parsed.trackers == (("https://例え.invalid/announce".encode(),),)
    assert parsed.web_seeds == ("https://seed.invalid/π".encode(),)

    for field, value in [
        ("trackers", ((b"https://tracker",),)),
        ("web_seeds", (b"https://seed",)),
        ("nodes", ((b"router", 6881),)),
        ("source", b"source"),
        ("comment", b"comment"),
        ("created_by", b"creator"),
    ]:
        with pytest.raises(TypeError, match=field):
            btpc.create_bytes(payload, options=btpc.CreateOptions(**{field: value}))  # type: ignore[arg-type]


def test_atomic_create_overwrite_and_cleanup(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    output = tmp_path / "payload.torrent"
    payload.write_bytes(b"data")
    first = btpc.create(payload, output)
    assert output.read_bytes() == first.bytes
    with pytest.raises(btpc.PathError):
        btpc.create(payload, output)
    replaced = btpc.create(payload, output, overwrite=True, durable=True)
    assert output.read_bytes() == replaced.bytes
    assert not list(tmp_path.glob("*.btpc-tmp*"))


def test_create_result_is_lazy_cached_immutable_and_not_subclassable(
    tmp_path: Path,
) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"result-policy")
    result = btpc.create_bytes(payload)
    native = result._native  # noqa: SLF001
    assert native.bytes is native.bytes
    assert result.bytes is result.bytes
    assert result.metrics is result.metrics
    assert "payload_bytes" in repr(result)
    with pytest.raises(AttributeError):
        result.file_count = 0  # type: ignore[misc]
    with pytest.raises(TypeError):
        pickle.dumps(result)
    with pytest.raises(TypeError):
        type("Child", (btpc.CreateResult,), {})


def test_progress_callback_exception_and_cancellation(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    output = tmp_path / "payload.torrent"
    payload.write_bytes(b"x" * (1024 * 1024))
    events: list[tuple[int, int, int]] = []
    result = btpc.create_bytes(payload, progress=lambda *event: events.append(event))
    assert events
    assert events[-1][0] == result.payload_bytes

    def fail(*_event: int) -> None:
        message = "callback failed"
        raise RuntimeError(message)

    with pytest.raises(RuntimeError, match="callback failed"):
        btpc.create(payload, output, progress=fail)
    assert not output.exists()
    assert not list(tmp_path.glob("*.btpc-tmp*"))

    cancellation = btpc.CancellationToken()

    def cancel(*_event: int) -> None:
        cancellation.cancel()

    with pytest.raises(btpc.CancelledError):
        btpc.create_bytes(payload, progress=cancel, cancellation=cancellation)
    assert cancellation.cancelled


def test_create_releases_gil(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"z" * (16 * 1024 * 1024))
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
    btpc.create_bytes(payload)
    finished.set()
    thread.join()
    assert counter > GIL_PROGRESS_MINIMUM


@pytest.mark.parametrize("mode", list(btpc.TorrentMode))
def test_all_modes_match_cli_and_report_applicable_hashes(
    tmp_path: Path, mode: btpc.TorrentMode
) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"mode parity")
    result = btpc.create_bytes(
        payload,
        options=btpc.CreateOptions(mode=mode, piece_length=16_384),
    )
    parsed = btpc.Metainfo.from_bytes(result.bytes)
    assert result.mode is mode
    assert parsed.mode is mode
    assert (result.info_hash_v1 is not None) is (mode is not btpc.TorrentMode.V2)
    assert (result.info_hash_v2 is not None) is (mode is not btpc.TorrentMode.V1)

    output = tmp_path / f"{mode.value}.torrent"
    cargo = shutil.which("cargo")
    assert cargo is not None
    subprocess.run(  # noqa: S603
        [
            cargo,
            "run",
            "-q",
            "-p",
            "btpc-cli",
            "--",
            "create",
            str(payload),
            "-o",
            str(output),
            "--mode",
            mode.value,
            "--piece-length",
            "16384",
            "--quiet",
        ],
        check=True,
    )
    assert output.read_bytes() == result.bytes

    automatic = btpc.create_bytes(payload, options=btpc.CreateOptions(mode=mode))
    automatic_output = tmp_path / f"{mode.value}-automatic.torrent"
    subprocess.run(  # noqa: S603
        [
            cargo,
            "run",
            "-q",
            "-p",
            "btpc-cli",
            "--",
            "create",
            str(payload),
            "-o",
            str(automatic_output),
            "--mode",
            mode.value,
            "--quiet",
        ],
        check=True,
    )
    assert automatic_output.read_bytes() == automatic.bytes


@pytest.mark.parametrize("mode", [btpc.TorrentMode.V2, btpc.TorrentMode.HYBRID])
def test_v2_modes_reject_v1_only_piece_lengths(
    tmp_path: Path, mode: btpc.TorrentMode
) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"data")
    with pytest.raises(btpc.MetainfoError, match="between 16384"):
        btpc.create_bytes(
            payload,
            options=btpc.CreateOptions(mode=mode, piece_length=8192),
        )


def test_hybrid_inspection_exposes_padding_entries(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.mkdir()
    (payload / "a").write_bytes(b"abc")
    (payload / "b").write_bytes(b"def")
    result = btpc.create_bytes(
        payload,
        options=btpc.CreateOptions(
            mode=btpc.TorrentMode.HYBRID,
            piece_length=16_384,
        ),
    )
    torrent = btpc.Metainfo.from_bytes(result.bytes)
    padding = [file for file in torrent.files if file.is_padding]
    assert len(padding) == 1
    assert padding[0].path[0] == b".pad"
    assert padding[0].attributes == b"p"


def test_python_thread_counts_match_sequential_oracle(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"parallel parity" * 300_000)
    sequential = btpc.create_bytes(
        payload,
        options=btpc.CreateOptions(piece_length=262_144, threads=1),
    )
    parallel = btpc.create_bytes(
        payload,
        options=btpc.CreateOptions(piece_length=262_144, threads=4),
    )
    automatic = btpc.create_bytes(
        payload,
        options=btpc.CreateOptions(piece_length=262_144, threads=0),
    )
    assert parallel.bytes == sequential.bytes
    assert parallel.info_hash_v1 == sequential.info_hash_v1
    assert automatic.bytes == sequential.bytes


@pytest.mark.parametrize("mode", list(btpc.TorrentMode))
def test_progress_and_cancellation_are_consistent_for_every_mode(
    tmp_path: Path, mode: btpc.TorrentMode
) -> None:
    payload = tmp_path / "payload"
    payload.mkdir()
    (payload / "a").write_bytes(b"a" * 40_000)
    (payload / "b").write_bytes(b"b" * 40_000)
    events: list[tuple[int, int, int]] = []
    result = btpc.create_bytes(
        payload,
        options=btpc.CreateOptions(mode=mode, piece_length=16_384),
        progress=lambda *event: events.append(event),
    )
    assert events[-1][0] == result.payload_bytes
    assert all(left[0] <= right[0] for left, right in pairwise(events))

    cancellation = btpc.CancellationToken()

    def cancel(*_event: int) -> None:
        cancellation.cancel()

    with pytest.raises(btpc.CancelledError):
        btpc.create_bytes(
            payload,
            options=btpc.CreateOptions(mode=mode, piece_length=16_384),
            progress=cancel,
            cancellation=cancellation,
        )


def test_creator_identity_defaults_overrides_and_omits(tmp_path: Path) -> None:
    # Spec: PYAPI-CREATOR-001
    payload = tmp_path / "payload"
    payload.write_bytes(b"creator")
    default = btpc.create_bytes(payload)
    explicit = btpc.create_bytes(
        payload, options=btpc.CreateOptions(created_by="custom/π")
    )
    omitted = btpc.create_bytes(
        payload, options=btpc.CreateOptions(omit_created_by=True)
    )
    assert b"10:created by10:btpc/0.1.0" in default.bytes
    assert "custom/π".encode() in explicit.bytes
    assert b"10:created by" not in omitted.bytes
    assert default.info_hash_v1 == explicit.info_hash_v1 == omitted.info_hash_v1
    with pytest.raises(TypeError, match="omit_created_by"):
        btpc.create_bytes(
            payload,
            options=btpc.CreateOptions(created_by="custom", omit_created_by=True),
        )
