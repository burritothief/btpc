from __future__ import annotations

import argparse
import json
import subprocess
import tempfile
from pathlib import Path


def run(binary: Path, *arguments: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [binary, *arguments],
        check=True,
        capture_output=True,
        text=True,
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", type=Path)
    arguments = parser.parse_args()
    binary = arguments.binary.resolve()
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        payload = root / "payload"
        payload.mkdir()
        (payload / "hello.txt").write_bytes(b"hello torrent\n")
        for mode in ["v1", "v2", "hybrid"]:
            torrent = root / f"payload-{mode}.torrent"
            command = [
                "create",
                str(payload),
                "--mode",
                mode,
                "--threads",
                "1",
                "--creation-date",
                "0",
                "--json",
                "-o",
                str(torrent),
            ]
            if mode != "v1":
                command.extend(["--piece-length", "16384"])
            created = json.loads(run(binary, *command).stdout)
            if created["mode"] != mode:
                raise RuntimeError(f"create mode mismatch for {mode}")
            inspected = json.loads(
                run(binary, "inspect", str(torrent), "--json").stdout
            )
            if inspected["mode"] != mode:
                raise RuntimeError(f"inspect mode mismatch for {mode}")
            if not run(binary, "magnet", str(torrent)).stdout.startswith("magnet:?xt="):
                raise RuntimeError(f"invalid magnet for {mode}")
            verified = json.loads(
                run(binary, "verify", str(torrent), str(payload), "--json").stdout
            )
            if not verified["valid"]:
                raise RuntimeError(f"verification failed for {mode}")


if __name__ == "__main__":
    main()
