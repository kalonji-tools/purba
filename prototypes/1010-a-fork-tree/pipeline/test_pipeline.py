"""#1010: how far the form of prototype 980 carries a reused arrange.

And an arrange + act that feeds a later test.
"""

from __future__ import annotations

from dataclasses import dataclass

from cases import case, use
from shop import Order, Shop


# ── P1 a shared arrange: one fixture, many tests ───────────────

def test_empty_shop_signs_up(shop: Shop = use(Shop)) -> None:
    shop.sign_up("ann")
    assert shop.users == ["ann"]


def test_empty_shop_has_no_orders(shop: Shop = use(Shop)) -> None:
    shop.carts["x"] = []
    assert shop.orders == []


# ── P2 an arrange that is itself an arrange + act, as a fixture: no check sees it ────

def shop_with_ann() -> Shop:
    shop = Shop()
    shop.sign_up("ann")
    return shop


def test_second_sign_up(shop: Shop = use(shop_with_ann)) -> None:
    shop.sign_up("bob")
    assert shop.users == ["ann", "bob"]


# ── P3 a pipeline: each stage is a test, checks its own state, hands on a product ───

@dataclass
class Signed:
    shop: Shop
    user: str


@dataclass
class Filled:
    shop: Shop
    user: str


@dataclass
class Paid:
    shop: Shop
    order: Order


def test_sign_up(name: str, shop: Shop = use(Shop)) -> Signed:
    user = shop.sign_up(name)
    assert shop.users == [name]
    return Signed(shop, user)


def test_fill(signed: Signed, sku: str, qty: int, lines: int) -> Filled:
    signed.shop.add(signed.user, sku, qty)
    assert len(signed.shop.carts[signed.user]) == lines
    return Filled(signed.shop, signed.user)


def test_checkout(filled: Filled, total: int) -> Paid:
    order = filled.shop.checkout(filled.user)
    assert order.total == total
    assert filled.shop.carts[filled.user] == []
    return Paid(filled.shop, order)


def test_refund(paid: Paid) -> None:
    paid.shop.refund(paid.order)
    assert paid.order.state == "refunded"


ann       = case(test_sign_up, name="ann")
ann_books = case(test_fill, signed=ann.product, sku="BOOK", qty=2, lines=1)
ann_paid  = case(test_checkout, filled=ann_books.product, total=2400)
ann_back  = case(test_refund, paid=ann_paid.product)

# ── P4 fan-out: two branches of the pipeline from one stage ───────────

ann_pens  = case(test_fill, signed=ann.product, sku="PEN", qty=3, lines=1)
pens_paid = case(test_checkout, filled=ann_pens.product, total=600)

# ── P5 fan-in: one stage takes the products of two earlier stages ─────────


def test_two_shops(left: Signed, right: Signed) -> None:
    left.shop.add(left.user, "PEN", 1)
    assert right.shop.carts[right.user] == []


bob       = case(test_sign_up, name="bob")
two_shops = case(test_two_shops, left=ann.product, right=bob.product)

# ── P6 a stage that only arranges and acts, with no check of its own ────────


def test_signed_quietly(name: str, shop: Shop = use(Shop)) -> Signed:
    user = shop.sign_up(name)
    return Signed(shop, user)


quiet       = case(test_signed_quietly, name="cy")
quiet_books = case(test_fill, signed=quiet.product, sku="BOOK", qty=1, lines=1)

# ── P7 an earlier stage that fails blocks every later one ────────────

wrong      = case(test_fill, signed=ann.product, sku="BOOK", qty=1, lines=9)
from_wrong = case(test_checkout, filled=wrong.product, total=1200)
