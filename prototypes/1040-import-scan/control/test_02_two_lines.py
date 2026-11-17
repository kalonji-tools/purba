import pytest
import shop
from pytest import approx


def test_price():
    got = shop.price(3)
    assert got == 6
