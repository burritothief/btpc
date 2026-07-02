"""Explicit command adapters for comparable v1 creation tools."""

from __future__ import annotations

import os
import shutil
import subprocess
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

from .models import ToolStatus

CommandBuilder = Callable[[Path, Path, Path, str, int], list[str]]
WorkerCommandBuilder = Callable[[list[str], int], list[str]]


@dataclass(frozen=True, slots=True)
class Availability:
    """Resolved adapter executable and version state."""

    status: ToolStatus
    executable: Path | None
    version: str
    reason: str = ""


@dataclass(frozen=True, slots=True)
class ToolAdapter:
    """Versioned competitor command contract."""

    name: str
    executables: tuple[str, ...]
    version_args: tuple[str, ...]
    build_command: CommandBuilder
    supported_profiles: tuple[str, ...] = ("v1-single-file",)
    piece_size_semantics: str = "bytes"
    quiet: bool = True
    supports_default_workers: bool = True
    with_workers: WorkerCommandBuilder | None = None
    executable_override: Path | None = None

    def discover(self, *, path_env: str | None = None) -> Availability:
        """Locate the executable and capture its exact reported version."""
        executable = self.executable_override
        if executable is None or not executable.is_file():
            executable = next(
                (
                    Path(found)
                    for candidate in self.executables
                    if (found := shutil.which(candidate, path=path_env)) is not None
                ),
                None,
            )
        if executable is None:
            names = ", ".join(self.executables)
            return Availability(
                ToolStatus.UNAVAILABLE,
                None,
                "unavailable",
                f"executable not found ({names})",
            )
        try:
            completed = subprocess.run(
                [str(executable), *self.version_args],
                check=False,
                capture_output=True,
                text=True,
                timeout=10,
                env={**os.environ, "NO_COLOR": "1"},
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            return Availability(
                ToolStatus.UNSUPPORTED,
                executable,
                "unknown",
                f"version command failed: {error}",
            )
        version = (completed.stdout or completed.stderr).strip().splitlines()
        if completed.returncode != 0 or not version:
            return Availability(
                ToolStatus.UNSUPPORTED,
                executable,
                "unknown",
                f"version command exited {completed.returncode}",
            )
        return Availability(ToolStatus.READY, executable, version[0])


def adapter_registry(*, project_root: Path, python: str) -> dict[str, ToolAdapter]:
    """Return every required adapter with normalized v1 options."""
    native_override = _native_binary(project_root)
    python_path = Path(python)
    return {
        "btpc-native": ToolAdapter(
            "btpc-native",
            ("btpc",),
            ("--version",),
            _btpc_native,
            executable_override=native_override,
        ),
        "btpc-python": ToolAdapter(
            "btpc-python",
            (python,),
            ("--version",),
            _btpc_python,
            executable_override=python_path,
        ),
        "mkbrr": ToolAdapter(
            "mkbrr",
            ("mkbrr",),
            ("version",),
            _mkbrr,
            piece_size_semantics="binary exponent",
            with_workers=lambda command, workers: [
                *command,
                "--threads",
                str(workers),
            ],
        ),
        "mktorrent": ToolAdapter(
            "mktorrent",
            ("mktorrent",),
            ("-h",),
            _mktorrent,
            piece_size_semantics="binary exponent",
            with_workers=lambda command, workers: [
                *command[:-1],
                "--threads",
                str(workers),
                command[-1],
            ],
        ),
        "torf-cli": ToolAdapter(
            "torf-cli",
            ("torf", "torf-cli"),
            ("--version",),
            _torf,
            piece_size_semantics="maximum MiB; exact for canonical ISO",
            with_workers=lambda command, workers: [
                *command,
                "--threads",
                str(workers),
            ],
        ),
        "torrenttools": ToolAdapter(
            "torrenttools", ("torrenttools",), ("--version",), _torrenttools
        ),
    }


def _native_binary(project_root: Path) -> Path | None:
    override = os.environ.get("BTPC_BIN")
    if override:
        return Path(override)
    for relative in ("target/release/btpc", "target/debug/btpc"):
        candidate = project_root / relative
        if candidate.is_file():
            return candidate
    return None


def _btpc_native(
    executable: Path, payload: Path, output: Path, tracker: str, exponent: int
) -> list[str]:
    return [
        str(executable),
        "create",
        str(payload),
        "--output",
        str(output),
        "--mode",
        "v1",
        "--piece-length",
        str(1 << exponent),
        "--tracker",
        tracker,
        "--private",
        "--quiet",
    ]


def _btpc_python(
    executable: Path, payload: Path, output: Path, tracker: str, exponent: int
) -> list[str]:
    return [
        str(executable),
        "-m",
        "benches.btpc_bench.python_adapter",
        str(payload),
        str(output),
        tracker,
        str(1 << exponent),
    ]


def _mkbrr(
    executable: Path, payload: Path, output: Path, tracker: str, exponent: int
) -> list[str]:
    return [
        str(executable),
        "create",
        str(payload),
        "--tracker",
        tracker,
        "--output",
        str(output),
        "--piece-length",
        str(exponent),
        "--private=true",
        "--no-date",
        "--no-creator",
    ]


def _mktorrent(
    executable: Path, payload: Path, output: Path, tracker: str, exponent: int
) -> list[str]:
    return [
        str(executable),
        "-a",
        tracker,
        "-l",
        str(exponent),
        "-d",
        "-p",
        "-o",
        str(output),
        str(payload),
    ]


def _torf(
    executable: Path, payload: Path, output: Path, tracker: str, exponent: int
) -> list[str]:
    piece_mib = (1 << exponent) / (1024 * 1024)
    return [
        str(executable),
        str(payload),
        "--out",
        str(output),
        "--tracker",
        tracker,
        "--private",
        "--max-piece-size",
        f"{piece_mib:g}",
        "--nodate",
        "--nocreator",
        "--yes",
        "--noconfig",
        "--nomagnet",
    ]


def _torrenttools(
    executable: Path, payload: Path, output: Path, tracker: str, exponent: int
) -> list[str]:
    return [
        str(executable),
        "create",
        "--protocol",
        "v1",
        "--output",
        str(output),
        "--announce",
        tracker,
        "--private",
        "on",
        "--piece-size",
        str(1 << exponent),
        "--no-creation-date",
        str(payload),
    ]
