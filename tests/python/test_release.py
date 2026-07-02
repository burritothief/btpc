from __future__ import annotations

import shutil
import subprocess
import sys
import tarfile
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).parents[2]


def workspace_version() -> str:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
    return str(cargo["workspace"]["package"]["version"])


# Spec: RELEASE-VERSION-001
def test_python_version_is_dynamic_and_matches_cargo_workspace() -> None:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
    pyproject = tomllib.loads((ROOT / "pyproject.toml").read_text())
    assert "version" not in pyproject["project"]
    assert "version" in pyproject["project"]["dynamic"]
    version = cargo["workspace"]["package"]["version"]
    for relative in ["crates/btpc-cli/Cargo.toml", "crates/btpc-python/Cargo.toml"]:
        manifest = tomllib.loads((ROOT / relative).read_text())
        assert manifest["dependencies"]["btpc-core"]["version"] == version


def test_project_license_metadata_is_mit() -> None:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
    pyproject = tomllib.loads((ROOT / "pyproject.toml").read_text())
    assert cargo["workspace"]["package"]["license"] == "MIT"
    assert pyproject["project"]["license"] == "MIT"
    assert {"path": "LICENSE", "format": "sdist"} in pyproject["tool"]["maturin"][
        "include"
    ]
    license_text = (ROOT / "LICENSE").read_text()
    assert "MIT License" in license_text
    assert "Copyright (c) 2026 Jeff" in license_text
    for crate in ["btpc-core", "btpc-cli", "btpc-python"]:
        assert (ROOT / "crates" / crate / "LICENSE").read_text() == license_text


def test_cargo_packages_include_license() -> None:
    cargo = shutil.which("cargo")
    assert cargo is not None
    for package in ["btpc-core", "btpc-cli", "btpc-python"]:
        result = subprocess.run(  # noqa: S603
            [
                cargo,
                "package",
                "--package",
                package,
                "--allow-dirty",
                "--list",
            ],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        assert "LICENSE" in result.stdout.splitlines()


# Spec: RELEASE-ARTIFACT-001
def test_release_automation_and_changelog_are_present() -> None:
    assert (ROOT / "CHANGELOG.md").is_file()
    workflow = (ROOT / ".github" / "workflows" / "release.yml").read_text()
    assert "workflow_dispatch" in workflow
    assert "attest-build-provenance" in workflow
    assert "gh-action-pypi-publish" in workflow
    assert "publish: true" not in workflow
    assert "publishing requires an existing vX.Y.Z tag" in workflow
    assert (ROOT / "scripts" / "check_version.py").is_file()
    assert (ROOT / "scripts" / "set_version.py").is_file()
    assert (ROOT / "scripts" / "package_cli.py").is_file()
    assert (ROOT / "scripts" / "package_source.py").is_file()
    assert (ROOT / "scripts" / "verify_artifacts.py").is_file()
    assert (ROOT / "scripts" / "write_gate_summary.py").is_file()
    assert "--python" in (ROOT / "scripts" / "verify_artifacts.py").read_text()
    assert "--binary" in (ROOT / "scripts" / "verify_artifacts.py").read_text()


def test_artifact_validator_distinguishes_sdist_and_cli_archives(
    tmp_path: Path,
) -> None:
    version = workspace_version()
    wheel = tmp_path / f"btpc-{version}-py3-none-any.whl"
    with zipfile.ZipFile(wheel, "w") as archive:
        archive.writestr(
            f"btpc-{version}.dist-info/METADATA",
            (
                f"Metadata-Version: 2.4\nName: btpc\nVersion: {version}\n"
                "License-Expression: MIT\n"
            ),
        )
        archive.writestr(f"btpc-{version}.dist-info/licenses/LICENSE", "MIT License")
    sdist = tmp_path / f"btpc-{version}.tar.gz"
    package_root = f"btpc-{version}"
    with tarfile.open(sdist, "w:gz") as archive:
        for name, content in [
            (f"{package_root}/LICENSE", b"MIT License"),
            (
                f"{package_root}/PKG-INFO",
                (
                    f"Metadata-Version: 2.4\nName: btpc\nVersion: {version}\n"
                    "License-Expression: MIT\n"
                ).encode(),
            ),
        ]:
            info = tarfile.TarInfo(name)
            info.size = len(content)
            archive.addfile(info, __import__("io").BytesIO(content))
    result = subprocess.run(  # noqa: S603
        [
            sys.executable,
            ROOT / "scripts" / "verify_artifacts.py",
            tmp_path,
            "--version",
            version,
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode != 0
    assert "no native CLI archives found" in result.stderr


def test_cli_packager_includes_license_and_cli_reference_artifacts(
    tmp_path: Path,
) -> None:
    # Spec: RELEASE-CLI-DOC-001
    binary = tmp_path / "btpc"
    binary.write_text("binary")
    result = subprocess.run(  # noqa: S603
        [
            sys.executable,
            ROOT / "scripts" / "package_cli.py",
            "--binary",
            binary,
            "--target",
            "x86_64-unknown-linux-gnu",
            "--version",
            workspace_version(),
            "--output",
            tmp_path,
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    with tarfile.open(Path(result.stdout.strip())) as archive:
        names = archive.getnames()
        for required in [
            "/LICENSE",
            "/btpc.1",
            "/completions/btpc.bash",
            "/completions/btpc.zsh",
            "/completions/btpc.fish",
            "/completions/btpc.powershell",
            "/completions/btpc.elvish",
        ]:
            assert any(name.endswith(required) for name in names)


def test_source_packager_includes_license_and_manifests(tmp_path: Path) -> None:
    result = subprocess.run(  # noqa: S603
        [
            sys.executable,
            ROOT / "scripts" / "package_source.py",
            "--version",
            workspace_version(),
            "--output",
            tmp_path,
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    with tarfile.open(Path(result.stdout.strip())) as archive:
        names = archive.getnames()
        for required in ["/LICENSE", "/Cargo.toml", "/pyproject.toml"]:
            assert any(name.endswith(required) for name in names)


def test_version_bump_dry_run_covers_every_derived_manifest() -> None:
    result = subprocess.run(  # noqa: S603
        [
            sys.executable,
            ROOT / "scripts" / "set_version.py",
            "9.8.7",
            "--dry-run",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    assert result.stdout.splitlines() == [
        "Cargo.toml",
        "crates/btpc-cli/Cargo.toml",
        "crates/btpc-python/Cargo.toml",
    ]


def test_wheel_and_sdist_include_complete_typing_artifacts(tmp_path: Path) -> None:
    # Spec: RELEASE-PY-TYPING-001
    subprocess.run(  # noqa: S603
        [
            sys.executable,
            "-m",
            "maturin",
            "build",
            "--release",
            "--out",
            str(tmp_path),
        ],
        cwd=ROOT,
        check=True,
    )
    subprocess.run(  # noqa: S603
        [sys.executable, "-m", "maturin", "sdist", "--out", str(tmp_path)],
        cwd=ROOT,
        check=True,
    )
    wheel = next(tmp_path.glob("*.whl"))
    with zipfile.ZipFile(wheel) as archive:
        names = archive.namelist()
        assert "btpc/py.typed" in names
        assert "btpc/_native.pyi" in names
        for module in [
            "creation.py",
            "errors.py",
            "metainfo.py",
            "types.py",
            "verification.py",
        ]:
            assert f"btpc/{module}" in names
        metadata_name = next(
            name for name in names if name.endswith(".dist-info/METADATA")
        )
        metadata = archive.read(metadata_name).decode()
        assert "Keywords: bittorrent,metainfo,cpython-gil-required" in metadata
    sdist = next(
        path for path in tmp_path.glob("*.tar.gz") if "source" not in path.name
    )
    with tarfile.open(sdist) as archive:
        names = archive.getnames()
        assert any(name.endswith("/python/btpc/py.typed") for name in names)
        assert any(name.endswith("/python/btpc/_native.pyi") for name in names)
