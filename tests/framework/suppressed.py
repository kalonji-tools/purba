# purba: suppress-module[foreign-framework] the suite still runs under pytest
import pytest


def price(n):
    return n * 2


def test_price():
    got = price(3)
    assert got == 6
