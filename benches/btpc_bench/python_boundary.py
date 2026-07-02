"""Deterministic Python/Rust boundary benchmark workflows."""

from __future__ import annotations

import argparse
import json
import platform
import statistics
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import TYPE_CHECKING

import btpc

if TYPE_CHECKING:
    from collections.abc import Callable

DEFAULT_REPETITIONS = 20
DEFAULT_WARMUPS = 3
DEFAULT_BUDGET_RATIO = 1.25


@dataclass(frozen=True, slots=True)
class BoundarySample:
    """One measured public Python workflow."""

    workflow: str
    median_ns: int
    minimum_ns: int
    maximum_ns: int
    repetitions: int


@dataclass(frozen=True, slots=True)
class BoundaryResult:
    """Versioned Python boundary benchmark result."""

    schema: int
    python: str
    btpc: str
    input_bytes: int
    warmups: int
    samples: tuple[BoundarySample, ...]

    def to_json(self) -> str:
        """Return stable machine-readable output."""
        return json.dumps(asdict(self), indent=2, sort_keys=True) + "\n"


def _measure(
    name: str, operation: Callable[[], object], warmups: int, repetitions: int
) -> BoundarySample:
    for _ in range(warmups):
        operation()
    durations = []
    for _ in range(repetitions):
        started = time.perf_counter_ns()
        operation()
        durations.append(time.perf_counter_ns() - started)
    return BoundarySample(
        workflow=name,
        median_ns=int(statistics.median(durations)),
        minimum_ns=min(durations),
        maximum_ns=max(durations),
        repetitions=repetitions,
    )


def run(
    path: Path,
    *,
    payload: Path | None = None,
    warmups: int = DEFAULT_WARMUPS,
    repetitions: int = DEFAULT_REPETITIONS,
) -> BoundaryResult:
    """Benchmark validated public Python workflows for one metainfo fixture."""
    if warmups < 0 or repetitions < 1:
        raise ValueError("warmups must be nonnegative and repetitions must be positive")
    data = path.read_bytes()
    metainfo = btpc.Metainfo.from_bytes(data)
    expected_name = metainfo.name

    def parse(value: object) -> btpc.Metainfo:
        parsed = btpc.Metainfo.from_bytes(value)
        if parsed.name != expected_name or parsed.original_bytes != data:
            raise ValueError("parse workflow produced a non-equivalent result")
        return parsed

    workflows: list[tuple[str, Callable[[], object]]] = [
        ("parse_path", lambda: btpc.Metainfo.read(path)),
        ("parse_bytes", lambda: parse(data)),
        ("parse_bytearray", lambda: parse(bytearray(data))),
        ("parse_memoryview", lambda: parse(memoryview(data))),
        ("property_name_first", lambda: btpc.Metainfo.from_bytes(data).name),
        ("property_name", lambda: metainfo.name),
        ("property_files_first", lambda: btpc.Metainfo.from_bytes(data).files),
        ("property_files", lambda: metainfo.files),
        ("magnet", metainfo.magnet),
        ("edit_noop", metainfo.edit),
        ("edit_top_level", lambda: metainfo.edit(comment="benchmark")),
        ("equality", lambda: metainfo == btpc.Metainfo.from_bytes(data)),
        ("create_setup", btpc.CreateOptions),
    ]
    if payload is not None:
        if not metainfo.verify(payload).is_valid:
            raise ValueError("payload does not match the metainfo fixture")
        workflows.extend(
            [
                ("verify", lambda: metainfo.verify(payload)),
                ("create", lambda: btpc.create_bytes(payload)),
            ]
        )
    samples = tuple(
        _measure(name, operation, warmups, repetitions) for name, operation in workflows
    )
    return BoundaryResult(
        schema=1,
        python=platform.python_version(),
        btpc=btpc.__version__,
        input_bytes=len(data),
        warmups=warmups,
        samples=samples,
    )


def compare(
    baseline: BoundaryResult,
    candidate: BoundaryResult,
    *,
    budget_ratio: float = DEFAULT_BUDGET_RATIO,
) -> list[str]:
    """Return stable regression messages for workflows over budget."""
    if (
        baseline.schema != candidate.schema
        or baseline.input_bytes != candidate.input_bytes
    ):
        raise ValueError("incompatible Python boundary benchmark results")
    baseline_by_name = {sample.workflow: sample for sample in baseline.samples}
    messages = []
    for sample in candidate.samples:
        reference = baseline_by_name.get(sample.workflow)
        if reference is None:
            raise ValueError(f"baseline missing workflow: {sample.workflow}")
        ratio = sample.median_ns / max(reference.median_ns, 1)
        if ratio > budget_ratio:
            messages.append(
                f"{sample.workflow}: {ratio:.3f}x exceeds {budget_ratio:.3f}x budget"
            )
    return messages


def render(result: BoundaryResult) -> str:
    """Render deterministic ASCII columns."""
    lines = [
        "workflow          median_us  min_us  max_us  runs",
        "----------------  ---------  ------  ------  ----",
    ]
    lines.extend(
        (
            f"{sample.workflow:<16}  {sample.median_ns / 1000:9.3f}  "
            f"{sample.minimum_ns / 1000:6.3f}  "
            f"{sample.maximum_ns / 1000:6.3f}  {sample.repetitions:4d}"
        )
        for sample in result.samples
    )
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    """Run and persist the Python boundary profile."""
    parser = argparse.ArgumentParser()
    parser.add_argument("metainfo", type=Path)
    parser.add_argument("--payload", type=Path)
    parser.add_argument("--warmups", type=int, default=DEFAULT_WARMUPS)
    parser.add_argument("--repetitions", type=int, default=DEFAULT_REPETITIONS)
    parser.add_argument("--json", type=Path)
    args = parser.parse_args(argv)
    result = run(
        args.metainfo,
        payload=args.payload,
        warmups=args.warmups,
        repetitions=args.repetitions,
    )
    if args.json is not None:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(result.to_json())
    sys.stdout.write(render(result))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
