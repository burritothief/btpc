"""Tests for the specification registry validator."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "check_specs", ROOT / "scripts/check_specs.py"
)
assert SPEC is not None
assert SPEC.loader is not None
CHECK_SPECS = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = CHECK_SPECS
SPEC.loader.exec_module(CHECK_SPECS)


def test_repository_spec_registry_is_valid() -> None:
    """The checked-in registry should satisfy every traceability invariant."""
    CHECK_SPECS.validate()


@pytest.mark.parametrize(
    ("text", "message"),
    [
        ("# Missing front matter", "missing opening"),
        ("---\nspec_id: TEST\n", "missing closing"),
        ("---\nspec_id: TEST\nspec_id: AGAIN\n---\n", "duplicate"),
    ],
)
def test_front_matter_failures_are_explicit(
    tmp_path: Path, text: str, message: str
) -> None:
    """Malformed metadata should fail with actionable diagnostics."""
    path = tmp_path / "broken.md"
    path.write_text(text)
    with pytest.raises(ValueError, match=message):
        CHECK_SPECS.parse_front_matter(path, text)


def test_requirement_parser_rejects_missing_fields(tmp_path: Path) -> None:
    """Requirements without traceability fields cannot enter the registry."""
    path = tmp_path / "broken.md"
    body = "### TEST-RULE-001 — Broken rule\n\n- **Status:** Implemented\n"
    with pytest.raises(ValueError, match="missing fields"):
        CHECK_SPECS.parse_requirements(path, body, "TEST")
