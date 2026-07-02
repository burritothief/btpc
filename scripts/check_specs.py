"""Validate BTPC's normative specification registry and traceability."""

from __future__ import annotations

import json
import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPECS = ROOT / "specs"
ALLOWED_STATUSES = {"Draft", "Accepted", "Implemented", "Deprecated"}
REQUIREMENT_HEADING = re.compile(
    r"^### (?P<id>[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)+-\d{3}) — (?P<title>.+)$",
    re.MULTILINE,
)
ANNOTATION = re.compile(r"\bSpec:\s*([A-Z][A-Z0-9]*(?:-[A-Z0-9]+)+-\d{3})\b")
MARKDOWN_LINK = re.compile(r"\[[^]]+\]\((?!https?://|mailto:|#)([^)#]+)(?:#[^)]+)?\)")
FIELD = re.compile(
    r"^- \*\*(Status|Sources|Verification|Depends on):\*\*\s*(.*)$", re.MULTILINE
)
DATE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


@dataclass(frozen=True)
class Requirement:
    """One normative requirement parsed from a subsystem specification."""

    requirement_id: str
    status: str
    sources: tuple[str, ...]
    verification: tuple[str, ...]
    dependencies: tuple[str, ...]
    spec_path: Path


@dataclass(frozen=True)
class Specification:
    """A subsystem specification and its parsed requirements."""

    spec_id: str
    path: Path
    metadata: dict[str, object]
    requirements: tuple[Requirement, ...]


def _parse_scalar(value: str) -> object:
    if value == "[]":
        return []
    if value.startswith('"') and value.endswith('"'):
        return value[1:-1]
    return value


def parse_front_matter(path: Path, text: str) -> tuple[dict[str, object], str]:
    """Parse the deliberately small YAML subset used by BTPC specs."""
    lines = text.splitlines()
    if not lines or lines[0] != "---":
        raise ValueError(f"{path}: missing opening YAML front matter delimiter")
    try:
        end = lines.index("---", 1)
    except ValueError as error:
        raise ValueError(
            f"{path}: missing closing YAML front matter delimiter"
        ) from error

    metadata: dict[str, object] = {}
    current_list: list[str] | None = None
    for line_number, line in enumerate(lines[1:end], start=2):
        if not line.strip():
            continue
        if line.startswith("  - "):
            if current_list is None:
                raise ValueError(f"{path}:{line_number}: list item has no key")
            current_list.append(str(_parse_scalar(line[4:].strip())))
            continue
        match = re.fullmatch(r"([a-z_]+):(?:\s*(.*))?", line)
        if match is None:
            raise ValueError(f"{path}:{line_number}: unsupported front matter syntax")
        key, raw_value = match.groups()
        if key in metadata:
            raise ValueError(f"{path}:{line_number}: duplicate front matter key {key}")
        if raw_value:
            metadata[key] = _parse_scalar(raw_value.strip())
            current_list = None
        else:
            values: list[str] = []
            metadata[key] = values
            current_list = values
    return metadata, "\n".join(lines[end + 1 :])


def _paths(value: str) -> tuple[str, ...]:
    if value in {"", "None"}:
        return ()
    paths = tuple(re.findall(r"`([^`]+)`", value))
    if not paths:
        raise ValueError(f"expected backtick-delimited paths or IDs, got {value!r}")
    return paths


def parse_requirements(path: Path, body: str, spec_id: str) -> tuple[Requirement, ...]:
    matches = list(REQUIREMENT_HEADING.finditer(body))
    requirements: list[Requirement] = []
    for index, match in enumerate(matches):
        requirement_id = match.group("id")
        if not requirement_id.startswith(f"{spec_id}-"):
            raise ValueError(f"{path}: {requirement_id} must start with {spec_id}-")
        end = matches[index + 1].start() if index + 1 < len(matches) else len(body)
        block = body[match.end() : end]
        fields = dict(FIELD.findall(block))
        missing = {"Status", "Sources", "Verification", "Depends on"} - fields.keys()
        if missing:
            raise ValueError(
                f"{path}: {requirement_id} missing fields: {sorted(missing)}"
            )
        status = fields["Status"]
        if status not in ALLOWED_STATUSES:
            raise ValueError(f"{path}: {requirement_id} has invalid status {status!r}")
        requirements.append(
            Requirement(
                requirement_id=requirement_id,
                status=status,
                sources=_paths(fields["Sources"]),
                verification=_paths(fields["Verification"]),
                dependencies=_paths(fields["Depends on"]),
                spec_path=path,
            )
        )
    if not requirements:
        raise ValueError(f"{path}: no requirement headings found")
    return tuple(requirements)


def _validate_metadata(path: Path, metadata: dict[str, object]) -> None:
    schema = json.loads((SPECS / "schema.json").read_text())
    required = set(schema["required"])
    missing = required - metadata.keys()
    extras = metadata.keys() - schema["properties"].keys()
    if missing:
        raise ValueError(f"{path}: missing front matter keys: {sorted(missing)}")
    if extras:
        raise ValueError(f"{path}: unknown front matter keys: {sorted(extras)}")
    spec_id = metadata["spec_id"]
    if not isinstance(spec_id, str) or re.fullmatch(r"[A-Z][A-Z0-9]*", spec_id) is None:
        raise ValueError(f"{path}: invalid spec_id {spec_id!r}")
    if metadata["status"] not in ALLOWED_STATUSES:
        raise ValueError(f"{path}: invalid status {metadata['status']!r}")
    if not isinstance(metadata["owners"], list) or not metadata["owners"]:
        raise ValueError(f"{path}: owners must be a non-empty list")
    for key in ("source_paths", "test_paths"):
        if not isinstance(metadata[key], list):
            raise ValueError(f"{path}: {key} must be a list")
    if (
        not isinstance(metadata["last_reviewed"], str)
        or DATE.fullmatch(metadata["last_reviewed"]) is None
    ):
        raise ValueError(f"{path}: last_reviewed must use YYYY-MM-DD")


def load_specifications() -> tuple[Specification, ...]:
    specs: list[Specification] = []
    for path in sorted(SPECS.glob("*.md")):
        if path.name == "README.md":
            continue
        metadata, body = parse_front_matter(path, path.read_text())
        _validate_metadata(path, metadata)
        spec_id = str(metadata["spec_id"])
        specs.append(
            Specification(
                spec_id=spec_id,
                path=path,
                metadata=metadata,
                requirements=parse_requirements(path, body, spec_id),
            )
        )
    return tuple(specs)


def _validate_paths(paths: tuple[str, ...] | list[str], context: str) -> None:
    for raw_path in paths:
        path = ROOT / raw_path
        if not path.exists():
            raise ValueError(f"{context}: referenced path does not exist: {raw_path}")


def _source_files(ownership: dict[str, object]) -> set[str]:
    extensions = set(ownership["extensions"])
    files: set[str] = set()
    for root_name in ownership["source_roots"]:
        root = ROOT / root_name
        for path in root.rglob("*"):
            if (
                path.is_file()
                and path.suffix.lstrip(".") in extensions
                and "__pycache__" not in path.parts
            ):
                files.add(path.relative_to(ROOT).as_posix())
    return files


def _validate_ownership(spec_ids: set[str]) -> None:
    data = tomllib.loads((SPECS / "ownership.toml").read_text())
    mappings: dict[str, set[str]] = {}
    for entry in data["ownership"]:
        path = entry["path"]
        if path in mappings:
            raise ValueError(f"specs/ownership.toml: duplicate ownership path {path}")
        _validate_paths((path,), "specs/ownership.toml")
        owned_specs = set(entry["specs"])
        unknown = owned_specs - spec_ids
        if unknown:
            raise ValueError(
                f"specs/ownership.toml: {path} has unknown specs {sorted(unknown)}"
            )
        mappings[path] = owned_specs
    missing = _source_files(data) - mappings.keys()
    if missing:
        raise ValueError(
            f"specs/ownership.toml: uncovered source paths: {sorted(missing)}"
        )


def _validate_links(path: Path, text: str) -> None:
    for target in MARKDOWN_LINK.findall(text):
        if not (path.parent / target).resolve().exists():
            raise ValueError(f"{path}: broken relative link {target}")


def _annotations() -> dict[str, set[str]]:
    references: dict[str, set[str]] = {}
    for root_name in ("crates", "python", "tests"):
        root = ROOT / root_name
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file() or path.suffix not in {".rs", ".py", ".pyi"}:
                continue
            for requirement_id in ANNOTATION.findall(path.read_text(errors="ignore")):
                references.setdefault(requirement_id, set()).add(
                    path.relative_to(ROOT).as_posix()
                )
    return references


def validate() -> None:
    specifications = load_specifications()
    if not specifications:
        raise ValueError("specs/: no subsystem specifications found")
    spec_ids = [spec.spec_id for spec in specifications]
    if len(spec_ids) != len(set(spec_ids)):
        raise ValueError("specs/: duplicate spec_id")

    requirements: dict[str, Requirement] = {}
    for spec in specifications:
        _validate_paths(spec.metadata["source_paths"], str(spec.path))
        _validate_paths(spec.metadata["test_paths"], str(spec.path))
        _validate_links(spec.path, spec.path.read_text())
        for requirement in spec.requirements:
            if requirement.requirement_id in requirements:
                raise ValueError(
                    f"duplicate requirement ID {requirement.requirement_id}"
                )
            requirements[requirement.requirement_id] = requirement
            _validate_paths(requirement.sources, requirement.requirement_id)
            _validate_paths(requirement.verification, requirement.requirement_id)

    for requirement in requirements.values():
        unknown = set(requirement.dependencies) - requirements.keys()
        if unknown:
            raise ValueError(
                f"{requirement.requirement_id}: unknown dependencies {sorted(unknown)}"
            )
        if requirement.status == "Implemented" and not requirement.verification:
            raise ValueError(
                f"{requirement.requirement_id}: implemented requirement lacks verification"
            )

    annotations = _annotations()
    unknown_annotations = annotations.keys() - requirements.keys()
    if unknown_annotations:
        raise ValueError(f"unknown Spec annotations: {sorted(unknown_annotations)}")
    non_implemented = {
        requirement_id
        for requirement_id in annotations
        if requirements[requirement_id].status != "Implemented"
    }
    if non_implemented:
        raise ValueError(
            f"annotations reference non-implemented requirements: {sorted(non_implemented)}"
        )
    missing_annotations = {
        requirement_id
        for requirement_id, requirement in requirements.items()
        if requirement.status == "Implemented" and requirement_id not in annotations
    }
    if missing_annotations:
        raise ValueError(
            f"implemented requirements lack Spec annotations: {sorted(missing_annotations)}"
        )

    _validate_ownership(set(spec_ids))
    _validate_links(SPECS / "README.md", (SPECS / "README.md").read_text())
    print(f"validated {len(specifications)} specs and {len(requirements)} requirements")


def main() -> int:
    try:
        validate()
    except (KeyError, OSError, ValueError, tomllib.TOMLDecodeError) as error:
        print(f"spec validation failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
