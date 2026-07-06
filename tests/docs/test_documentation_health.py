from __future__ import annotations

import importlib.util
import json
import re
import sys
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

import yaml

if TYPE_CHECKING:
    from types import ModuleType

ROOT = Path(__file__).parents[2]
WORKFLOW = ROOT / ".github/workflows/maintenance.yml"
CONFIG = ROOT / ".lychee.toml"
MANIFEST = ROOT / ".github/docs-health.json"
CHECKER = ROOT / "scripts/check_docs_health.py"
COLLECTOR = ROOT / "scripts/collect_docs_external_links.py"
FIXTURES = ROOT / "tests/docs/fixtures/health"
SHA_ACTION = re.compile(r"^[^@]+@[0-9a-f]{40}$")
JOB_TIMEOUT_MINUTES = 20
MAX_RETRIES = 3
RETRY_WAIT_SECONDS = 2
REQUEST_TIMEOUT_SECONDS = 20
MAX_CONCURRENCY = 8
HOST_CONCURRENCY = 2
NOT_FOUND = 404


def _workflow() -> dict[str, object]:
    data = yaml.safe_load(WORKFLOW.read_text())
    assert isinstance(data, dict)
    if True in data:
        data["on"] = data.pop(True)
    return data


def _module(name: str, path: Path) -> ModuleType:
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_weekly_health_workflow_is_read_only_pinned_and_bounded() -> None:
    workflow = _workflow()
    assert workflow["on"] == {
        "schedule": [{"cron": "47 8 * * 4"}],
        "workflow_dispatch": None,
    }
    assert workflow["permissions"] == {"contents": "read"}
    assert workflow["concurrency"] == {
        "group": "maintenance-${{ github.ref }}",
        "cancel-in-progress": True,
    }
    job = workflow["jobs"]["documentation"]
    assert job["timeout-minutes"] == JOB_TIMEOUT_MINUTES
    assert job["env"] == {
        "LYCHEE_VERSION": "0.24.2",
        "LYCHEE_SHA256": (
            "1f4e0ef7f6554a6ed33dd7ac144fb2e1bbed98598e7af973042fc5cd43951c9a"
        ),
    }
    steps = job["steps"]
    for step in steps:
        if action := step.get("uses"):
            assert SHA_ACTION.fullmatch(action), action
    install = next(step["run"] for step in steps if step["name"] == "Install Lychee")
    rust = next(step for step in steps if step["name"] == "Install Rust")
    assert rust["with"] == {"toolchain": "1.94.1"}
    assert "lychee-v${LYCHEE_VERSION}" in install
    assert "sha256sum --check" in install
    assert "--proto '=https'" in install
    assert "--tlsv1.2" in install
    commands = [step.get("run") for step in steps]
    assert "make docs-check" in commands
    assert "make docs-health" in commands
    summary = steps[-1]
    assert summary["name"] == "Publish documentation health summary"
    assert summary["if"] == "always()"
    assert "docs-link-health.md" in summary["run"]
    assert "docs-external-links.md" in summary["run"]
    assert "GITHUB_STEP_SUMMARY" in CHECKER.read_text()


def test_lychee_policy_retries_limits_hosts_and_has_only_narrow_exclusions() -> None:
    config = tomllib.loads(CONFIG.read_text())
    assert config["max_retries"] == MAX_RETRIES
    assert config["retry_wait_time"] == RETRY_WAIT_SECONDS
    assert config["timeout"] == REQUEST_TIMEOUT_SECONDS
    assert config["max_concurrency"] == MAX_CONCURRENCY
    assert config["host_concurrency"] == HOST_CONCURRENCY
    assert config["host_request_interval"] == "250ms"
    assert config["require_https"] is True
    assert config["accept"] == ["200..=399"]
    assert config["exclude"] == [
        r"^http://127\.0\.0\.1:8000/btpc/$",
        r"^https://(?:tracker|seed)\.example/",
        r"^https://tracker\.invalid/",
        r"^https://burritothief\.github\.io/btpc/",
        r"^https://github\.com/burritothief/btpc/edit/main/",
        r"^https://github\.com/burritothief/btpc/releases/tag/v0\.1\.0$",
        r"^https://github\.com/burritothief/btpc/compare/v0\.1\.0\.\.\.HEAD$",
    ]
    policy = CONFIG.read_text().lower()
    assert "400..=499" not in policy
    assert "4xx" not in policy
    assert "accept_timeouts = true" not in policy


def test_live_manifest_names_every_required_production_entry() -> None:
    checks = json.loads(MANIFEST.read_text())
    assert [check["name"] for check in checks] == [
        "homepage",
        "getting-started",
        "cli",
        "python",
        "rust",
        "search-index",
        "sitemap",
        "stylesheet",
        "custom-404",
    ]
    assert checks[-1]["status"] == NOT_FOUND
    assert checks[-1]["url"].endswith("/health-check/missing/nested/")
    assert all(check["url"].startswith("https://") for check in checks)
    assert all(check["marker"] for check in checks)


def test_live_validator_rejects_broken_generic_missing_and_mixed_content() -> None:
    checker = _module("check_docs_health", CHECKER)
    check = checker.Check(
        name="fixture",
        url="https://example.test/page/",
        status=200,
        content_type="text/html",
        marker="Expected marker",
    )
    cases = {
        "broken-external.json": "status 503",
        "github-404.json": "generic GitHub Pages error",
        "missing-marker.json": "missing marker",
        "mixed-content.json": "mixed-content URL",
    }
    for fixture_name, message in cases.items():
        response = checker.Response(**json.loads((FIXTURES / fixture_name).read_text()))
        assert any(message in error for error in checker.validate(check, response))


def test_live_validator_accepts_the_healthy_fixture() -> None:
    checker = _module("check_docs_health", CHECKER)
    check = checker.Check(
        name="fixture",
        url="https://example.test/page/",
        status=200,
        content_type="text/html",
        marker="Expected marker",
    )
    response = checker.Response(**json.loads((FIXTURES / "healthy.json").read_text()))
    assert checker.validate(check, response) == []


def test_external_link_inventory_names_source_and_generated_pages(
    tmp_path: Path,
) -> None:
    collector = _module("collect_docs_external_links", COLLECTOR)
    source = tmp_path / "guide.md"
    generated = tmp_path / "site/index.html"
    generated.parent.mkdir()
    source.write_text("[external](https://example.test/docs) [local](other.md)\n")
    generated.write_text(
        '<a href="https://example.test/docs">Docs</a>'
        '<script src="https://cdn.example.test/app.js"></script>'
        '<a href="/btpc/local/">Local</a>'
    )

    links = collector.collect((source, generated), root=tmp_path)
    rendered = collector.render(links)

    assert rendered.count("https://example.test/docs") == 1
    assert "guide.md" in rendered
    assert "site/index.html" in rendered
    assert "https://cdn.example.test/app.js" in rendered
    assert "/btpc/local/" not in rendered
