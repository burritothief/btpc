from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).parents[1]
SEMVER = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$")
WORKSPACE_VERSION = re.compile(r'(?m)^(\[workspace\.package\]\nversion = ")[^"]+("$)')
CORE_DEPENDENCY_VERSION = re.compile(
    r'(?m)^(btpc-core = \{ version = ")[^"]+("\s*,\s*path = "\.\./btpc-core" \}$)'
)


def planned_updates(root: Path, version: str) -> dict[Path, str]:
    updates: dict[Path, str] = {}
    cargo_path = root / "Cargo.toml"
    cargo, count = WORKSPACE_VERSION.subn(
        rf"\g<1>{version}\g<2>", cargo_path.read_text(), count=1
    )
    if count != 1:
        msg = "could not locate workspace package version"
        raise ValueError(msg)
    updates[cargo_path] = cargo

    for relative in ["crates/btpc-cli/Cargo.toml", "crates/btpc-python/Cargo.toml"]:
        path = root / relative
        manifest, count = CORE_DEPENDENCY_VERSION.subn(
            rf"\g<1>{version}\g<2>", path.read_text(), count=1
        )
        if count != 1:
            msg = f"could not locate btpc-core dependency version in {relative}"
            raise ValueError(msg)
        updates[path] = manifest
    return updates


def apply_updates(updates: dict[Path, str]) -> None:
    originals = {path: path.read_text() for path in updates}
    try:
        for path, text in updates.items():
            temporary = path.with_suffix(f"{path.suffix}.tmp")
            temporary.write_text(text)
            temporary.replace(path)
    except BaseException:
        for path, text in originals.items():
            path.write_text(text)
        raise


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("version")
    parser.add_argument("--dry-run", action="store_true")
    arguments = parser.parse_args()
    if not SEMVER.fullmatch(arguments.version):
        msg = f"invalid semantic version: {arguments.version}"
        raise SystemExit(msg)
    try:
        updates = planned_updates(ROOT, arguments.version)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    if arguments.dry_run:
        for path in updates:
            print(path.relative_to(ROOT))
        return
    apply_updates(updates)
    subprocess.run(["cargo", "metadata", "--format-version", "1"], cwd=ROOT, check=True)
    print(
        f"set workspace and internal dependency versions to {arguments.version}; "
        "update CHANGELOG.md and generated references next"
    )


if __name__ == "__main__":
    main()
