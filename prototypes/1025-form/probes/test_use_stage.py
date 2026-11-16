"""An arrange and an act as a checked stage, taken as a default input with `use(stage)`."""

from __future__ import annotations

from dataclasses import dataclass

from cases import case, use
from shop import Shop

# ── the stages ──────────────────────────


@dataclass
class Signed:
    shop: Shop
    user: str


@dataclass
class Filled:
    shop: Shop
    user: str


def test_sign_up(name: str, shop: Shop = use(Shop)) -> Signed:
    """PASS: a stage checks its own act and hands on its product."""
    user = shop.sign_up(name)
    assert shop.users == [name]
    return Signed(shop, user)


ann = case(test_sign_up, name="ann")
bob = case(test_sign_up, name="bob")


def test_fill(signed: Signed, sku: str, qty: int, lines: int) -> Filled:
    """ann_books PASS, wrong FAIL: a stage on a stage, by a chain line."""
    signed.shop.add(signed.user, sku, qty)
    assert len(signed.shop.carts[signed.user]) == lines
    return Filled(signed.shop, signed.user)


ann_books = case(test_fill, signed=ann.product, sku="BOOK", qty=2, lines=1)
wrong = case(test_fill, signed=ann.product, sku="BOOK", qty=1, lines=9)


def test_signed_quietly(name: str, shop: Shop = use(Shop)) -> Signed:
    """REFUSED: a stage with no check, refused while it runs; each test that uses it is
    blocked."""
    user = shop.sign_up(name)
    return Signed(shop, user)


quiet = case(test_signed_quietly, name="cy")


def test_sign_up_only(shop: Shop = use(Shop)) -> None:
    """PASS: a test that returns nothing; it cannot be a stage."""
    shop.sign_up("dee")
    assert shop.users == ["dee"]


# ── use(stage): a later test reuses a checked stage with no case line ───────


def test_second_sign_up(signed: Signed = use(ann)) -> None:
    """PASS: `ann` runs first, with its check, then bob signs up on the same shop."""
    signed.shop.sign_up("bob")
    assert signed.shop.users == ["ann", "bob"]


def test_add_one(signed: Signed = use(ann)) -> None:
    """PASS: a second test on the same stage forks from the same act, in its own process."""
    signed.shop.add(signed.user, "PEN", 1)
    assert signed.shop.carts["ann"] == [("PEN", 1)]


# ── a stage on a stage, through use(): the chain runs from the first stage ─────


def test_checkout(filled: Filled = use(ann_books)) -> None:
    """PASS: ann, then ann_books, each with its checks, then the checkout."""
    order = filled.shop.checkout(filled.user)
    assert order.total == 2400


# ── a stage that failed blocks the test, and names it ────────────


def test_after_wrong(filled: Filled = use(wrong)) -> None:
    """BLOCKED. Says "it takes the product of wrong (test_use_stage.py:44), which failed" """
    order = filled.shop.checkout(filled.user)
    assert order.total == 1200


# ── a stage that was refused blocks the test, and names it ───────────


def test_after_quiet(signed: Signed = use(quiet)) -> None:
    """BLOCKED. Says "it takes the product of quiet (test_use_stage.py:54), which was
    refused" """
    signed.shop.add(signed.user, "BOOK", 1)
    assert signed.shop.carts["cy"] == [("BOOK", 1)]


# ── a stage whose test returns nothing is refused before the run ─────────

dee = case(test_sign_up_only)


def test_after_dee(shop: Shop = use(dee)) -> None:
    """REFUSED: test_sign_up_only returns nothing, so it is no stage."""
    shop.sign_up("eve")
    assert shop.users == ["dee", "eve"]


# ── fan-in: two stages as two defaults ─────────────────


def test_two_shops(left: Signed = use(ann), right: Signed = use(bob)) -> None:
    """PASS: two stages, two shops; a line in one does not reach the other."""
    left.shop.add(left.user, "PEN", 1)
    assert right.shop.carts["bob"] == []


# ── a fixture may act ──────


def shop_with_ann() -> Shop:
    shop = Shop()
    shop.sign_up("ann")
    return shop


def test_fixture_that_acts(shop: Shop = use(shop_with_ann)) -> None:
    """PASS: a fixture may act, and no check reads what its act did."""
    shop.sign_up("bob")
    assert shop.users == ["ann", "bob"]


def wrapped_shop_with_ann() -> Shop:
    return shop_with_ann()


def test_fixture_that_acts_in_one_call(shop: Shop = use(wrapped_shop_with_ann)) -> None:
    """PASS: a fixture of one call that acts."""
    shop.sign_up("bob")
    assert shop.users == ["ann", "bob"]


def test_fixture_of_one_call(shop: Shop = use(Shop)) -> None:
    """PASS: a fixture that builds one object."""
    shop.sign_up("bob")
    assert shop.users == ["bob"]
