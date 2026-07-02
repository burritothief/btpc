"""Reproducible benchmark execution and process resource sampling."""

from __future__ import annotations

import json
import locale
import os
import platform
import random
import shutil
import socket
import subprocess
import sys
import time
from contextlib import suppress
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

import psutil

from .adapters import ToolAdapter, adapter_registry
from .dataset import fingerprint_payload, validate_canonical_iso
from .models import BenchmarkResult, RunSample, ToolResult, ToolStatus
from .report import render_summary, write_reports
from .validation import ExpectedTorrent, validate_torrent

HARNESS_VERSION = "0.1.0"
PROFILE = "v1-single-file"
MIN_PIECE_EXPONENT = 14
MAX_PIECE_EXPONENT = 26


@dataclass(frozen=True, slots=True)
class BenchmarkConfig:
    """Inputs controlling one reproducible benchmark session."""

    input_path: Path
    output_root: Path
    tools: tuple[str, ...]
    warmups: int = 2
    rounds: int = 10
    seed: int = 20260701
    tracker: str = "https://tracker.invalid/announce"
    piece_exponent: int = 22
    profile: str = PROFILE
    preset: str = "standard"
    cache_state: str = "warm"
    require_tools: bool = False
    timeout_seconds: float = 600.0
    sample_interval_seconds: float = 0.01
    cache_prepare_command: tuple[str, ...] = ()


@dataclass(frozen=True, slots=True)
class ProcessMeasurement:
    """Measured process outcome and best-effort process-tree resources."""

    exit_code: int
    elapsed_seconds: float
    user_cpu_seconds: float
    system_cpu_seconds: float
    peak_rss_bytes: int | None
    signal: int | None = None
    timed_out: bool = False


def randomized_blocks(
    tools: list[str], *, rounds: int, rng: random.Random
) -> list[str]:
    """Randomize tool order independently inside each complete round."""
    order: list[str] = []
    for _ in range(rounds):
        block = list(tools)
        rng.shuffle(block)
        order.extend(block)
    return order


def run_benchmark(
    config: BenchmarkConfig,
    *,
    registry: dict[str, ToolAdapter] | None = None,
) -> tuple[BenchmarkResult, Path]:
    """Discover, smoke-test, run, validate, and report all selected tools."""
    _validate_config(config)
    project_root = Path(__file__).parents[2]
    registry = registry or adapter_registry(
        project_root=project_root, python=sys.executable
    )
    started = datetime.now(UTC)
    result_dir = _result_directory(config.output_root, started)
    logs_dir = result_dir / "logs"
    torrents_dir = result_dir / "torrents"
    homes_dir = result_dir / "homes"
    for directory in (logs_dir, torrents_dir, homes_dir):
        directory.mkdir(parents=True, exist_ok=True)

    fingerprint = fingerprint_payload(
        config.input_path, piece_length=1 << config.piece_exponent
    )
    validate_canonical_iso(fingerprint)
    expected = ExpectedTorrent.from_fingerprint(fingerprint, tracker=config.tracker)
    selected = config.tools or tuple(registry)
    ready: dict[str, tuple[ToolAdapter, Path, str]] = {}
    initial: dict[str, ToolResult] = {}
    commands: dict[str, list[list[str]]] = {}

    for name in selected:
        adapter = registry.get(name)
        if adapter is None:
            initial[name] = ToolResult(
                name,
                "unavailable",
                ToolStatus.UNAVAILABLE,
                reason="unknown adapter",
            )
            continue
        availability = adapter.discover()
        if (
            availability.status is not ToolStatus.READY
            or availability.executable is None
        ):
            initial[name] = ToolResult(
                name,
                availability.version,
                availability.status,
                reason=availability.reason,
                executable=str(availability.executable or ""),
            )
            continue
        if config.profile not in adapter.supported_profiles:
            initial[name] = ToolResult(
                name,
                availability.version,
                ToolStatus.UNSUPPORTED,
                reason=f"profile {config.profile!r} is unsupported",
                executable=str(availability.executable),
            )
            continue
        ready[name] = (adapter, availability.executable, availability.version)
        commands[name] = []

    if config.require_tools and len(ready) != len(selected):
        missing = ", ".join(name for name in selected if name not in ready)
        msg = f"required tools unavailable or unsupported: {missing}"
        raise RuntimeError(msg)

    active: dict[str, tuple[ToolAdapter, Path, str, str]] = {}
    expected_info_hash: str | None = None
    for name, (adapter, executable, version) in ready.items():
        output = result_dir / f"smoke-{name}.torrent"
        command = adapter.build_command(
            executable,
            config.input_path,
            output,
            config.tracker,
            config.piece_exponent,
        )
        commands[name].append(command)
        measurement = _measure_process(
            command,
            cwd=project_root,
            env=_isolated_environment(homes_dir / name, project_root),
            stdout_path=logs_dir / f"{name}-smoke.stdout.log",
            stderr_path=logs_dir / f"{name}-smoke.stderr.log",
            timeout_seconds=config.timeout_seconds,
            sample_interval_seconds=config.sample_interval_seconds,
        )
        if measurement.exit_code != 0:
            initial[name] = ToolResult(
                name,
                version,
                ToolStatus.SMOKE_FAILED,
                reason=f"smoke command exited {measurement.exit_code}",
                executable=str(executable),
                command_template=tuple(command),
            )
            continue
        validation = validate_torrent(
            output, expected, expected_info_hash=expected_info_hash
        )
        if not validation.valid:
            initial[name] = ToolResult(
                name,
                version,
                ToolStatus.SMOKE_FAILED,
                reason=f"smoke validation: {validation.reason}",
                executable=str(executable),
                command_template=tuple(command),
            )
            continue
        if expected_info_hash is None:
            expected_info_hash = validation.info_hash_v1
        active[name] = (adapter, executable, version, validation.info_hash_v1)
        output.unlink(missing_ok=True)

    if config.require_tools and len(active) != len(selected):
        failed = ", ".join(name for name in selected if name not in active)
        msg = f"required tools failed smoke validation: {failed}"
        raise RuntimeError(msg)

    for warmup in range(config.warmups):
        for name in sorted(active):
            adapter, executable, _version, _hash = active[name]
            output = result_dir / f"warmup-{name}.torrent"
            output.unlink(missing_ok=True)
            command = adapter.build_command(
                executable,
                config.input_path,
                output,
                config.tracker,
                config.piece_exponent,
            )
            commands[name].append(command)
            measurement = _measure_process(
                command,
                cwd=project_root,
                env=_isolated_environment(homes_dir / name, project_root),
                stdout_path=logs_dir / f"{name}-warmup-{warmup}.stdout.log",
                stderr_path=logs_dir / f"{name}-warmup-{warmup}.stderr.log",
                timeout_seconds=config.timeout_seconds,
                sample_interval_seconds=config.sample_interval_seconds,
            )
            warmup_validation = (
                validate_torrent(
                    output, expected, expected_info_hash=expected_info_hash
                )
                if measurement.exit_code == 0
                else None
            )
            if (
                measurement.exit_code != 0
                or warmup_validation is None
                or not warmup_validation.valid
            ):
                if measurement.exit_code != 0:
                    reason = f"warmup exited {measurement.exit_code}"
                elif warmup_validation is None:
                    reason = "warmup did not produce validation output"
                else:
                    reason = f"warmup validation: {warmup_validation.reason}"
                adapter_value = active.pop(name)
                initial[name] = ToolResult(
                    name,
                    adapter_value[2],
                    ToolStatus.FAILED if measurement.exit_code else ToolStatus.INVALID,
                    reason=reason,
                    executable=str(executable),
                    command_template=tuple(command),
                )
            output.unlink(missing_ok=True)

    order = randomized_blocks(
        list(active), rounds=config.rounds, rng=random.Random(config.seed)
    )
    samples: dict[str, list[RunSample]] = {name: [] for name in active}
    occurrence: dict[str, int] = dict.fromkeys(active, 0)
    failed_measured: set[str] = set()
    for name in order:
        if name in failed_measured:
            continue
        adapter, executable, _version, _hash = active[name]
        round_number = occurrence[name]
        occurrence[name] += 1
        output = result_dir / f"run-{name}-{round_number}.torrent"
        output.unlink(missing_ok=True)
        if config.cache_prepare_command:
            prepared = subprocess.run(
                config.cache_prepare_command,
                cwd=project_root,
                env=_isolated_environment(homes_dir / name, project_root),
                check=False,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            if prepared.returncode != 0:
                samples[name].append(
                    RunSample(
                        round=round_number,
                        exit_code=prepared.returncode,
                        elapsed_seconds=0.0,
                        user_cpu_seconds=0.0,
                        system_cpu_seconds=0.0,
                        throughput_mib_s=0.0,
                        peak_rss_bytes=None,
                        valid=False,
                        cache_state=config.cache_state,
                        log_prefix=f"{name}-round-{round_number}",
                        reason="cache preparation command failed",
                    )
                )
                failed_measured.add(name)
                continue
        command = adapter.build_command(
            executable,
            config.input_path,
            output,
            config.tracker,
            config.piece_exponent,
        )
        commands[name].append(command)
        log_prefix = f"{name}-round-{round_number}"
        measurement = _measure_process(
            command,
            cwd=project_root,
            env=_isolated_environment(homes_dir / name, project_root),
            stdout_path=logs_dir / f"{log_prefix}.stdout.log",
            stderr_path=logs_dir / f"{log_prefix}.stderr.log",
            timeout_seconds=config.timeout_seconds,
            sample_interval_seconds=config.sample_interval_seconds,
        )
        measured_validation = (
            validate_torrent(output, expected, expected_info_hash=expected_info_hash)
            if measurement.exit_code == 0
            else None
        )
        valid = measured_validation is not None and measured_validation.valid
        reason = ""
        if measurement.timed_out:
            reason = f"timed out after {config.timeout_seconds:g} seconds"
        elif measurement.exit_code != 0:
            reason = f"exit {measurement.exit_code}"
        elif measured_validation is not None and not measured_validation.valid:
            reason = measured_validation.reason
        throughput = (
            fingerprint.size_bytes / (1024 * 1024) / measurement.elapsed_seconds
            if measurement.elapsed_seconds > 0
            else 0.0
        )
        samples[name].append(
            RunSample(
                round=round_number,
                exit_code=measurement.exit_code,
                elapsed_seconds=measurement.elapsed_seconds,
                user_cpu_seconds=measurement.user_cpu_seconds,
                system_cpu_seconds=measurement.system_cpu_seconds,
                throughput_mib_s=throughput,
                peak_rss_bytes=measurement.peak_rss_bytes,
                valid=valid,
                cache_state=config.cache_state,
                log_prefix=log_prefix,
                reason=reason,
                signal=measurement.signal,
                output_size_bytes=(output.stat().st_size if output.is_file() else None),
                timed_out=measurement.timed_out,
            )
        )
        if valid:
            shutil.copy2(output, torrents_dir / f"{name}.torrent")
        else:
            failed_measured.add(name)
        output.unlink(missing_ok=True)

    tool_results: list[ToolResult] = []
    for name in selected:
        if name in initial:
            tool_results.append(initial[name])
            continue
        _adapter, executable, version, info_hash = active[name]
        tool_samples = tuple(samples[name])
        status = ToolStatus.SUCCESS
        reason = ""
        if any(sample.exit_code != 0 for sample in tool_samples):
            status = ToolStatus.FAILED
            reason = "one or more measured commands failed"
        elif any(not sample.valid for sample in tool_samples):
            status = ToolStatus.INVALID
            reason = "one or more measured torrents failed validation"
        tool_results.append(
            ToolResult(
                name,
                version,
                status,
                reason=reason,
                executable=str(executable),
                command_template=tuple(
                    active[name][0].build_command(
                        executable,
                        Path("{input}"),
                        Path("{output}"),
                        config.tracker,
                        config.piece_exponent,
                    )
                ),
                info_hash_v1=info_hash,
                samples=tool_samples,
            )
        )

    warnings: list[str] = []
    if config.cache_state == "cold":
        warnings.append(
            "cold-cache is caller-labeled; the harness does not flush "
            "operating-system caches"
        )
    result = BenchmarkResult(
        schema_version=1,
        harness_version=HARNESS_VERSION,
        started_at=started.isoformat().replace("+00:00", "Z"),
        seed=config.seed,
        profile=config.profile,
        preset=config.preset,
        cache_state=config.cache_state,
        warmups=config.warmups,
        rounds=config.rounds,
        tracker=config.tracker,
        piece_exponent=config.piece_exponent,
        dataset=fingerprint,
        machine=machine_metadata(),
        run_order=tuple(order),
        tools=tuple(tool_results),
        warnings=tuple(warnings),
    )
    (result_dir / "environment.json").write_text(
        json.dumps(result.machine, indent=2, sort_keys=True) + "\n"
    )
    (result_dir / "commands.json").write_text(
        json.dumps(commands, indent=2, sort_keys=True) + "\n"
    )
    write_reports(result, result_dir)
    print(render_summary(result), end="")
    return result, result_dir


def machine_metadata() -> dict[str, Any]:
    """Capture the machine and runtime context needed to interpret results."""
    memory = psutil.virtual_memory()
    return {
        "hostname": socket.gethostname(),
        "os": platform.system(),
        "os_release": platform.release(),
        "kernel": platform.version(),
        "architecture": platform.machine(),
        "processor": platform.processor(),
        "logical_cores": psutil.cpu_count(logical=True),
        "physical_cores": psutil.cpu_count(logical=False),
        "memory_bytes": memory.total,
        "filesystem": _filesystem_type(Path.cwd()),
        "python": sys.version.replace("\n", " "),
        "python_executable": sys.executable,
        "locale": locale.setlocale(locale.LC_ALL, None),
        "timezone": time.tzname,
        "psutil": psutil.__version__,
    }


def _measure_process(  # noqa: PLR0913
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    stdout_path: Path,
    stderr_path: Path,
    timeout_seconds: float = 600.0,
    sample_interval_seconds: float = 0.01,
) -> ProcessMeasurement:
    start = time.perf_counter()
    peak_rss = 0
    user_cpu = 0.0
    system_cpu = 0.0
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdout=stdout,
            stderr=stderr,
        )
        root = psutil.Process(process.pid)
        timed_out = False
        try:
            while process.poll() is None:
                rss, user, system = _sample_process_tree(root)
                peak_rss = max(peak_rss, rss)
                user_cpu = max(user_cpu, user)
                system_cpu = max(system_cpu, system)
                if time.perf_counter() - start >= timeout_seconds:
                    timed_out = True
                    _terminate_process_tree(root, process)
                    break
                time.sleep(sample_interval_seconds)
        except KeyboardInterrupt:
            _terminate_process_tree(root, process)
            raise
        process.wait()
        rss, user, system = _sample_process_tree(root)
        peak_rss = max(peak_rss, rss)
        user_cpu = max(user_cpu, user)
        system_cpu = max(system_cpu, system)
    return ProcessMeasurement(
        exit_code=process.returncode,
        elapsed_seconds=time.perf_counter() - start,
        user_cpu_seconds=user_cpu,
        system_cpu_seconds=system_cpu,
        peak_rss_bytes=peak_rss,
        signal=-process.returncode if process.returncode < 0 else None,
        timed_out=timed_out,
    )


def _terminate_process_tree(
    root: psutil.Process, process: subprocess.Popen[bytes]
) -> None:
    try:
        children = root.children(recursive=True)
    except psutil.Error:
        children = []
    for target in children:
        with suppress(psutil.Error):
            target.terminate()
    process.terminate()
    _gone, alive = psutil.wait_procs(children, timeout=1.0)
    for target in alive:
        with suppress(psutil.Error):
            target.kill()
    try:
        process.wait(timeout=1.0)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def _sample_process_tree(root: psutil.Process) -> tuple[int, float, float]:
    try:
        processes = [root, *root.children(recursive=True)]
    except psutil.Error:
        processes = [root]
    rss = 0
    user = 0.0
    system = 0.0
    for process in processes:
        try:
            rss += process.memory_info().rss
            cpu = process.cpu_times()
            user += cpu.user
            system += cpu.system
        except psutil.Error:
            continue
    return rss, user, system


def _isolated_environment(home: Path, project_root: Path) -> dict[str, str]:
    home.mkdir(parents=True, exist_ok=True)
    python_path = os.pathsep.join(
        item for item in (str(project_root), os.environ.get("PYTHONPATH", "")) if item
    )
    return {
        **os.environ,
        "HOME": str(home),
        "XDG_CONFIG_HOME": str(home / ".config"),
        "XDG_CACHE_HOME": str(home / ".cache"),
        "LC_ALL": "C",
        "LANG": "C",
        "TZ": "UTC",
        "NO_COLOR": "1",
        "CLICOLOR": "0",
        "PYTHONHASHSEED": "0",
        "PYTHONPATH": python_path,
    }


def _result_directory(root: Path, started: datetime) -> Path:
    timestamp = started.strftime("%Y%m%dT%H%M%S.%fZ")
    output = root / timestamp
    output.mkdir(parents=True, exist_ok=False)
    return output


def _filesystem_type(path: Path) -> str:
    try:
        partitions = sorted(
            psutil.disk_partitions(all=True),
            key=lambda item: len(item.mountpoint),
            reverse=True,
        )
        resolved = str(path.resolve())
        for partition in partitions:
            if resolved.startswith(partition.mountpoint):
                return partition.fstype or "unknown"
    except (OSError, psutil.Error):
        pass
    return "unknown"


def _validate_config(config: BenchmarkConfig) -> None:
    if not config.input_path.is_file():
        msg = f"benchmark input is not a file: {config.input_path}"
        raise ValueError(msg)
    if config.rounds < 1 or config.warmups < 0:
        msg = "rounds must be positive and warmups non-negative"
        raise ValueError(msg)
    if not MIN_PIECE_EXPONENT <= config.piece_exponent <= MAX_PIECE_EXPONENT:
        msg = "piece exponent must be between 14 and 26"
        raise ValueError(msg)
    if config.profile != PROFILE:
        msg = f"unsupported profile: {config.profile}"
        raise ValueError(msg)
    if config.cache_state not in {"warm", "cold"}:
        msg = "cache state must be warm or cold"
        raise ValueError(msg)
    if config.cache_state == "cold" and not config.cache_prepare_command:
        msg = "cold cache state requires an explicit preparation command"
        raise ValueError(msg)
    if config.cache_state == "warm" and config.cache_prepare_command:
        msg = "cache preparation command requires cold cache state"
        raise ValueError(msg)
    if config.timeout_seconds <= 0:
        msg = "timeout must be positive"
        raise ValueError(msg)
    if config.sample_interval_seconds <= 0:
        msg = "sample interval must be positive"
        raise ValueError(msg)
