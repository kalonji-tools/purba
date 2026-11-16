import pytest

import shop


def test_fail_inside():
    got = shop.price(3)
    if got != 6:
        pytest.fail("wrong price")
    assert got == 6
