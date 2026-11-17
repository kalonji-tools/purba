try:
    import pytest
except ImportError:
    pytest = None

import shop


def test_price():
    got = shop.price(3)
    assert got == 6
