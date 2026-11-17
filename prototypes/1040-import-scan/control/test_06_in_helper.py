import shop


def helper():
    import pytest
    return pytest


def test_price():
    got = shop.price(3)
    assert got == 6
