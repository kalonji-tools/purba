"""An assert that runs inside a reading is refused: a helper that a check's reading calls returns
what it reads, and the check compares it."""

from __future__ import annotations

from cases import use
from helpers import total_is
from shop import Order, Shop


def check_paid(order: Order) -> None:
    assert order.state == "paid"


def state_of(order: Order) -> str:
    return order.state


def test_assert_in_a_reading(shop: Shop = use(Shop)) -> None:
    """REFUSED: `check_paid` asserts inside the reading, and its assert holds. Says "an assert ran
    inside the reading of the check at line 24, at test_reading_assert.py:12 (check_paid, reached
    from line 24 of the test)" """
    order = shop.checkout(shop.sign_up("ann"))
    assert check_paid(order) is None


def test_failing_assert_in_a_reading(shop: Shop = use(Shop)) -> None:
    """REFUSED, rather than failed: the helper's assert fails, and a refusal outranks a failure.
    Says "an assert ran inside the reading of the check at line 31" """
    order = shop.refund(shop.checkout(shop.sign_up("ann")))
    assert check_paid(order) is None


def test_helper_that_returns(shop: Shop = use(Shop)) -> None:
    """PASS: the helper returns what it reads, and the check compares it."""
    order = shop.checkout(shop.sign_up("ann"))
    assert state_of(order) == "paid"


def test_asserting_helper_of_another_module(shop: Shop = use(Shop)) -> None:
    """PASS, a hole: `total_is` asserts inside the reading, but it sits in helpers.py, which the
    watch does not cover."""
    order = shop.checkout(shop.sign_up("ann"))
    assert total_is(order, 0) is None
