"""Deterministic benchmark dataset generation and fingerprinting."""

from __future__ import annotations

import hashlib
import json
import random
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from .models import DatasetFingerprint

DEFAULT_PIECE_LENGTH = 4 * 1024 * 1024
CANONICAL_ISO_NAME = "debian-13.5.0-amd64-DVD-1.iso"
CANONICAL_ISO_SIZE = 3_989_078_016
CANONICAL_ISO_SHA256 = (
    "343b6e02a8bdf6429eb3722ee0056b5c7d9ad17d88328e499909da7205e55f50"
)


@dataclass(frozen=True, slots=True)
class GeneratedDataset:
    """Paths and checksums for one generated payload."""

    payload_path: Path
    manifest_path: Path
    seed: int
    size_bytes: int
    sha256: str
    piece_length: int
    piece_sha1: tuple[str, ...]

    def fingerprint(self) -> DatasetFingerprint:
        """Return the serializable benchmark fingerprint."""
        return DatasetFingerprint(
            path=str(self.payload_path),
            size_bytes=self.size_bytes,
            sha256=self.sha256,
            piece_length=self.piece_length,
            piece_sha1=self.piece_sha1,
            name=self.payload_path.name,
            mtime_ns=self.payload_path.stat().st_mtime_ns,
        )


def generate_dataset(
    root: Path,
    *,
    seed: int,
    size_bytes: int,
    piece_length: int = DEFAULT_PIECE_LENGTH,
    name: str = "payload.bin",
) -> GeneratedDataset:
    """Generate a deterministic single-file payload without large allocations."""
    if size_bytes <= 0:
        msg = "size_bytes must be positive"
        raise ValueError(msg)
    root.mkdir(parents=True, exist_ok=True)
    payload_path = root / name
    random_source = random.Random(seed)
    remaining = size_bytes
    with payload_path.open("wb") as stream:
        while remaining:
            count = min(1024 * 1024, remaining)
            stream.write(random_source.randbytes(count))
            remaining -= count
    fingerprint = fingerprint_payload(payload_path, piece_length=piece_length)
    manifest_path = root / "manifest.json"
    document = {
        "format": 1,
        "seed": seed,
        "name": name,
        **asdict(fingerprint),
    }
    document["path"] = name
    manifest_path.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n")
    return GeneratedDataset(
        payload_path=payload_path,
        manifest_path=manifest_path,
        seed=seed,
        size_bytes=size_bytes,
        sha256=fingerprint.sha256,
        piece_length=piece_length,
        piece_sha1=fingerprint.piece_sha1,
    )


def fingerprint_payload(
    path: Path,
    *,
    piece_length: int = DEFAULT_PIECE_LENGTH,
) -> DatasetFingerprint:
    """Stream full-file SHA-256 and per-piece SHA-1 checksums."""
    full = hashlib.sha256()
    pieces: list[str] = []
    size = 0
    with path.open("rb") as stream:
        while chunk := stream.read(piece_length):
            size += len(chunk)
            full.update(chunk)
            pieces.append(hashlib.sha1(chunk).hexdigest())
    metadata = path.stat()
    return DatasetFingerprint(
        path=str(path),
        size_bytes=size,
        sha256=full.hexdigest(),
        piece_length=piece_length,
        piece_sha1=tuple(pieces),
        name=path.name,
        mtime_ns=metadata.st_mtime_ns,
    )


def validate_canonical_iso(fingerprint: DatasetFingerprint) -> None:
    """Reject a mislabeled canonical Debian benchmark payload."""
    if Path(fingerprint.path).name != CANONICAL_ISO_NAME:
        return
    if (
        fingerprint.size_bytes != CANONICAL_ISO_SIZE
        or fingerprint.sha256 != CANONICAL_ISO_SHA256
    ):
        msg = "canonical Debian ISO fingerprint mismatch"
        raise ValueError(msg)


def load_manifest(path: Path) -> dict[str, Any]:
    """Load a generated manifest while ignoring its root-specific path."""
    document: dict[str, Any] = json.loads(path.read_text())
    document.pop("path", None)
    document.pop("mtime_ns", None)
    return document
