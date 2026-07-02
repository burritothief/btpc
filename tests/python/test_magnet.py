from __future__ import annotations

import shutil
import subprocess
from typing import TYPE_CHECKING

import btpc
import pytest

if TYPE_CHECKING:
    from pathlib import Path


@pytest.mark.parametrize("mode", list(btpc.TorrentMode))
def test_magnet_matches_cli_for_every_mode(
    tmp_path: Path, mode: btpc.TorrentMode
) -> None:
    payload = tmp_path / "payload"
    torrent = tmp_path / "payload.torrent"
    payload.write_bytes(b"magnet payload")
    result = btpc.create(payload, torrent, options=btpc.CreateOptions(mode=mode))
    metainfo = btpc.Metainfo.from_bytes(result.bytes)
    magnet = metainfo.magnet()
    assert magnet.startswith("magnet:?xt=")
    assert ("urn:btih:" in magnet) is (mode is not btpc.TorrentMode.V2)
    assert ("urn:btmh:1220" in magnet) is (mode is not btpc.TorrentMode.V1)

    cargo = shutil.which("cargo")
    assert cargo is not None
    process = subprocess.run(  # noqa: S603
        [cargo, "run", "-q", "-p", "btpc-cli", "--", "magnet", str(torrent)],
        check=True,
        capture_output=True,
        text=True,
    )
    assert process.stdout == f"{magnet}\n"
    assert process.stderr == ""


def test_magnet_options_omit_parameters(tmp_path: Path) -> None:
    payload = tmp_path / "payload"
    payload.write_bytes(b"data")
    metainfo = btpc.Metainfo.from_bytes(btpc.create_bytes(payload).bytes)
    magnet = metainfo.magnet(display_name=False, trackers=False, web_seeds=False)
    assert "&dn=" not in magnet
    assert "&tr=" not in magnet
    assert "&ws=" not in magnet
