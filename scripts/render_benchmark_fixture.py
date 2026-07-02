#!/usr/bin/env python3
"""Render a deterministic all-status benchmark report for CI."""

import sys
from pathlib import Path

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from benches.btpc_bench.models import (
    BenchmarkResult,
    DatasetFingerprint,
    RunSample,
    ToolResult,
    ToolStatus,
)
from benches.btpc_bench.report import render_summary


def main() -> None:
    dataset = DatasetFingerprint("fixture.bin", 1024, "ab" * 32, 1024, ("cd" * 20,))
    tools = (
        ToolResult(
            "success",
            "1.0",
            ToolStatus.SUCCESS,
            info_hash_v1="12" * 20,
            samples=(
                RunSample(
                    round=0,
                    exit_code=0,
                    elapsed_seconds=1.0,
                    user_cpu_seconds=0.5,
                    system_cpu_seconds=0.1,
                    throughput_mib_s=1.0,
                    peak_rss_bytes=1024,
                    valid=True,
                    cache_state="warm",
                ),
            ),
        ),
        ToolResult("unavailable", "-", ToolStatus.UNAVAILABLE, reason="not found"),
        ToolResult("unsupported", "1.0", ToolStatus.UNSUPPORTED, reason="profile"),
        ToolResult("smoke", "1.0", ToolStatus.SMOKE_FAILED, reason="exit 2"),
        ToolResult("failed", "1.0", ToolStatus.FAILED, reason="timeout"),
        ToolResult("invalid", "1.0", ToolStatus.INVALID, reason="piece mismatch"),
    )
    print(render_summary(BenchmarkResult.example(dataset, tools)), end="")


if __name__ == "__main__":
    main()
