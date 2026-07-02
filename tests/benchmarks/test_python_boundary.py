from __future__ import annotations

import json
from dataclasses import replace
from typing import TYPE_CHECKING

import btpc
import pytest

from benches.btpc_bench.python_boundary import compare, render, run

if TYPE_CHECKING:
    from pathlib import Path


def _fixture(tmp_path: Path) -> tuple[Path, Path]:
    payload = tmp_path / "payload"
    payload.write_bytes(b"python boundary benchmark")
    torrent = tmp_path / "payload.torrent"
    torrent.write_bytes(btpc.create_bytes(payload).bytes)
    return torrent, payload


def test_python_boundary_result_is_validated_and_deterministic(tmp_path: Path) -> None:
    torrent, payload = _fixture(tmp_path)
    result = run(torrent, payload=payload, warmups=0, repetitions=2)
    assert result.schema == 1
    assert [sample.workflow for sample in result.samples] == sorted(
        [sample.workflow for sample in result.samples],
        key=[sample.workflow for sample in result.samples].index,
    )
    parsed = json.loads(result.to_json())
    assert parsed["input_bytes"] > 0
    table = render(result)
    assert table.startswith("workflow")
    assert "parse_memoryview" in table


def test_python_boundary_comparison_enforces_budgets(tmp_path: Path) -> None:
    baseline = run(_fixture(tmp_path)[0], warmups=0, repetitions=1)
    first = baseline.samples[0]
    candidate = replace(
        baseline,
        samples=(
            replace(first, median_ns=max(first.median_ns * 2, 2)),
            *baseline.samples[1:],
        ),
    )
    assert compare(baseline, candidate, budget_ratio=1.25)[0].startswith(first.workflow)
    with pytest.raises(ValueError, match="incompatible"):
        compare(baseline, replace(candidate, input_bytes=candidate.input_bytes + 1))


def test_python_boundary_rejects_invalid_run_configuration(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="repetitions"):
        run(_fixture(tmp_path)[0], repetitions=0)
