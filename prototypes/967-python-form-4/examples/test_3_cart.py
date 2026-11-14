"""3: a stateful API in one act, and a chain: a later case starts from the product of an earlier case."""

from app import Cart, open_cart
from cases import case, raises


def test_checkout(owner: str, sku: str, qty: int, total: int, lines: int) -> None:
    cart = open_cart(owner)
    cart.add(sku, qty)
    cart.add("PEN-7", 1)
    receipt = cart.checkout()
    assert receipt.total == total, "the total of the receipt, in cents"
    assert receipt.lines == lines


two_books = case(test_checkout, owner="ann", sku="BOOK-1", qty=2, total=2550, lines=2)
pens      = case(test_checkout, owner="ann", sku="PEN-7", qty=3, total=600, lines=1)


def test_unknown_sku() -> None:
    cart = open_cart("ann")
    with raises(KeyError, match="NOPE"):
        cart.add("NOPE", 1)


# A chain: a test returns its product, and a later case takes it. The earlier case runs again first, once per case.
def test_open(owner: str, count: int) -> Cart:
    cart = open_cart(owner)
    assert cart.count() == count
    return cart


def test_fill(cart: Cart, again: Cart, total: int) -> None:
    cart.add("BOOK-1", 1)
    again.add("BOOK-1", 1)
    assert cart.total() == total, "both inputs are the one cart of the step"


ann        = case(test_open, owner="ann", count=0)
both_ann   = case(test_fill, cart=ann.product, again=ann.product, total=2400)
bob        = case(test_open, owner="bob", count=1)  # a wrong expected value, so the chain below is blocked
from_wrong = case(test_fill, cart=bob.product, again=bob.product, total=2400)
