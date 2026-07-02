from __future__ import annotations

import argparse
import hashlib
import shutil
import tarfile
import tempfile
import zipfile
from pathlib import Path


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> None:
    # Spec: RELEASE-CLI-DOC-001
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    arguments.output.mkdir(parents=True, exist_ok=True)
    archive_name = f"btpc-{arguments.version}-{arguments.target}"
    suffix = ".zip" if arguments.binary.suffix == ".exe" else ".tar.gz"
    archive = arguments.output / f"{archive_name}{suffix}"
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary) / archive_name
        root.mkdir()
        shutil.copy2(arguments.binary, root / arguments.binary.name)
        for name in ["README.md", "CHANGELOG.md", "LICENSE"]:
            shutil.copy2(name, root / name)
        shutil.copy2("docs/reference/btpc.1", root / "btpc.1")
        shutil.copytree("docs/completions", root / "completions")
        if suffix == ".zip":
            with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as output:
                for path in sorted(root.rglob("*")):
                    if path.is_file():
                        output.write(path, path.relative_to(root.parent))
        else:
            with tarfile.open(archive, "w:gz") as output:
                output.add(root, arcname=root.name)
    checksum = arguments.output / f"{archive.name}.sha256"
    checksum.write_text(f"{sha256(archive)}  {archive.name}\n")
    print(archive)


if __name__ == "__main__":
    main()
