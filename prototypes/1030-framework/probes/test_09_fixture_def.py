import pytest

import shop


@pytest.fixture
def three():
    return 3


def test_price():
    got = shop.price(3)
    assert got == 6
