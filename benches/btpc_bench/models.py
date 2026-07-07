"""Serializable benchmark result models."""

from __future__ import annotations

import dataclasses
import json
import statistics
from dataclasses import dataclass
from enum import StrEnum
from typing import Any

from .paths import filesystem_path_document, filesystem_path_from_document, path_name


class ToolStatus(StrEnum):
    """Lifecycle state for a benchmark adapter."""

    READY = "ready"
    SUCCESS = "success"
    UNAVAILABLE = "unavailable"
    UNSUPPORTED = "unsupported"
    SMOKE_FAILED = "smoke_failed"
    FAILED = "failed"
    INVALID = "invalid"


@dataclass(frozen=True, slots=True)
class DatasetFingerprint:
    """Streaming checksum oracle for one benchmark payload."""

    path: str
    size_bytes: int
    sha256: str
    piece_length: int
    piece_sha1: tuple[str, ...]
    name: str = ""
    mtime_ns: int | None = None


@dataclass(frozen=True, slots=True)
class RunSample:
    """One measured process execution."""

    round: int
    exit_code: int
    elapsed_seconds: float
    user_cpu_seconds: float
    system_cpu_seconds: float
    throughput_mib_s: float
    peak_rss_bytes: int | None
    valid: bool
    cache_state: str
    log_prefix: str = ""
    reason: str = ""
    signal: int | None = None
    output_size_bytes: int | None = None
    timed_out: bool = False


@dataclass(frozen=True, slots=True)
class SampleStatistics:
    """Descriptive statistics for valid measured samples."""

    count: int
    median: float
    mean: float
    standard_deviation: float
    minimum: float
    maximum: float
    median_absolute_deviation: float
    coefficient_of_variation: float
    median_throughput_mib_s: float
    peak_rss_bytes: int | None


@dataclass(frozen=True, slots=True)
class ToolResult:
    """Availability, version, samples, and validation for one tool."""

    name: str
    version: str
    status: ToolStatus
    reason: str = ""
    executable: str = ""
    command_template: tuple[str, ...] = ()
    info_hash_v1: str = ""
    samples: tuple[RunSample, ...] = ()

    def statistics(self) -> SampleStatistics | None:
        """Compute statistics from valid successful measured samples."""
        samples = [
            sample for sample in self.samples if sample.exit_code == 0 and sample.valid
        ]
        if not samples:
            return None
        elapsed = [sample.elapsed_seconds for sample in samples]
        median = statistics.median(elapsed)
        mean = statistics.fmean(elapsed)
        deviation = statistics.stdev(elapsed) if len(elapsed) > 1 else 0.0
        absolute = [abs(value - median) for value in elapsed]
        rss = [sample.peak_rss_bytes for sample in samples if sample.peak_rss_bytes]
        return SampleStatistics(
            count=len(samples),
            median=median,
            mean=mean,
            standard_deviation=deviation,
            minimum=min(elapsed),
            maximum=max(elapsed),
            median_absolute_deviation=statistics.median(absolute),
            coefficient_of_variation=deviation / mean if mean else 0.0,
            median_throughput_mib_s=statistics.median(
                sample.throughput_mib_s for sample in samples
            ),
            peak_rss_bytes=max(rss) if rss else None,
        )


@dataclass(frozen=True, slots=True)
class BenchmarkResult:
    """Complete reproducible benchmark result document."""

    schema_version: int
    harness_version: str
    started_at: str
    seed: int
    profile: str
    preset: str
    cache_state: str
    warmups: int
    rounds: int
    tracker: str
    piece_exponent: int
    dataset: DatasetFingerprint
    machine: dict[str, Any]
    run_order: tuple[str, ...]
    tools: tuple[ToolResult, ...]
    warnings: tuple[str, ...] = ()

    @classmethod
    def example(
        cls,
        dataset: DatasetFingerprint,
        tools: tuple[ToolResult, ...],
    ) -> BenchmarkResult:
        """Build a compact deterministic fixture result."""
        return cls(
            schema_version=2,
            harness_version="0.1.0",
            started_at="2026-01-01T00:00:00Z",
            seed=1,
            profile="v1-single-file",
            preset="quick",
            cache_state="warm",
            warmups=1,
            rounds=2,
            tracker="https://tracker.invalid/announce",
            piece_exponent=22,
            dataset=dataset,
            machine={},
            run_order=tuple(tool.name for tool in tools),
            tools=tools,
        )

    def to_json(self) -> str:
        """Serialize with stable indentation and key order."""
        return json.dumps(_encode(self), indent=2, sort_keys=True) + "\n"

    @classmethod
    def from_json(cls, value: str) -> BenchmarkResult:
        """Deserialize a result document."""
        raw = json.loads(value)
        schema_version = raw.get("schema_version")
        if schema_version not in {1, 2}:
            msg = f"unsupported benchmark schema version: {schema_version!r}"
            raise ValueError(msg)
        dataset = dict(raw["dataset"])
        dataset_path = filesystem_path_from_document(dataset.pop("path"))
        dataset.pop("path_display", None)
        return cls(
            schema_version=schema_version,
            harness_version=raw["harness_version"],
            started_at=raw["started_at"],
            seed=raw["seed"],
            profile=raw["profile"],
            preset=raw["preset"],
            cache_state=raw["cache_state"],
            warmups=raw["warmups"],
            rounds=raw["rounds"],
            tracker=raw["tracker"],
            piece_exponent=raw["piece_exponent"],
            dataset=DatasetFingerprint(
                **{
                    **dataset,
                    "path": dataset_path,
                    "name": dataset.get("name", path_name(dataset_path)),
                    "mtime_ns": dataset.get("mtime_ns"),
                    "piece_sha1": tuple(dataset["piece_sha1"]),
                }
            ),
            machine=raw["machine"],
            run_order=tuple(raw["run_order"]),
            tools=tuple(
                ToolResult(
                    name=tool["name"],
                    version=tool["version"],
                    status=ToolStatus(tool["status"]),
                    reason=tool["reason"],
                    executable=tool["executable"],
                    command_template=tuple(tool["command_template"]),
                    info_hash_v1=tool["info_hash_v1"],
                    samples=tuple(
                        RunSample(
                            **{
                                "signal": None,
                                "output_size_bytes": None,
                                "timed_out": False,
                                **sample,
                            }
                        )
                        for sample in tool["samples"]
                    ),
                )
                for tool in raw["tools"]
            ),
            warnings=tuple(raw["warnings"]),
        )


def _encode(value: object) -> object:
    if isinstance(value, DatasetFingerprint):
        return {
            "path": filesystem_path_document(value.path),
            "path_display": filesystem_path_document(value.path)["display"],
            "size_bytes": value.size_bytes,
            "sha256": value.sha256,
            "piece_length": value.piece_length,
            "piece_sha1": _encode(value.piece_sha1),
            "name": value.name,
            "mtime_ns": value.mtime_ns,
        }
    if dataclasses.is_dataclass(value) and not isinstance(value, type):
        return {
            field.name: _encode(getattr(value, field.name))
            for field in dataclasses.fields(value)
        }
    if isinstance(value, StrEnum):
        return value.value
    if isinstance(value, tuple):
        return [_encode(item) for item in value]
    if isinstance(value, dict):
        return {key: _encode(item) for key, item in value.items()}
    return value


def dataset_fingerprint_document(value: DatasetFingerprint) -> dict[str, object]:
    """Return the v2 JSON representation for one dataset fingerprint."""
    encoded = _encode(value)
    if not isinstance(encoded, dict):
        msg = "dataset fingerprint did not encode as an object"
        raise TypeError(msg)
    return encoded
