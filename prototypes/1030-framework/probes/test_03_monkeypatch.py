import pytest

import shop


def test_price_patched():
    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(shop, "RATE", 10)
        got = shop.price(3)
    assert got == 30
