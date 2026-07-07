from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from types import ModuleType

ROOT = Path(__file__).parents[2]
CHECKER = ROOT / "scripts/check_docs_live.py"
FIXTURES = ROOT / "tests/docs/fixtures/health"


def _module() -> ModuleType:
    spec = importlib.util.spec_from_file_location("check_docs_live", CHECKER)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_route_urls_preserve_project_subpath_and_file_routes() -> None:
    checker = _module()
    assert checker.route_url("index.html") == "https://burritothief.github.io/btpc/"
    assert checker.route_url("guides/index.html").endswith("/btpc/guides/")
    assert checker.route_url("guides/creating.html").endswith(
        "/btpc/guides/creating.html"
    )


def test_live_route_validation_accepts_one_document_redirect() -> None:
    checker = _module()
    source_url = checker.route_url("cli/reference/create/index.html")
    target_url = checker.route_url("cli/reference/create.html")
    bodies = {
        source_url: (
            '<meta name="btpc-redirect" content="/btpc/cli/reference/create.html">'
        ),
        target_url: '<h1 id="create">Create</h1>',
    }
    responses = {
        url: checker.Response(200, url, "text/html; charset=utf-8", body)
        for url, body in bodies.items()
    }
    assert checker.validate_route(source_url, bodies, responses) == []


def test_live_route_validation_rejects_redirect_loop_fixture() -> None:
    checker = _module()
    source_url = checker.route_url("loop/index.html")
    target_url = checker.route_url("loop.html")
    source = (FIXTURES / "redirect-loop.html").read_text()
    bodies = {source_url: source, target_url: source}
    responses = {
        url: checker.Response(200, url, "text/html; charset=utf-8", body)
        for url, body in bodies.items()
    }
    errors = checker.validate_route(source_url, bodies, responses)
    assert any("redirect loop" in error for error in errors)
