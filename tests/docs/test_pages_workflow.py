from __future__ import annotations

import re
from pathlib import Path

import yaml

ROOT = Path(__file__).parents[2]
WORKFLOW = ROOT / ".github/workflows/docs.yml"
MDBOOK_VERSION = (ROOT / ".mdbook-version").read_text().strip()
INSTALL_ACTION = "taiki-e/install-action@16b05812d776ae1dfaabc8277e421fb6d2506419"
SHA_ACTION = re.compile(r"^[^@]+@[0-9a-f]{40}$")
CONCURRENCY_GROUP = (
    "docs-${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}"
)
BUILD_TIMEOUT_MINUTES = 20
DEPLOY_TIMEOUT_MINUTES = 10


def _workflow() -> dict[str, object]:
    data = yaml.safe_load(WORKFLOW.read_text())
    assert isinstance(data, dict)
    if True in data:
        data["on"] = data.pop(True)
    return data


def test_pages_workflow_triggers_without_path_filters() -> None:
    workflow = _workflow()
    triggers = workflow["on"]
    assert isinstance(triggers, dict)
    assert set(triggers) == {"pull_request", "push", "workflow_dispatch"}
    assert triggers["pull_request"] is None
    assert triggers["push"] == {"branches": ["main"]}
    assert triggers["workflow_dispatch"] is None
    assert "paths" not in WORKFLOW.read_text()
    assert workflow["permissions"] == {"contents": "read"}
    assert workflow["concurrency"] == {
        "group": CONCURRENCY_GROUP,
        "cancel-in-progress": (
            "${{ github.event_name != 'push' || github.ref != 'refs/heads/main' }}"
        ),
    }


def test_pages_build_is_read_only_locked_and_uploads_only_site() -> None:
    workflow = _workflow()
    jobs = workflow["jobs"]
    assert isinstance(jobs, dict)
    build = jobs["build"]
    assert build["timeout-minutes"] == BUILD_TIMEOUT_MINUTES
    assert "permissions" not in build
    steps = build["steps"]
    mdbook = next(step for step in steps if step["name"] == "Install mdBook")
    assert mdbook["uses"] == INSTALL_ACTION
    assert mdbook["with"] == {"tool": f"mdbook@{MDBOOK_VERSION}"}
    assert any(step.get("run") == "uv sync --all-groups --locked" for step in steps)
    assert any(step.get("run") == "make docs-check" for step in steps)
    checkout = next(
        step for step in steps if step.get("uses", "").startswith("actions/checkout@")
    )
    assert checkout["with"]["persist-credentials"] is False
    configure = next(
        step
        for step in steps
        if step.get("uses", "").startswith("actions/configure-pages@")
    )
    upload = next(
        step
        for step in steps
        if step.get("uses", "").startswith("actions/upload-pages-artifact@")
    )
    assert upload["with"] == {"path": "site", "retention-days": 1}
    assert configure
    for step in steps:
        if action := step.get("uses"):
            assert SHA_ACTION.match(action), action

    workflow_text = WORKFLOW.read_text().lower()
    assert "mkdocs" not in workflow_text
    assert "curl" not in workflow_text
    assert "gh-pages" not in workflow_text


def test_pages_deploy_has_only_deployment_permissions_and_trust_condition() -> None:
    workflow = _workflow()
    deploy = workflow["jobs"]["deploy"]
    assert deploy["needs"] == "build"
    assert deploy["timeout-minutes"] == DEPLOY_TIMEOUT_MINUTES
    assert deploy["permissions"] == {"pages": "write", "id-token": "write"}
    assert deploy["environment"] == {
        "name": "github-pages",
        "url": "${{ steps.deployment.outputs.page_url }}",
    }
    assert deploy["concurrency"] == {
        "group": "pages-production",
        "cancel-in-progress": False,
    }
    condition = deploy["if"]
    assert "github.event_name == 'push'" in condition
    assert "github.ref == 'refs/heads/main'" in condition
    assert "github.event_name == 'workflow_dispatch'" in condition
    assert "pull_request" not in condition
    steps = deploy["steps"]
    assert len(steps) == 1
    action = steps[0]["uses"]
    assert action.startswith("actions/deploy-pages@")
    assert SHA_ACTION.match(action)
    assert steps[0]["id"] == "deployment"
