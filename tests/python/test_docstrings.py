from __future__ import annotations

import doctest
import inspect
from dataclasses import fields, is_dataclass

import btpc
from btpc import creation, errors, metainfo, types, verification

PUBLIC_MODULES = (creation, errors, metainfo, types, verification)
HIGH_USE = (
    creation.CreateOptions,
    creation.CancellationToken,
    creation.create,
    creation.create_bytes,
    metainfo.Metainfo,
    metainfo.Metainfo.from_bytes,
    metainfo.Metainfo.read,
    metainfo.Metainfo.edit,
    metainfo.Metainfo.verify,
    metainfo.Metainfo.magnet,
    metainfo.Metainfo.to_bytes,
    verification.verify,
    types.ParseOptions,
)


# Spec: PYAPI-DOCSTRING-001
def _doc(value: object) -> str:
    return inspect.getdoc(value) or ""


def _public_members(cls: type[object]) -> list[tuple[str, object]]:
    return [
        (name, value)
        for name, value in vars(cls).items()
        if not name.startswith("_")
        and (inspect.isfunction(value) or isinstance(value, property))
    ]


def test_public_modules_and_exports_have_summary_docstrings() -> None:
    for module in PUBLIC_MODULES:
        assert _doc(module), module.__name__
        for name in module.__all__:
            value = getattr(module, name)
            assert _doc(value), f"{module.__name__}.{name}"


def test_public_methods_and_properties_have_summary_docstrings() -> None:
    for module in PUBLIC_MODULES:
        for name in module.__all__:
            value = getattr(module, name)
            if not inspect.isclass(value):
                continue
            for member_name, member in _public_members(value):
                assert _doc(member), f"{value.__qualname__}.{member_name}"


def test_dataclass_fields_are_described_by_the_class_docstring() -> None:
    for module in PUBLIC_MODULES:
        for name in module.__all__:
            value = getattr(module, name)
            if not inspect.isclass(value) or not is_dataclass(value):
                continue
            doc = _doc(value)
            assert "Attributes:" in doc, value.__qualname__
            for field in fields(value):
                assert f"{field.name}:" in doc, f"{value.__qualname__}.{field.name}"


def test_high_use_apis_have_structured_docs_and_examples() -> None:
    for value in HIGH_USE:
        doc = _doc(value)
        assert "Examples:" in doc, value.__qualname__
        assert ">>>" in doc, value.__qualname__

    for value in (creation.create, creation.create_bytes, verification.verify):
        doc = _doc(value)
        for section in ("Args:", "Returns:", "Raises:"):
            assert section in doc, f"{value.__qualname__}: {section}"

    assert "Attributes:" in _doc(creation.CreateOptions)
    assert "Attributes:" in _doc(types.ParseOptions)
    assert "completed_bytes" in _doc(creation.create_bytes)
    assert "total_bytes" in _doc(creation.create_bytes)
    assert "completed_pieces" in _doc(creation.create_bytes)
    assert "UNCHANGED" in _doc(metainfo.Metainfo.edit)
    assert "canonical" in _doc(metainfo.Metainfo.to_bytes).lower()
    assert "original" in _doc(metainfo.Metainfo)


def test_root_reexports_preserve_identity_and_docstrings() -> None:
    for module in PUBLIC_MODULES:
        for name in module.__all__:
            if name in btpc.__all__:
                assert getattr(btpc, name) is getattr(module, name)
                assert _doc(getattr(btpc, name)) == _doc(getattr(module, name))


def test_docstring_examples_execute() -> None:
    failures = 0
    for module in PUBLIC_MODULES:
        failures += doctest.testmod(module, optionflags=doctest.ELLIPSIS).failed
    assert failures == 0
