import shop


def test_price():
    import pytest
    got = shop.price(3)
    assert got == 6


def test_other():
    got = shop.price(4)
    assert got == 8
