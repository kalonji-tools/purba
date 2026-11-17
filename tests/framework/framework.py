import importlib
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import pytest


def price(n):
    return n * 2


def helper():
    from _pytest.monkeypatch import MonkeyPatch
    return MonkeyPatch


def test_price():
    got = price(3)
    assert got == 6


def test_call():
    importlib.import_module("pytest")
    got = price(5)
    assert got == 10


# purba: suppress[foreign-framework] this test still runs under pytest
def test_suppressed():
    got = price(4)
    assert got == 8
