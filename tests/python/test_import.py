"""Import smoke tests for the mixed Python/Rust package."""

import subprocess
import sys
from importlib.metadata import version

import btpc
import btpc._native as native
import btpc.creation
import btpc.errors
import btpc.metainfo
import btpc.types
import btpc.verification
import pytest


# Spec: PYAPI-PACKAGE-001
def test_public_and_native_versions_match() -> None:
    """The public package should expose the native crate version."""
    assert btpc.__version__ == native.__version__
    assert btpc.__version__ == version("btpc")


def test_native_runtime_policy_is_explicit() -> None:
    assert native.__gil_required__ is True
    assert native.__subinterpreters_supported__ is False


# Spec: PYAPI-MODULES-001
def test_public_domain_modules_are_canonical_and_root_reexports_preserve_identity() -> (
    None
):
    identities = {
        btpc.errors: [
            "BtpcError",
            "BencodeError",
            "MetainfoError",
            "PathError",
            "ResourceLimitError",
            "UnsupportedError",
            "VerificationError",
            "CancelledError",
        ],
        btpc.types: [
            "HashValue",
            "ParseOptions",
            "TorrentBytes",
            "TorrentMode",
            "TorrentPath",
            "UNCHANGED",
        ],
        btpc.metainfo: ["Metainfo", "TorrentFile", "ValidationReport"],
        btpc.creation: [
            "CancellationToken",
            "CreateMetrics",
            "CreateOptions",
            "CreateResult",
            "create",
            "create_bytes",
        ],
        btpc.verification: [
            "MismatchKind",
            "PayloadMismatch",
            "PayloadVerificationReport",
            "verify",
        ],
    }
    for module, names in identities.items():
        for name in names:
            value = getattr(module, name)
            assert getattr(btpc, name) is value
            assert value.__module__ == module.__name__
    assert "_native" not in btpc.__all__
    assert "_conversion" not in btpc.__all__


def test_public_modules_import_independently() -> None:
    modules = ["errors", "types", "creation", "verification", "metainfo"]
    for order in (modules, list(reversed(modules))):
        script = "; ".join(f"import btpc.{name}" for name in order)
        subprocess.run([sys.executable, "-c", script], check=True)  # noqa: S603


@pytest.mark.skipif(
    sys.version_info < (3, 14),
    reason="rejection is enforceable through the supported CPython 3.14 API",
)
def test_second_subinterpreter_import_is_rejected() -> None:
    script = """
import btpc._native
try:
    import concurrent.interpreters as interpreters
except ImportError:
    import _xxsubinterpreters as interpreters
    interpreter = interpreters.create()
    try:
        interpreters.run_string(interpreter, "import btpc._native")
    except interpreters.RunFailedError as error:
        assert "subinterpreter" in str(error)
    else:
        raise AssertionError("btpc._native unexpectedly imported in a subinterpreter")
    finally:
        interpreters.destroy(interpreter)
else:
    interpreter = interpreters.create()
    try:
        interpreter.exec("import btpc._native")
    except interpreters.ExecutionFailed as error:
        assert "subinterpreter" in str(error)
    else:
        raise AssertionError("btpc._native unexpectedly imported in a subinterpreter")
    finally:
        interpreter.close()
"""
    subprocess.run([sys.executable, "-c", script], check=True)  # noqa: S603
