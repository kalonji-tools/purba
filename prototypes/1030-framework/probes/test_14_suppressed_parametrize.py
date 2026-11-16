import pytest

import shop


# purba: suppress[foreign-decorator] cases wait for case()
@pytest.mark.parametrize("n", [1, 2])
def test_price(n=3):
    got = shop.price(n)
    assert got == 6
