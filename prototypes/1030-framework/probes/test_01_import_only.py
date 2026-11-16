import pytest

import shop


def test_price():
    got = shop.price(3)
    assert got == 6
