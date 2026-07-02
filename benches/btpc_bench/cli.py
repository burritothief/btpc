"""Command-line interface for reproducible BTPC competitor benchmarks."""

from __future__ import annotations

import argparse
import json
import sys
import time
from dataclasses import asdict
from pathlib import Path

from .adapters import adapter_registry
from .dataset import fingerprint_payload, generate_dataset, validate_canonical_iso
from .models import BenchmarkResult, ToolStatus
from .report import render_summary
from .runner import BenchmarkConfig, run_benchmark

DEFAULT_TOOLS = (
    "btpc-native",
    "btpc-python",
    "mkbrr",
    "mktorrent",
    "torf-cli",
    "torrenttools",
)


def main(arguments: list[str] | None = None) -> int:
    """Run a benchmark helper subcommand."""
    parser = _parser()
    options = parser.parse_args(arguments)
    try:
        if options.command == "generate":
            generated = generate_dataset(
                options.output,
                seed=options.seed,
                size_bytes=options.size_bytes,
                piece_length=1 << options.piece_exponent,
                name=options.name,
            )
            print(generated.manifest_path)
            return 0
        if options.command == "render":
            result = BenchmarkResult.from_json(options.result.read_text())
            print(render_summary(result), end="")
            return 0
        if options.command == "preflight":
            started = time.perf_counter()
            fingerprint = fingerprint_payload(
                options.input, piece_length=1 << options.piece_exponent
            )
            validate_canonical_iso(fingerprint)
            document = {
                "schema_version": 1,
                "elapsed_seconds": time.perf_counter() - started,
                "dataset": asdict(fingerprint),
            }
            options.output.parent.mkdir(parents=True, exist_ok=True)
            options.output.write_text(
                json.dumps(document, indent=2, sort_keys=True) + "\n"
            )
            print(options.output)
            return 0
        if options.command == "compare":
            baseline = BenchmarkResult.from_json(options.baseline.read_text())
            candidate = BenchmarkResult.from_json(options.candidate.read_text())
            print(_compare(baseline, candidate), end="")
            return 0
        if options.command == "run":
            warmups, rounds = _preset(options.preset, options.warmups, options.rounds)
            selected = (
                DEFAULT_TOOLS if options.tools == ["all"] else tuple(options.tools)
            )
            config = BenchmarkConfig(
                input_path=options.input,
                output_root=options.output_root,
                tools=tuple(selected),
                warmups=warmups,
                rounds=rounds,
                seed=options.seed,
                tracker=options.tracker,
                piece_exponent=options.piece_exponent,
                profile=options.profile,
                preset=options.preset,
                cache_state=options.cache_state,
                require_tools=options.require_tools,
                timeout_seconds=options.timeout,
                sample_interval_seconds=options.sample_interval_ms / 1000,
                cache_prepare_command=tuple(options.cache_prepare_command or ()),
            )
            registry = adapter_registry(
                project_root=Path(__file__).parents[2], python=sys.executable
            )
            result, output = run_benchmark(config, registry=registry)
            print(f"results: {output}")
            return (
                0
                if all(
                    tool.status in {ToolStatus.SUCCESS, ToolStatus.UNAVAILABLE}
                    for tool in result.tools
                )
                else 1
            )
    except (OSError, ValueError, RuntimeError) as error:
        parser.error(str(error))
    return 2


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="btpc-bench")
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser(
        "generate", help="generate a deterministic payload"
    )
    generate.add_argument("output", type=Path)
    generate.add_argument("--seed", type=int, default=20260701)
    generate.add_argument("--size-bytes", type=int, default=64 * 1024 * 1024)
    generate.add_argument("--piece-exponent", type=int, default=22)
    generate.add_argument("--name", default="payload.bin")

    run = subparsers.add_parser("run", help="run and validate competitor tools")
    run.add_argument("input", type=Path)
    run.add_argument("--output-root", type=Path, default=Path("benchmark-results"))
    run.add_argument(
        "--tools", nargs="+", default=["all"], choices=["all", *DEFAULT_TOOLS]
    )
    run.add_argument("--preset", choices=("quick", "standard"), default="standard")
    run.add_argument("--warmups", type=int)
    run.add_argument("--rounds", type=int)
    run.add_argument("--seed", type=int, default=20260701)
    run.add_argument("--tracker", default="https://tracker.invalid/announce")
    run.add_argument("--piece-exponent", type=int, default=22)
    run.add_argument("--profile", default="v1-single-file")
    run.add_argument("--cache-state", choices=("warm", "cold"), default="warm")
    run.add_argument("--require-tools", action="store_true")
    run.add_argument("--timeout", type=float, default=600.0)
    run.add_argument("--sample-interval-ms", type=float, default=10.0)
    run.add_argument("--cache-prepare-command", nargs="+")

    preflight = subparsers.add_parser(
        "preflight", help="fingerprint and verify input without running tools"
    )
    preflight.add_argument("input", type=Path)
    preflight.add_argument("--output", type=Path, default=Path("preflight.json"))
    preflight.add_argument("--piece-exponent", type=int, default=22)

    render = subparsers.add_parser("render", help="render a saved JSON result")
    render.add_argument("result", type=Path)

    compare = subparsers.add_parser("compare", help="compare two saved JSON results")
    compare.add_argument("baseline", type=Path)
    compare.add_argument("candidate", type=Path)
    return parser


def _preset(preset: str, warmups: int | None, rounds: int | None) -> tuple[int, int]:
    defaults = (1, 3) if preset == "quick" else (2, 10)
    return (
        warmups if warmups is not None else defaults[0],
        rounds if rounds is not None else defaults[1],
    )


def _compare(baseline: BenchmarkResult, candidate: BenchmarkResult) -> str:
    baseline_tools = {tool.name: tool for tool in baseline.tools}
    candidate_tools = {tool.name: tool for tool in candidate.tools}
    lines = [
        "BTPC benchmark comparison "
        "(descriptive only; no statistical significance claimed)",
        "",
        "Tool              Baseline      Candidate     Change",
        "----------------  ------------  ------------  ----------------",
    ]
    for name in sorted(baseline_tools.keys() | candidate_tools.keys()):
        old = baseline_tools.get(name)
        new = candidate_tools.get(name)
        old_stats = old.statistics() if old else None
        new_stats = new.statistics() if new else None
        if old_stats is None or new_stats is None:
            change = "not comparable"
            old_text = old.status.value if old else "missing"
            new_text = new.status.value if new else "missing"
        else:
            ratio = old_stats.median / new_stats.median if new_stats.median else 0.0
            absolute = new_stats.median - old_stats.median
            percentage = absolute / old_stats.median * 100 if old_stats.median else 0.0
            if ratio >= 1:
                change = f"{ratio:.2f}x faster; {absolute:+.6f}s ({percentage:+.2f}%)"
            else:
                change = (
                    f"{1 / ratio:.2f}x slower; {absolute:+.6f}s ({percentage:+.2f}%)"
                    if ratio
                    else "not comparable"
                )
            old_text = (
                f"{old_stats.median:.6f}s n={old_stats.count} "
                f"CV={old_stats.coefficient_of_variation:.1%}"
            )
            new_text = (
                f"{new_stats.median:.6f}s n={new_stats.count} "
                f"CV={new_stats.coefficient_of_variation:.1%}"
            )
        lines.append(f"{name:<16}  {old_text:<12}  {new_text:<12}  {change}")
    if baseline.dataset.sha256 != candidate.dataset.sha256:
        lines.extend(["", "WARNING: dataset SHA-256 fingerprints differ."])
    if (
        baseline.profile != candidate.profile
        or baseline.cache_state != candidate.cache_state
    ):
        lines.extend(["", "WARNING: profile or cache-state labels differ."])
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    raise SystemExit(main())
