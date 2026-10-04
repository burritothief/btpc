from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]


def test_pyrefly_primary_gate_and_negative_fixture() -> None:
    subprocess.run(  # noqa: S603
        [ROOT / "scripts" / "check_python_types.sh"], cwd=ROOT, check=True
    )


def test_native_stub_parity_and_failure_detection(tmp_path: Path) -> None:
    script = ROOT / "scripts" / "check_native_stub.py"
    subprocess.run(  # noqa: S603
        [sys.executable, script], cwd=ROOT, check=True
    )
    broken = tmp_path / "_native.pyi"
    text = (ROOT / "python" / "btpc" / "_native.pyi").read_text()
    broken.write_text(text.replace("def inspect_bytes(", "def stale_inspect_bytes("))
    result = subprocess.run(  # noqa: S603
        [sys.executable, script, "--stub", broken],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode != 0
    assert "missing runtime exports" in result.stderr


# Spec: RELEASE-PY-TYPING-001
@pytest.mark.parametrize(
    ("original", "replacement"),
    [
        ("def magnet(", "def stale_magnet("),
        ("display_name: bool", "obsolete_parameter: bool"),
        ("display_name: bool,", "display_name: bool, /,"),
    ],
)
def test_native_stub_checks_class_members_and_parameters(
    tmp_path: Path,
    original: str,
    replacement: str,
) -> None:
    broken = tmp_path / "_native.pyi"
    broken.write_text(
        (ROOT / "python/btpc/_native.pyi").read_text().replace(original, replacement)
    )
    result = subprocess.run(  # noqa: S603
        [sys.executable, ROOT / "scripts/check_native_stub.py", "--stub", broken],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode != 0
    assert "_NativeMetainfo" in result.stderr
