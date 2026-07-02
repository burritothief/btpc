from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

PLATFORMS = ("-linux-", "-apple-", "-windows-")
REQUIRED_ARCHIVE_FILES = ("/README.md", "/CHANGELOG.md", "/LICENSE")
REQUIRED_CLI_DOCS = (
    "/btpc.1",
    "/completions/btpc.bash",
    "/completions/btpc.zsh",
    "/completions/btpc.fish",
    "/completions/btpc.powershell",
    "/completions/btpc.elvish",
)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verify_checksums(directory: Path) -> list[str]:
    errors: list[str] = []
    for checksum in directory.rglob("*.sha256"):
        expected, name = checksum.read_text().strip().split(maxsplit=1)
        artifact = checksum.parent / name
        if not artifact.is_file() or sha256(artifact) != expected:
            errors.append(f"checksum mismatch: {artifact}")
    return errors


def verify_wheels(artifacts: list[Path], version: str) -> list[str]:
    wheels = [path for path in artifacts if path.suffix == ".whl"]
    errors = [] if wheels else ["no wheels found"]
    for wheel in wheels:
        with zipfile.ZipFile(wheel) as archive:
            metadata_names = [
                name
                for name in archive.namelist()
                if name.endswith(".dist-info/METADATA")
            ]
            if len(metadata_names) != 1:
                errors.append(f"invalid wheel metadata: {wheel}")
                continue
            metadata = archive.read(metadata_names[0]).decode()
            if (
                "Name: btpc\n" not in metadata
                or f"Version: {version}\n" not in metadata
            ):
                errors.append(f"wheel name/version mismatch: {wheel}")
            if "License-Expression: MIT\n" not in metadata:
                errors.append(f"wheel license metadata mismatch: {wheel}")
            if not any(
                name.endswith((".dist-info/licenses/LICENSE", "/LICENSE"))
                for name in archive.namelist()
            ):
                errors.append(f"wheel lacks LICENSE: {wheel}")
    return errors


def archive_names(archive: Path) -> list[str]:
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as source:
            return source.namelist()
    with tarfile.open(archive) as source:
        return source.getnames()


def verify_sdists(artifacts: list[Path], version: str) -> list[str]:
    sdists = [
        path
        for path in artifacts
        if path.name == f"btpc-{version}.tar.gz"
        and not any(platform in path.name for platform in PLATFORMS)
    ]
    errors = [] if sdists else ["no source distribution found"]
    for sdist in sdists:
        with tarfile.open(sdist) as archive:
            names = archive.getnames()
            if not any(name.endswith("/LICENSE") for name in names):
                errors.append(f"source distribution lacks LICENSE: {sdist}")
            package_info = next(
                (name for name in names if name.endswith("/PKG-INFO")), None
            )
            if package_info is None:
                errors.append(f"source distribution lacks PKG-INFO: {sdist}")
            else:
                metadata = archive.extractfile(package_info)
                if (
                    metadata is None
                    or "License-Expression: MIT\n" not in metadata.read().decode()
                ):
                    errors.append(
                        f"source distribution license metadata mismatch: {sdist}"
                    )
    return errors


def verify_cli_archives(artifacts: list[Path]) -> list[str]:
    archives = [
        path
        for path in artifacts
        if path.name.endswith((".tar.gz", ".zip"))
        and any(platform in path.name for platform in PLATFORMS)
    ]
    errors = [] if archives else ["no native CLI archives found"]
    for archive in archives:
        names = archive_names(archive)
        if not any(name.endswith(("/btpc", "/btpc.exe")) for name in names):
            errors.append(f"CLI archive lacks executable: {archive}")
        errors.extend(
            f"CLI archive lacks {required[1:]}: {archive}"
            for required in (*REQUIRED_ARCHIVE_FILES, *REQUIRED_CLI_DOCS)
            if not any(name.endswith(required) for name in names)
        )
    return errors


def verify_source_archives(artifacts: list[Path], version: str) -> list[str]:
    archives = [
        path for path in artifacts if path.name == f"btpc-{version}-source.tar.gz"
    ]
    errors = [] if archives else ["no project source archive found"]
    for archive in archives:
        names = archive_names(archive)
        errors.extend(
            f"source archive lacks {required[1:]}: {archive}"
            for required in ["/LICENSE", "/Cargo.toml", "/pyproject.toml"]
            if not any(name.endswith(required) for name in names)
        )
    return errors


def write_aggregate_checksums(directory: Path, artifacts: list[Path]) -> None:
    checksum_lines = [
        f"{sha256(path)}  {path.relative_to(directory)}"
        for path in artifacts
        if path.name != "SHA256SUMS"
    ]
    (directory / "SHA256SUMS").write_text("\n".join(checksum_lines) + "\n")


def verify_runtime_versions(
    version: str, binary: Path | None, python: Path | None
) -> list[str]:
    errors: list[str] = []
    if binary is not None:
        reported = subprocess.run(
            [binary, "--version"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        if reported != f"btpc {version}":
            errors.append(f"CLI version mismatch: {reported!r}")
    if python is not None:
        reported = json.loads(
            subprocess.run(
                [
                    python,
                    "-c",
                    (
                        "import json, importlib.metadata, btpc, btpc._native as n; "
                        "print(json.dumps([importlib.metadata.version('btpc'), "
                        "btpc.__version__, n.__version__]))"
                    ),
                ],
                check=True,
                capture_output=True,
                text=True,
            ).stdout
        )
        if reported != [version, version, version]:
            errors.append(f"Python version mismatch: {reported!r}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("directory", type=Path)
    parser.add_argument("--version", required=True)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--python", type=Path)
    arguments = parser.parse_args()
    artifacts = sorted(
        path
        for path in arguments.directory.rglob("*")
        if path.is_file() and path.suffix != ".sha256"
    )
    errors = [] if artifacts else ["no artifacts found"]
    errors.extend(verify_checksums(arguments.directory))
    errors.extend(verify_wheels(artifacts, arguments.version))
    errors.extend(verify_sdists(artifacts, arguments.version))
    errors.extend(verify_cli_archives(artifacts))
    errors.extend(verify_source_archives(artifacts, arguments.version))
    errors.extend(
        verify_runtime_versions(arguments.version, arguments.binary, arguments.python)
    )
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    write_aggregate_checksums(arguments.directory, artifacts)
    print(f"verified {len(artifacts)} artifacts")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
