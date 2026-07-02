"""Raw JSON/CSV and human-readable benchmark reporting."""

from __future__ import annotations

import csv
from dataclasses import dataclass
from typing import TYPE_CHECKING

from .models import BenchmarkResult, ToolResult, ToolStatus

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True, slots=True)
class ReportPaths:
    """Generated report artifact paths."""

    json_path: Path
    compatibility_json_path: Path
    csv_path: Path
    summary_path: Path
    markdown_path: Path


def render_summary(result: BenchmarkResult, *, max_width: int | None = None) -> str:
    """Render a fixed-width ASCII leaderboard and actionable failures."""
    ranked, unranked = _rank(result)
    headers: tuple[str, ...] = (
        "Tool",
        "Version",
        "Status",
        "Runs",
        "Median",
        "Mean +/- SD",
        "Min..Max",
        "MiB/s",
        "Peak RSS",
        "Rel.",
        "Hash",
    )
    rows: list[tuple[str, ...]] = []
    fastest = ranked[0].statistics().median if ranked else 0.0  # type: ignore[union-attr]
    for tool in ranked:
        stats = tool.statistics()
        if stats is None:
            continue
        rows.append(
            (
                tool.name,
                tool.version,
                tool.status.value,
                f"{stats.count}/{result.rounds}",
                f"{stats.median:.3f} s",
                f"{stats.mean:.3f} +/- {stats.standard_deviation:.3f}",
                f"{stats.minimum:.3f}..{stats.maximum:.3f}",
                f"{stats.median_throughput_mib_s:.1f}",
                _format_rss(stats.peak_rss_bytes),
                f"{stats.median / fastest:.2f}x" if fastest else "-",
                tool.info_hash_v1[:12],
            )
        )
    for tool in unranked:
        reason = f"{tool.status.value}: {tool.reason}".rstrip(": ")
        rows.append(
            (tool.name, tool.version, reason, "-", "-", "-", "-", "-", "-", "-", "-")
        )
    rows = [tuple(_safe_text(value) for value in row) for row in rows]
    headers = tuple(_safe_text(value) for value in headers)
    if max_width is not None:
        rows = _truncate_rows(headers, rows, max_width=max_width)
    widths = [
        max(len(headers[index]), *(len(row[index]) for row in rows))
        for index in range(len(headers))
    ]
    separator = "+" + "+".join("-" * (width + 2) for width in widths) + "+"
    lines = [separator, _ascii_row(headers, widths), separator]
    lines.extend(_ascii_row(row, widths) for row in rows)
    lines.append(separator)
    lines.extend(
        [
            f"dataset: {result.dataset.path} ({result.dataset.size_bytes} bytes)",
            f"profile: {result.profile}; cache: {result.cache_state}; "
            f"seed: {result.seed}",
        ]
    )
    return "\n".join(lines) + "\n"


def render_markdown(result: BenchmarkResult) -> str:
    """Render a deterministic Markdown summary with full statistics."""
    ranked, unranked = _rank(result)
    lines = [
        "# BTPC benchmark summary",
        "",
        f"- Started: `{result.started_at}`",
        f"- Dataset: `{result.dataset.path}` ({result.dataset.size_bytes} bytes)",
        f"- Profile: `{result.profile}`; cache: `{result.cache_state}`",
        f"- Seed: `{result.seed}`; rounds: `{result.rounds}`; "
        f"warmups: `{result.warmups}`",
        "",
        "| Tool | Version | Runs | Median | Mean | SD | MAD | CV | MiB/s | "
        "Peak RSS | Relative | Info hash |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|",
    ]
    fastest = ranked[0].statistics().median if ranked else 0.0  # type: ignore[union-attr]
    for tool in ranked:
        stats = tool.statistics()
        if stats is None:
            continue
        lines.append(
            "| "
            + " | ".join(
                (
                    tool.name,
                    tool.version,
                    f"{stats.count}/{result.rounds}",
                    f"{stats.median:.6f}s",
                    f"{stats.mean:.6f}s",
                    f"{stats.standard_deviation:.6f}s",
                    f"{stats.median_absolute_deviation:.6f}s",
                    f"{stats.coefficient_of_variation:.2%}",
                    f"{stats.median_throughput_mib_s:.2f}",
                    _format_rss(stats.peak_rss_bytes),
                    f"{stats.median / fastest:.2f}x" if fastest else "-",
                    tool.info_hash_v1,
                )
            )
            + " |"
        )
    if unranked:
        lines.extend(["", "## Unranked tools", ""])
        lines.extend(
            f"- `{tool.name}` ({tool.version}): **{tool.status.value}** — {tool.reason}"
            for tool in unranked
        )
    return "\n".join(lines) + "\n"


def write_reports(result: BenchmarkResult, output: Path) -> ReportPaths:
    """Write raw result, sample CSV, ASCII, and Markdown summaries."""
    output.mkdir(parents=True, exist_ok=True)
    paths = ReportPaths(
        json_path=output / "results.json",
        compatibility_json_path=output / "result.json",
        csv_path=output / "samples.csv",
        summary_path=output / "summary.txt",
        markdown_path=output / "summary.md",
    )
    paths.json_path.write_text(result.to_json())
    paths.compatibility_json_path.write_text(result.to_json())
    with paths.csv_path.open("w", newline="") as stream:
        writer = csv.writer(stream)
        writer.writerow(
            (
                "tool",
                "version",
                "status",
                "round",
                "exit_code",
                "signal",
                "elapsed_seconds",
                "user_cpu_seconds",
                "system_cpu_seconds",
                "throughput_mib_s",
                "peak_rss_bytes",
                "output_size_bytes",
                "timed_out",
                "valid",
                "cache_state",
                "reason",
            )
        )
        for tool in result.tools:
            for sample in tool.samples:
                writer.writerow(
                    (
                        tool.name,
                        tool.version,
                        tool.status.value,
                        sample.round,
                        sample.exit_code,
                        sample.signal if sample.signal is not None else "",
                        sample.elapsed_seconds,
                        sample.user_cpu_seconds,
                        sample.system_cpu_seconds,
                        sample.throughput_mib_s,
                        sample.peak_rss_bytes or "",
                        sample.output_size_bytes or "",
                        sample.timed_out,
                        sample.valid,
                        sample.cache_state,
                        sample.reason,
                    )
                )
    paths.summary_path.write_text(render_summary(result))
    paths.markdown_path.write_text(render_markdown(result))
    return paths


def _rank(result: BenchmarkResult) -> tuple[list[ToolResult], list[ToolResult]]:
    ranked = [
        tool
        for tool in result.tools
        if tool.status is ToolStatus.SUCCESS and tool.statistics() is not None
    ]
    ranked.sort(
        key=lambda tool: (tool.statistics().median, tool.name)  # type: ignore[union-attr]
    )
    unranked = sorted(
        (tool for tool in result.tools if tool not in ranked),
        key=lambda tool: tool.name,
    )
    return ranked, unranked


def _ascii_row(values: tuple[str, ...], widths: list[int]) -> str:
    return (
        "| "
        + " | ".join(value.ljust(widths[index]) for index, value in enumerate(values))
        + " |"
    )


def _format_rss(value: int | None) -> str:
    return f"{value / (1024 * 1024):.1f} MiB" if value is not None else "n/a"


def _safe_text(value: str) -> str:
    return value.encode("utf-8", errors="replace").decode("utf-8").replace("\n", " ")


def _truncate_rows(
    headers: tuple[str, ...], rows: list[tuple[str, ...]], *, max_width: int
) -> list[tuple[str, ...]]:
    minimum = sum(len(header) for header in headers) + 3 * len(headers) + 1
    max_width = max(max_width, minimum)
    widths = [
        max(len(headers[index]), *(len(row[index]) for row in rows))
        for index in range(len(headers))
    ]
    overflow = sum(widths) + 3 * len(widths) + 1 - max_width
    shrinkable = [2, 1, 0]
    for index in shrinkable:
        while overflow > 0 and widths[index] > len(headers[index]):
            widths[index] -= 1
            overflow -= 1
    return [
        tuple(
            value
            if len(value) <= widths[index]
            else value[: max(0, widths[index] - 1)] + "…"
            for index, value in enumerate(row)
        )
        for row in rows
    ]
