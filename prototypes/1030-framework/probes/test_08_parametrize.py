import pytest

import shop


@pytest.mark.parametrize("n", [1, 2])
def test_price(n):
    got = shop.price(n)
    assert got == 2
