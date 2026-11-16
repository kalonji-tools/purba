"""A reset written as a fixture in the signature still works; FORM.md teaches `@before` instead."""

from __future__ import annotations

from cases import use
from registry import names, register, reset


def test_register(_reset: None = use(reset)) -> None:
    """PASS: the fixture emptied the registry, which starts with "builtin", before the act."""
    register("ann")
    assert names() == ["ann"]


def test_register_without_reset() -> None:
    """FAIL: no fixture resets the registry, so "builtin" is still in it."""
    register("ann")
    assert names() == ["ann"]
