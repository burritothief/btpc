from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parents[2]
FIXTURE = ROOT / "tests/docs/fixtures/renderer_migration_baseline.json"
SCRIPT = ROOT / "scripts/docs_renderer_baseline.py"


# Spec: DOCSITE-MIGRATE-001
def test_renderer_migration_baseline_is_versioned_and_deterministic() -> None:
    baseline = json.loads(FIXTURE.read_text())
    assert baseline["schema"] == "btpc.docs-renderer-baseline.v1"
    assert baseline["site_url"] == "https://burritothief.github.io/btpc/"
    assert baseline["routes"] == sorted(
        baseline["routes"], key=lambda route: route["path"]
    )
    assert all(route["mode"] in {"direct", "redirect"} for route in baseline["routes"])
    assert baseline["artifact"]["html_pages"] == len(baseline["routes"])


def test_renderer_comparator_reports_missing_routes_and_anchors(tmp_path: Path) -> None:
    site = tmp_path / "site"
    site.mkdir()
    (site / "index.html").write_text(
        '<html><head><link rel="canonical" href="https://example.test/"></head>'
        '<body><h1 id="present">Present</h1></body></html>'
    )
    manifest = tmp_path / "manifest.json"
    manifest.write_text(
        json.dumps(
            {
                "schema": "btpc.docs-renderer-baseline.v1",
                "routes": [
                    {"path": "index.html", "mode": "direct"},
                    {"path": "missing/index.html", "mode": "direct"},
                ],
                "anchors": {"index.html": ["present", "missing-anchor"]},
                "canonical_urls": {
                    "index.html": "https://example.test/",
                },
                "assets": [],
                "sitemap_routes": [],
                "custom_404": "404.html",
            }
        )
    )
    result = subprocess.run(  # noqa: S603
        [sys.executable, SCRIPT, "compare", "--site-dir", site, "--manifest", manifest],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 1
    assert "missing route: missing/index.html" in result.stderr
    assert "missing anchor: index.html#missing-anchor" in result.stderr
