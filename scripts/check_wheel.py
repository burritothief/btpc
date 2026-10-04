"""Check runtime behavior and typing in a clean, installed wheel environment."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def check(wheel: Path, directory: Path) -> None:
    directory.mkdir(parents=True, exist_ok=False)
    environment = os.environ.copy()
    environment.pop("PYTHONPATH", None)
    environment.pop("PYTHONHOME", None)
    venv = directory / "venv"
    subprocess.run(
        ["uv", "venv", "--python", sys.executable, str(venv)],
        cwd=directory,
        env=environment,
        check=True,
    )
    python = venv / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    environment["VIRTUAL_ENV"] = str(venv)
    environment["PATH"] = str(python.parent) + os.pathsep + environment.get("PATH", "")
    subprocess.run(
        ["uv", "pip", "install", "--python", str(python), str(wheel)],
        cwd=directory,
        env=environment,
        check=True,
    )
    # Copy consumers so neither Python nor a type checker can find source stubs.
    consumer = directory / "consumer.py"
    shutil.copyfile(ROOT / "tests/python/typing/consumer.py", consumer)
    shutil.copyfile(ROOT / "tests/python/typing/negative.py", directory / "negative.py")
    for name in ["smoke_wheel.py", "check_native_stub.py"]:
        target = directory / name
        shutil.copyfile(ROOT / "scripts" / name, target)
    subprocess.run(
        [
            str(python),
            "-c",
            "import btpc, pathlib; "
            "root = pathlib.Path(btpc.__file__).resolve().parent; "
            "assert root.is_relative_to(pathlib.Path('venv').resolve()); "
            "assert (root / 'py.typed').is_file(); "
            "assert (root / '_native.pyi').is_file()",
        ],
        cwd=directory,
        env=environment,
        check=True,
    )
    for script, arguments in [
        ("check_native_stub.py", []),
        ("smoke_wheel.py", [str(directory / "payload-checks")]),
    ]:
        subprocess.run(
            [str(python), script, *arguments],
            cwd=directory,
            env=environment,
            check=True,
        )
    (directory / "pyrefly.toml").write_text(
        'project-includes = ["consumer.py"]\n'
        'python-version = "3.11.0"\npreset = "legacy"\n',
        encoding="utf-8",
    )
    (directory / "pyrightconfig.json").write_text(
        json.dumps(
            {
                "include": ["consumer.py"],
                "pythonVersion": "3.11",
                "typeCheckingMode": "strict",
                "reportMissingTypeStubs": "error",
                "venvPath": str(directory),
                "venv": "venv",
                "extraPaths": [],
            }
        ),
        encoding="utf-8",
    )
    subprocess.run(
        [
            "pyrefly",
            "check",
            "--config",
            "pyrefly.toml",
            "--python-interpreter-path",
            str(python),
            str(consumer),
        ],
        cwd=directory,
        env=environment,
        check=True,
    )
    subprocess.run(
        ["pyright", "--project", "pyrightconfig.json", "--pythonpath", str(python)],
        cwd=directory,
        env=environment,
        check=True,
    )
    for command in [
        [
            "pyrefly",
            "check",
            "--config",
            "pyrefly.toml",
            "--python-interpreter-path",
            str(python),
            "negative.py",
        ],
        ["pyright", "--pythonpath", str(python), "negative.py"],
    ]:
        result = subprocess.run(
            command,
            cwd=directory,
            env=environment,
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 1:
            raise RuntimeError(
                f"negative typing consumer did not fail normally: {result.stdout}\n{result.stderr}"
            )
    subprocess.run(
        [
            "pyright",
            "--project",
            "pyrightconfig.json",
            "--verifytypes",
            "btpc",
            "--ignoreexternal",
            "--pythonpath",
            str(python),
        ],
        cwd=directory,
        env=environment,
        check=True,
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("wheel", type=Path)
    parser.add_argument("--work-dir", type=Path)
    arguments = parser.parse_args()
    wheel = arguments.wheel.resolve(strict=True)
    if arguments.work_dir is not None:
        check(wheel, arguments.work_dir.resolve())
    else:
        with tempfile.TemporaryDirectory(prefix="btpc-wheel-") as temporary:
            check(wheel, Path(temporary) / "installed")


if __name__ == "__main__":
    main()
