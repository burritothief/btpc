from __future__ import annotations

import argparse
import shutil
import tarfile
import tempfile
from pathlib import Path

EXCLUDED = {
    ".git",
    ".pytest_cache",
    ".ruff_cache",
    ".tmp",
    ".venv",
    "benchmark-data",
    "benchmark-results",
    "dist",
    "target",
}
EXCLUDED_SUFFIXES = {".iso", ".pyc"}


def included(path: Path) -> bool:
    return (
        not any(part in EXCLUDED for part in path.parts)
        and path.suffix not in EXCLUDED_SUFFIXES
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    repository = Path(__file__).resolve().parents[1]
    arguments.output.mkdir(parents=True, exist_ok=True)
    archive = arguments.output / f"btpc-{arguments.version}-source.tar.gz"
    root_name = f"btpc-{arguments.version}-source"
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary) / root_name
        root.mkdir()
        for source in sorted(repository.iterdir()):
            if not included(source.relative_to(repository)):
                continue
            destination = root / source.name
            if source.is_dir():
                shutil.copytree(
                    source,
                    destination,
                    ignore=lambda directory, names: {
                        name
                        for name in names
                        if not included(
                            (Path(directory) / name).relative_to(repository)
                        )
                    },
                )
            else:
                shutil.copy2(source, destination)
        with tarfile.open(archive, "w:gz") as output:
            output.add(root, arcname=root.name)
    print(archive)


if __name__ == "__main__":
    main()
