from unittest import mock

import shop


def test_price_patched():
    with mock.patch.object(shop, "RATE", 10):
        got = shop.price(3)
    assert got == 30
