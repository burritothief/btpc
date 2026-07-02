from __future__ import annotations

import csv
import hashlib
import json
import os
import random
import shutil
import subprocess
import sys
import time
from dataclasses import replace
from pathlib import Path

import psutil
import pytest

from benches.btpc_bench.adapters import adapter_registry
from benches.btpc_bench.cli import main
from benches.btpc_bench.dataset import (
    CANONICAL_ISO_NAME,
    CANONICAL_ISO_SHA256,
    CANONICAL_ISO_SIZE,
    fingerprint_payload,
    generate_dataset,
    load_manifest,
    validate_canonical_iso,
)
from benches.btpc_bench.models import (
    BenchmarkResult,
    DatasetFingerprint,
    RunSample,
    ToolResult,
    ToolStatus,
)
from benches.btpc_bench.report import render_markdown, render_summary, write_reports
from benches.btpc_bench.runner import (
    BenchmarkConfig,
    _isolated_environment,
    _measure_process,
    randomized_blocks,
    run_benchmark,
)
from benches.btpc_bench.validation import (
    ExpectedTorrent,
    ValidationFailure,
    validate_torrent,
)

TRACKER = "https://tracker.invalid/announce"
PIECE_LENGTH = 16 * 1024


# Spec: BENCH-DATA-001
def test_canonical_iso_preflight_rejects_mislabeled_payload() -> None:
    valid = DatasetFingerprint(
        path=CANONICAL_ISO_NAME,
        size_bytes=CANONICAL_ISO_SIZE,
        sha256=CANONICAL_ISO_SHA256,
        piece_length=4 * 1024 * 1024,
        piece_sha1=(),
    )
    validate_canonical_iso(valid)
    invalid = DatasetFingerprint(
        path=CANONICAL_ISO_NAME,
        size_bytes=1,
        sha256="0" * 64,
        piece_length=4 * 1024 * 1024,
        piece_sha1=(),
    )
    with pytest.raises(ValueError, match="fingerprint mismatch"):
        validate_canonical_iso(invalid)


def _bencode(value: object) -> bytes:
    if isinstance(value, int):
        return f"i{value}e".encode()
    if isinstance(value, bytes):
        return str(len(value)).encode() + b":" + value
    if isinstance(value, list):
        return b"l" + b"".join(_bencode(item) for item in value) + b"e"
    if isinstance(value, dict):
        items = sorted(value.items())
        return (
            b"d"
            + b"".join(_bencode(key) + _bencode(item) for key, item in items)
            + b"e"
        )
    raise TypeError(type(value))


def _torrent_bytes(payload: bytes, *, name: bytes = b"payload.bin") -> bytes:
    pieces = b"".join(
        hashlib.sha1(payload[offset : offset + PIECE_LENGTH]).digest()
        for offset in range(0, len(payload), PIECE_LENGTH)
    )
    return _bencode(
        {
            b"announce": TRACKER.encode(),
            b"info": {
                b"length": len(payload),
                b"name": name,
                b"piece length": PIECE_LENGTH,
                b"pieces": pieces,
                b"private": 1,
            },
        }
    )


def test_dataset_generation_is_reproducible(tmp_path: Path) -> None:
    first = generate_dataset(tmp_path / "one", seed=73, size_bytes=131_099)
    second = generate_dataset(tmp_path / "two", seed=73, size_bytes=131_099)

    assert first.sha256 == second.sha256
    assert first.piece_sha1 == second.piece_sha1
    assert first.payload_path.read_bytes() == second.payload_path.read_bytes()
    assert load_manifest(first.manifest_path) == load_manifest(second.manifest_path)
    assert json.loads(first.manifest_path.read_text())["seed"] == 73


def test_fingerprint_records_name_mtime_and_piece_boundaries(tmp_path: Path) -> None:
    payload = tmp_path / "named.bin"
    contents = b"abcdefghij"
    payload.write_bytes(contents)

    fingerprint = fingerprint_payload(payload, piece_length=4)

    assert fingerprint.name == "named.bin"
    assert fingerprint.mtime_ns == payload.stat().st_mtime_ns
    assert fingerprint.size_bytes == len(contents)
    assert fingerprint.sha256 == hashlib.sha256(contents).hexdigest()
    assert fingerprint.piece_sha1 == tuple(
        hashlib.sha1(contents[offset : offset + 4]).hexdigest()
        for offset in range(0, len(contents), 4)
    )


# Spec: BENCH-ENV-001
def test_isolated_environment_has_stable_child_configuration(tmp_path: Path) -> None:
    environment = _isolated_environment(tmp_path / "home", tmp_path / "project")

    assert environment["HOME"] == str(tmp_path / "home")
    assert environment["XDG_CONFIG_HOME"] == str(tmp_path / "home/.config")
    assert environment["XDG_CACHE_HOME"] == str(tmp_path / "home/.cache")
    assert environment["LC_ALL"] == "C"
    assert environment["TZ"] == "UTC"
    assert environment["NO_COLOR"] == "1"
    assert environment["PYTHONHASHSEED"] == "0"


# Spec: BENCH-REPRO-001
def test_cli_preflight_writes_versioned_json_without_running_tools(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    payload = tmp_path / "payload.bin"
    payload.write_bytes(b"payload")
    output = tmp_path / "preflight.json"

    assert main(["preflight", str(payload), "--output", str(output)]) == 0
    document = json.loads(output.read_text())
    assert document["schema_version"] == 1
    assert document["dataset"]["name"] == "payload.bin"
    assert document["dataset"]["size_bytes"] == 7
    assert "elapsed_seconds" in document
    assert str(output) in capsys.readouterr().out


# Spec: BENCH-PROFILE-001
# Spec: BENCH-TOOLS-001
def test_adapters_construct_equivalent_v1_commands(tmp_path: Path) -> None:
    payload = tmp_path / "payload.bin"
    output = tmp_path / "result.torrent"
    registry = adapter_registry(project_root=tmp_path, python=sys.executable)

    native = registry["btpc-native"].build_command(
        Path("/bin/btpc"), payload, output, TRACKER, 22
    )
    assert native == [
        "/bin/btpc",
        "create",
        str(payload),
        "--output",
        str(output),
        "--mode",
        "v1",
        "--piece-length",
        "4194304",
        "--tracker",
        TRACKER,
        "--private",
        "--quiet",
    ]

    assert registry["mktorrent"].build_command(
        Path("/usr/bin/mktorrent"), payload, output, TRACKER, 22
    ) == [
        "/usr/bin/mktorrent",
        "-a",
        TRACKER,
        "-l",
        "22",
        "-d",
        "-p",
        "-o",
        str(output),
        str(payload),
    ]
    assert "--no-creation-date" in registry["torrenttools"].build_command(
        Path("/usr/bin/torrenttools"), payload, output, TRACKER, 22
    )
    assert "--nodate" in registry["torf-cli"].build_command(
        Path("/usr/bin/torf"), payload, output, TRACKER, 22
    )
    assert "--noconfig" in registry["torf-cli"].build_command(
        Path("/usr/bin/torf"), payload, output, TRACKER, 22
    )
    assert registry["mkbrr"].build_command(
        Path("/usr/bin/mkbrr"), payload, output, TRACKER, 22
    )[0:3] == ["/usr/bin/mkbrr", "create", str(payload)]
    assert registry["mktorrent"].piece_size_semantics == "binary exponent"
    assert registry["mktorrent"].with_workers is not None
    assert "--threads" in registry["mktorrent"].with_workers(
        registry["mktorrent"].build_command(
            Path("/usr/bin/mktorrent"), payload, output, TRACKER, 22
        ),
        1,
    )


def test_adapter_discovery_preserves_unavailable_state(tmp_path: Path) -> None:
    adapter = adapter_registry(project_root=tmp_path, python=sys.executable)["mkbrr"]
    availability = adapter.discover(path_env=str(tmp_path))

    assert availability.status is ToolStatus.UNAVAILABLE
    assert availability.executable is None
    assert "not found" in availability.reason


# Spec: BENCH-RUN-001
def test_randomized_blocks_are_seeded_and_blocked() -> None:
    tools = ["a", "b", "c"]
    order = randomized_blocks(tools, rounds=4, rng=random.Random(9))

    assert order == randomized_blocks(tools, rounds=4, rng=random.Random(9))
    assert len(order) == 12
    for start in range(0, len(order), len(tools)):
        assert sorted(order[start : start + len(tools)]) == tools


# Spec: BENCH-METRIC-001
def test_process_measurement_samples_child_rss_and_times_out(tmp_path: Path) -> None:
    allocator = tmp_path / "allocator.py"
    allocator.write_text(
        "import subprocess, sys, time\n"
        "child = subprocess.Popen([sys.executable, '-c', "
        "'import time; data=bytearray(24*1024*1024); time.sleep(0.25)'])\n"
        "data = bytearray(16*1024*1024)\n"
        "child.wait()\n"
    )
    measurement = _measure_process(
        [sys.executable, str(allocator)],
        cwd=tmp_path,
        env=dict(os.environ),
        stdout_path=tmp_path / "stdout.log",
        stderr_path=tmp_path / "stderr.log",
        timeout_seconds=2.0,
        sample_interval_seconds=0.005,
    )
    assert measurement.exit_code == 0
    assert not measurement.timed_out
    assert measurement.peak_rss_bytes is not None
    assert measurement.peak_rss_bytes >= 30 * 1024 * 1024

    timeout = _measure_process(
        [sys.executable, "-c", "import time; time.sleep(30)"],
        cwd=tmp_path,
        env=dict(os.environ),
        stdout_path=tmp_path / "timeout.stdout.log",
        stderr_path=tmp_path / "timeout.stderr.log",
        timeout_seconds=0.05,
        sample_interval_seconds=0.005,
    )
    assert timeout.timed_out
    assert timeout.exit_code != 0
    assert timeout.signal is not None


def test_process_measurement_interrupt_terminates_child(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    pid_path = tmp_path / "pid"
    script = tmp_path / "wait.py"
    script.write_text(
        "import os, pathlib, time\n"
        f"pathlib.Path({str(pid_path)!r}).write_text(str(os.getpid()))\n"
        "time.sleep(30)\n"
    )
    calls = 0
    started_process: psutil.Process | None = None

    def interrupt_after_start(root: psutil.Process) -> tuple[int, float, float]:
        nonlocal calls, started_process
        calls += 1
        if calls > 1:
            started_process = root
            raise KeyboardInterrupt
        time.sleep(0.02)
        return 0, 0.0, 0.0

    monkeypatch.setattr(
        "benches.btpc_bench.runner._sample_process_tree", interrupt_after_start
    )
    with pytest.raises(KeyboardInterrupt):
        _measure_process(
            [sys.executable, str(script)],
            cwd=tmp_path,
            env=dict(os.environ),
            stdout_path=tmp_path / "interrupt.stdout.log",
            stderr_path=tmp_path / "interrupt.stderr.log",
            timeout_seconds=2.0,
            sample_interval_seconds=0.01,
        )
    assert pid_path.exists()
    assert started_process is not None
    assert (
        not started_process.is_running()
        or started_process.status() == psutil.STATUS_ZOMBIE
    )


# Spec: BENCH-CACHE-001
def test_cold_cache_requires_explicit_preparation_command(tmp_path: Path) -> None:
    payload = tmp_path / "payload.bin"
    payload.write_bytes(b"payload")
    config = BenchmarkConfig(
        input_path=payload,
        output_root=tmp_path / "results",
        tools=(),
        cache_state="cold",
    )
    with pytest.raises(ValueError, match="explicit preparation"):
        run_benchmark(config, registry={})


# Spec: BENCH-VALID-001
def test_independent_validator_checks_every_piece_and_semantics(tmp_path: Path) -> None:
    payload = bytes(range(251)) * 271
    torrent = tmp_path / "valid.torrent"
    torrent.write_bytes(_torrent_bytes(payload))
    expected = ExpectedTorrent.from_payload(
        payload_path=tmp_path / "payload.bin",
        payload=payload,
        tracker=TRACKER,
        piece_length=PIECE_LENGTH,
    )

    result = validate_torrent(torrent, expected, require_btpc=False)
    assert result.valid
    assert len(result.info_hash_v1) == 40

    damaged = bytearray(torrent.read_bytes())
    digest = hashlib.sha1(payload[:PIECE_LENGTH]).digest()
    index = damaged.index(digest)
    damaged[index] ^= 0x01
    torrent.write_bytes(damaged)
    invalid = validate_torrent(torrent, expected, require_btpc=False)
    assert not invalid.valid
    assert "piece digest mismatch" in invalid.reason

    extra_info = _torrent_bytes(payload).replace(
        b"d6:length",
        b"d5:xseed1:x6:length",
        1,
    )
    torrent.write_bytes(extra_info)
    invalid = validate_torrent(torrent, expected, require_btpc=False)
    assert not invalid.valid
    assert "unexpected info fields" in invalid.reason


@pytest.mark.parametrize(
    ("replacement", "reason"),
    [
        ({b"name": b"wrong.bin"}, "name mismatch"),
        ({b"length": 1}, "length mismatch"),
        ({b"piece length": 32 * 1024}, "piece length mismatch"),
        ({b"private": 0}, "private flag mismatch"),
    ],
)
def test_validator_classifies_profile_mismatches(
    tmp_path: Path,
    replacement: dict[bytes, object],
    reason: str,
) -> None:
    payload = b"payload"
    expected = ExpectedTorrent.from_payload(
        payload_path=tmp_path / "payload.bin",
        payload=payload,
        tracker=TRACKER,
        piece_length=PIECE_LENGTH,
    )
    info: dict[bytes, object] = {
        b"length": len(payload),
        b"name": b"payload.bin",
        b"piece length": PIECE_LENGTH,
        b"pieces": hashlib.sha1(payload).digest(),
        b"private": 1,
    }
    info.update(replacement)
    torrent = tmp_path / "profile.torrent"
    torrent.write_bytes(_bencode({b"announce": TRACKER.encode(), b"info": info}))

    result = validate_torrent(torrent, expected, require_btpc=False)

    assert result.failure is ValidationFailure.PROFILE
    assert reason in result.reason


def test_validator_accepts_top_level_metadata_and_enforces_raw_info_hash(
    tmp_path: Path,
) -> None:
    payload = b"payload"
    expected = ExpectedTorrent.from_payload(
        payload_path=tmp_path / "payload.bin",
        payload=payload,
        tracker=TRACKER,
        piece_length=PIECE_LENGTH,
    )
    base = _torrent_bytes(payload)
    decorated = base.replace(
        b"d8:announce",
        b"d10:created by4:test13:creation datei1e8:announce",
        1,
    )
    first = tmp_path / "first.torrent"
    second = tmp_path / "second.torrent"
    first.write_bytes(base)
    second.write_bytes(decorated)

    baseline = validate_torrent(first, expected, require_btpc=False)
    matching = validate_torrent(
        second,
        expected,
        require_btpc=False,
        expected_info_hash=baseline.info_hash_v1,
    )
    assert matching.valid
    assert matching.created_by == b"test"
    assert matching.creation_date == 1

    second.write_bytes(base.replace(b"7:privatei1e", b"7:privatei01e"))
    mismatch = validate_torrent(
        second,
        expected,
        require_btpc=False,
        expected_info_hash=baseline.info_hash_v1,
    )
    assert mismatch.failure is ValidationFailure.INFO_HASH


def test_validator_distinguishes_parse_profile_and_payload_failures(
    tmp_path: Path,
) -> None:
    payload = b"payload"
    expected = ExpectedTorrent.from_payload(
        payload_path=tmp_path / "payload.bin",
        payload=payload,
        tracker=TRACKER,
        piece_length=PIECE_LENGTH,
    )
    torrent = tmp_path / "case.torrent"

    torrent.write_bytes(b"d4:info")
    parsed = validate_torrent(torrent, expected, require_btpc=False)
    assert parsed.failure is ValidationFailure.PARSE

    multifile_info = {
        b"files": [{b"length": len(payload), b"path": [b"payload.bin"]}],
        b"name": b"payload.bin",
        b"piece length": PIECE_LENGTH,
        b"pieces": hashlib.sha1(payload).digest(),
        b"private": 1,
    }
    torrent.write_bytes(
        _bencode({b"announce": TRACKER.encode(), b"info": multifile_info})
    )
    multifile = validate_torrent(torrent, expected, require_btpc=False)
    assert multifile.failure is ValidationFailure.PROFILE

    torrent.write_bytes(
        _bencode(
            {
                b"announce": b"https://wrong.invalid",
                b"info": {
                    b"length": len(payload),
                    b"name": b"payload.bin",
                    b"piece length": PIECE_LENGTH,
                    b"pieces": hashlib.sha1(payload).digest(),
                    b"private": 1,
                },
            }
        )
    )
    tracker = validate_torrent(torrent, expected, require_btpc=False)
    assert tracker.failure is ValidationFailure.PROFILE
    assert "tracker mismatch" in tracker.reason

    torrent.write_bytes(
        _bencode(
            {
                b"announce": TRACKER.encode(),
                b"info": {
                    b"length": len(payload),
                    b"name": b"payload.bin",
                    b"piece length": PIECE_LENGTH,
                    b"pieces": b"",
                    b"private": 1,
                },
            }
        )
    )
    pieces = validate_torrent(torrent, expected, require_btpc=False)
    assert pieces.failure is ValidationFailure.PAYLOAD


def test_independent_validator_cross_checks_btpc_raw_hash(tmp_path: Path) -> None:
    payload = b"payload"
    expected = ExpectedTorrent.from_payload(
        payload_path=tmp_path / "payload.bin",
        payload=payload,
        tracker=TRACKER,
        piece_length=PIECE_LENGTH,
    )
    torrent = tmp_path / "valid.torrent"
    torrent.write_bytes(_torrent_bytes(payload))

    result = validate_torrent(torrent, expected)
    parsed = __import__("btpc").Metainfo.read(torrent)

    assert result.valid
    assert parsed.info_hash_v1 is not None
    assert result.info_hash_v1 == parsed.info_hash_v1.hex


def test_independent_validator_cross_checks_external_torf_when_available(
    tmp_path: Path,
) -> None:
    executable = shutil.which("torf")
    if executable is None:
        pytest.skip("torf CLI is unavailable")
    payload = b"payload"
    expected = ExpectedTorrent.from_payload(
        payload_path=tmp_path / "payload.bin",
        payload=payload,
        tracker=TRACKER,
        piece_length=PIECE_LENGTH,
    )
    torrent = tmp_path / "valid.torrent"
    torrent.write_bytes(_torrent_bytes(payload))
    independent = validate_torrent(torrent, expected, require_btpc=False)

    completed = subprocess.run(  # noqa: S603
        [executable, "-i", str(torrent), "--json", "--noconfig"],
        check=True,
        capture_output=True,
        text=True,
    )
    external = json.loads(completed.stdout)

    assert external["Info Hash"] == independent.info_hash_v1
    assert external["Private"] is True
    assert external["Piece Size"] == PIECE_LENGTH
    assert external["Piece Count"] == 1


# Spec: BENCH-OUTPUT-001
def test_report_generation_keeps_failures_below_ranked_rows(tmp_path: Path) -> None:
    fingerprint = DatasetFingerprint(
        path="payload.bin",
        size_bytes=8 * 1024 * 1024,
        sha256="ab" * 32,
        piece_length=4 * 1024 * 1024,
        piece_sha1=("01" * 20, "02" * 20),
    )
    success = ToolResult(
        name="btpc-native",
        version="btpc 0.1.0",
        status=ToolStatus.SUCCESS,
        info_hash_v1="12" * 20,
        samples=(
            RunSample(0, 0, 1.0, 0.7, 0.2, 8.0, 1024, True, "warm"),
            RunSample(1, 0, 1.2, 0.8, 0.2, 6.6667, 2048, True, "warm"),
        ),
    )
    missing = ToolResult(
        name="mkbrr",
        version="unavailable",
        status=ToolStatus.UNAVAILABLE,
        reason="executable not found",
    )
    result = BenchmarkResult.example(fingerprint, (success, missing))

    text = render_summary(result)
    markdown = render_markdown(result)
    paths = write_reports(result, tmp_path)

    assert text.index("btpc-native") < text.index("mkbrr")
    assert "1.00x" in text
    assert "unavailable: executable not found" in text
    assert "| btpc-native |" in markdown
    assert paths.json_path.exists()
    assert paths.json_path.name == "results.json"
    assert paths.compatibility_json_path.exists()
    assert paths.csv_path.exists()
    assert paths.markdown_path.read_text() == markdown
    assert paths.summary_path.read_text() == text
    with paths.csv_path.open(newline="") as stream:
        rows = list(csv.DictReader(stream))
    assert rows[0]["signal"] == ""
    assert rows[0]["output_size_bytes"] == ""
    assert rows[0]["timed_out"] == "False"


def test_result_json_round_trip_preserves_failed_samples() -> None:
    result = BenchmarkResult.example(
        DatasetFingerprint("x", 1, "00", 1, ("11",)),
        (
            ToolResult(
                "broken",
                "1.0",
                ToolStatus.FAILED,
                reason="exit 2",
                samples=(RunSample(0, 2, 0.1, 0.0, 0.0, 0.0, None, False, "cold"),),
            ),
        ),
    )

    decoded = BenchmarkResult.from_json(result.to_json())
    assert decoded == result
    assert decoded.tools[0].samples[0].exit_code == 2


def test_result_json_rejects_unknown_future_schema() -> None:
    document = json.loads(
        BenchmarkResult.example(
            DatasetFingerprint("x", 1, "00", 1, ("11",)), ()
        ).to_json()
    )
    document["schema_version"] = 99

    with pytest.raises(ValueError, match="unsupported benchmark schema"):
        BenchmarkResult.from_json(json.dumps(document))


def test_report_ties_are_sorted_by_tool_name_and_unicode_is_stable() -> None:
    dataset = DatasetFingerprint("payload.bin", 1, "00", 1, ("11",))
    tools = tuple(
        ToolResult(
            name,
            "versión 🚀",
            ToolStatus.SUCCESS,
            samples=(RunSample(0, 0, 1.0, 0.0, 0.0, 1.0, None, True, "warm"),),
        )
        for name in ("zeta", "alpha")
    )

    rendered = render_summary(BenchmarkResult.example(dataset, tools))

    assert rendered.index("alpha") < rendered.index("zeta")
    assert "versión 🚀" in rendered
    narrow = render_summary(BenchmarkResult.example(dataset, tools), max_width=140)
    assert max(len(line) for line in narrow.splitlines()) <= 140


@pytest.mark.parametrize("name", ["btpc-native", "btpc-python"])
def test_btpc_adapters_have_explicit_version_commands(
    name: str, tmp_path: Path
) -> None:
    adapter = adapter_registry(project_root=tmp_path, python=sys.executable)[name]
    assert adapter.version_args


def test_runner_records_failure_without_aborting_other_tools(tmp_path: Path) -> None:
    payload = generate_dataset(tmp_path / "data", seed=9, size_bytes=64 * 1024)
    failing = tmp_path / "failing"
    failing.write_text("#!/bin/sh\nexit 7\n")
    failing.chmod(0o755)
    registry = adapter_registry(project_root=tmp_path, python=sys.executable)
    registry["failing"] = registry["btpc-native"].__class__(
        name="failing",
        executables=(str(failing),),
        version_args=("--version",),
        build_command=lambda executable, _payload, _output, _tracker, _exponent: [
            str(executable)
        ],
        executable_override=failing,
    )
    config = BenchmarkConfig(
        input_path=payload.payload_path,
        output_root=tmp_path / "results",
        tools=("failing", "missing"),
        warmups=0,
        rounds=1,
        seed=2,
        tracker=TRACKER,
        piece_exponent=14,
        cache_state="warm",
    )

    result, output = run_benchmark(config, registry=registry)

    assert output.exists()
    assert result.tools[0].status is ToolStatus.UNSUPPORTED
    assert result.tools[1].status is ToolStatus.UNAVAILABLE
    assert "unknown adapter" in result.tools[1].reason


def test_runner_reports_smoke_failure_and_require_tools_blocks(tmp_path: Path) -> None:
    payload = generate_dataset(tmp_path / "data", seed=10, size_bytes=64 * 1024)
    executable = tmp_path / "smoke-fail"
    executable.write_text(
        "#!/bin/sh\n"
        'if [ "$1" = "--version" ]; then echo \'smoke-fail 1.0\'; exit 0; fi\n'
        "exit 9\n"
    )
    executable.chmod(0o755)
    registry = {
        "smoke-fail": adapter_registry(project_root=tmp_path, python=sys.executable)[
            "btpc-native"
        ].__class__(
            name="smoke-fail",
            executables=(str(executable),),
            version_args=("--version",),
            build_command=lambda resolved, *_arguments: [str(resolved), "create"],
            executable_override=executable,
        )
    }
    config = BenchmarkConfig(
        input_path=payload.payload_path,
        output_root=tmp_path / "results",
        tools=("smoke-fail",),
        warmups=0,
        rounds=1,
        seed=2,
        tracker=TRACKER,
        piece_exponent=14,
        cache_state="warm",
    )

    result, _output = run_benchmark(config, registry=registry)
    assert result.tools[0].status is ToolStatus.SMOKE_FAILED
    assert "exited 9" in result.tools[0].reason

    required = replace(config, output_root=tmp_path / "required", require_tools=True)
    with pytest.raises(RuntimeError, match="required tools"):
        run_benchmark(required, registry=registry)


def test_native_btpc_tiny_run_creates_valid_retained_torrent(tmp_path: Path) -> None:
    project_root = Path(__file__).parents[2]
    native = project_root / "target/debug/btpc"
    if not native.exists():
        pytest.skip("debug btpc binary not built")
    payload = generate_dataset(
        tmp_path / "data", seed=17, size_bytes=96 * 1024, piece_length=16 * 1024
    )
    config = BenchmarkConfig(
        input_path=payload.payload_path,
        output_root=tmp_path / "results",
        tools=("btpc-native",),
        warmups=0,
        rounds=1,
        seed=5,
        tracker=TRACKER,
        piece_exponent=14,
        cache_state="warm",
    )

    result, output = run_benchmark(
        config,
        registry=adapter_registry(project_root=project_root, python=sys.executable),
    )

    tool = result.tools[0]
    assert tool.status is ToolStatus.SUCCESS
    assert tool.samples[0].valid
    assert tool.samples[0].peak_rss_bytes is not None
    assert tool.samples[0].output_size_bytes is not None
    assert tool.samples[0].signal is None
    assert not tool.samples[0].timed_out
    assert (output / "torrents" / "btpc-native.torrent").exists()
    assert (output / "environment.json").exists()
    assert (output / "commands.json").exists()


def test_measured_failure_stops_later_rounds_and_retains_sample(tmp_path: Path) -> None:
    payload = generate_dataset(tmp_path / "data", seed=18, size_bytes=64 * 1024)
    executable = tmp_path / "fails-after-smoke.py"
    counter = tmp_path / "counter"
    fixture = tmp_path / "fixture.torrent"
    fixture.write_bytes(_torrent_bytes(payload.payload_path.read_bytes()))
    executable.write_text(
        "import pathlib, shutil, sys\n"
        f"counter = pathlib.Path({str(counter)!r})\n"
        "if '--version' in sys.argv: print('fake 1.0'); raise SystemExit(0)\n"
        "count = int(counter.read_text()) if counter.exists() else 0\n"
        "counter.write_text(str(count + 1))\n"
        "if count == 0:\n"
        "    shutil.copyfile(sys.argv[1], sys.argv[2])\n"
        "    raise SystemExit(0)\n"
        "raise SystemExit(7)\n"
    )
    registry = adapter_registry(project_root=tmp_path, python=sys.executable)
    registry["fake"] = registry["btpc-native"].__class__(
        name="fake",
        executables=(sys.executable,),
        version_args=(str(executable), "--version"),
        build_command=lambda resolved, _payload, output, _tracker, _exponent: [
            str(resolved),
            str(executable),
            str(fixture),
            str(output),
        ],
        executable_override=Path(sys.executable),
    )
    config = BenchmarkConfig(
        input_path=payload.payload_path,
        output_root=tmp_path / "results",
        tools=("fake",),
        warmups=0,
        rounds=3,
        seed=5,
        tracker=TRACKER,
        piece_exponent=14,
    )

    result, _output = run_benchmark(config, registry=registry)

    assert result.tools[0].status is ToolStatus.FAILED
    assert len(result.tools[0].samples) == 1
    assert result.tools[0].samples[0].exit_code == 7
    assert counter.read_text() == "2"


def test_warmup_runs_are_excluded_from_raw_samples(tmp_path: Path) -> None:
    project_root = Path(__file__).parents[2]
    payload = generate_dataset(
        tmp_path / "data", seed=19, size_bytes=96 * 1024, piece_length=16 * 1024
    )
    config = BenchmarkConfig(
        input_path=payload.payload_path,
        output_root=tmp_path / "results",
        tools=("btpc-native",),
        warmups=1,
        rounds=2,
        seed=5,
        tracker=TRACKER,
        piece_exponent=14,
    )

    result, output = run_benchmark(
        config,
        registry=adapter_registry(project_root=project_root, python=sys.executable),
    )

    assert len(result.tools[0].samples) == 2
    commands = json.loads((output / "commands.json").read_text())
    assert len(commands["btpc-native"]) == 4


def test_cli_renders_and_compares_saved_results(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    dataset = DatasetFingerprint("payload.bin", 1024, "ab" * 32, 1024, ("cd" * 20,))
    baseline = BenchmarkResult.example(
        dataset,
        (
            ToolResult(
                "btpc-native",
                "0.1",
                ToolStatus.SUCCESS,
                samples=(RunSample(0, 0, 2.0, 1.0, 0.1, 0.5, 1000, True, "warm"),),
            ),
        ),
    )
    candidate = BenchmarkResult.example(
        dataset,
        (
            ToolResult(
                "btpc-native",
                "0.2",
                ToolStatus.SUCCESS,
                samples=(RunSample(0, 0, 1.0, 0.5, 0.1, 1.0, 1000, True, "warm"),),
            ),
        ),
    )
    baseline_path = tmp_path / "baseline.json"
    candidate_path = tmp_path / "candidate.json"
    baseline_path.write_text(baseline.to_json())
    candidate_path.write_text(candidate.to_json())

    assert main(["render", str(candidate_path)]) == 0
    rendered = capsys.readouterr().out
    assert "btpc-native" in rendered
    assert main(["compare", str(baseline_path), str(candidate_path)]) == 0
    comparison = capsys.readouterr().out
    assert "2.00x faster" in comparison
    assert "-1.000000s (-50.00%)" in comparison
    assert "n=1 CV=0.0%" in comparison
    assert "descriptive only" in comparison


def test_cli_generates_reproducible_tiny_dataset(tmp_path: Path) -> None:
    root = tmp_path / "dataset"
    assert (
        main(
            [
                "generate",
                str(root),
                "--seed",
                "44",
                "--size-bytes",
                "4096",
                "--piece-exponent",
                "14",
            ]
        )
        == 0
    )
    first = (root / "payload.bin").read_bytes()
    shutil_root = tmp_path / "dataset-two"
    assert (
        main(
            [
                "generate",
                str(shutil_root),
                "--seed",
                "44",
                "--size-bytes",
                "4096",
                "--piece-exponent",
                "14",
            ]
        )
        == 0
    )
    assert first == (shutil_root / "payload.bin").read_bytes()
