"""No rule refuses a second act."""

from __future__ import annotations

from cases import use
from shop import Shop


def test_sign_up_add_and_check_out(shop: Shop = use(Shop)) -> None:
    """PASS: three calls of the code under test in one act."""
    shop.sign_up("ann")
    shop.add("ann", "BOOK", 1)
    order = shop.checkout("ann")
    assert order.total == 1200
    assert shop.carts["ann"] == []
