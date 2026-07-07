from __future__ import annotations

import argparse
import functools
import shutil
import threading
import time
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

from build_mdbook_site import ROOT, build_site

WATCH_ROOTS = (
    ROOT / "book.toml",
    ROOT / "docs",
    ROOT / "python/btpc",
    ROOT / "crates/btpc-cli/src",
    ROOT / "crates/btpc-core/src",
    ROOT / "scripts/mdbook_python_api.py",
    ROOT / "scripts/postprocess_mdbook.py",
)


def _snapshot() -> dict[Path, int]:
    paths: list[Path] = []
    for root in WATCH_ROOTS:
        if root.is_dir():
            paths.extend(path for path in root.rglob("*") if path.is_file())
        elif root.is_file():
            paths.append(root)
    return {path: path.stat().st_mtime_ns for path in paths}


def _watch(destination: Path) -> None:
    previous = _snapshot()
    while True:
        time.sleep(1)
        current = _snapshot()
        if current == previous:
            continue
        print("Documentation source changed; rebuilding preview...", flush=True)
        build_site(destination)
        previous = current
        print("Documentation preview rebuilt.", flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description="Preview the BTPC mdBook site")
    parser.add_argument("--hostname", default="127.0.0.1")
    parser.add_argument("--port", default=8000, type=int)
    arguments = parser.parse_args()
    preview_root = ROOT / ".tmp/docs-preview"
    shutil.rmtree(preview_root, ignore_errors=True)
    preview_root.mkdir(parents=True)
    destination = preview_root / "btpc"
    build_site(destination)
    watcher = threading.Thread(target=_watch, args=(destination,), daemon=True)
    watcher.start()
    handler = functools.partial(SimpleHTTPRequestHandler, directory=preview_root)
    server = ThreadingHTTPServer((arguments.hostname, arguments.port), handler)
    print(
        f"Serving documentation on http://{arguments.hostname}:{arguments.port}/btpc/",
        flush=True,
    )
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        return 0
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
