from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).parents[1]
SEMVER = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$")


def workspace_version() -> str:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
    return str(cargo["workspace"]["package"]["version"])


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag")
    arguments = parser.parse_args()
    version = workspace_version()
    errors: list[str] = []
    if not SEMVER.fullmatch(version):
        errors.append(f"workspace version is not semver: {version}")
    pyproject = tomllib.loads((ROOT / "pyproject.toml").read_text())
    project = pyproject["project"]
    if "version" in project or "version" not in project.get("dynamic", []):
        errors.append("pyproject must derive its dynamic version from Cargo")
    for relative in ["crates/btpc-cli/Cargo.toml", "crates/btpc-python/Cargo.toml"]:
        manifest = tomllib.loads((ROOT / relative).read_text())
        requirement = manifest["dependencies"]["btpc-core"].get("version")
        if requirement != version:
            errors.append(
                f"{relative} btpc-core requirement {requirement!r} does not match {version}"
            )
    metadata = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    workspace_versions = {
        package["name"]: package["version"]
        for package in metadata["packages"]
        if package["id"] in metadata["workspace_members"]
    }
    for name, package_version in workspace_versions.items():
        if package_version != version:
            errors.append(
                f"workspace package {name} reports {package_version}, expected {version}"
            )
    changelog = (ROOT / "CHANGELOG.md").read_text()
    if f"## [{version}]" not in changelog:
        errors.append(f"CHANGELOG.md lacks a [{version}] release section")
    if arguments.tag is not None and arguments.tag != f"v{version}":
        errors.append(f"tag {arguments.tag} does not match v{version}")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(version)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
