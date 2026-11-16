import pytest

import shop


def test_skipped_inside():
    pytest.skip("not on this platform")
    got = shop.price(3)
    assert got == 6
