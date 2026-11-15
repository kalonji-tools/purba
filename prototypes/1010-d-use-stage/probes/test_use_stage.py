"""#1010 Q3: an arrange + act as a checked stage, taken as a default input with `use(stage)`. Each
probe's docstring says what the runner should do. Run with PYTHONPATH=../undertest/shop."""

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
    """A stage: it checks its own act and hands on its product. Passes."""
    user = shop.sign_up(name)
    assert shop.users == [name]
    return Signed(shop, user)


ann = case(test_sign_up, name="ann")
bob = case(test_sign_up, name="bob")


def test_fill(signed: Signed, sku: str, qty: int, lines: int) -> Filled:
    """A stage on a stage, by a chain line. `wrong` fails its own check."""
    signed.shop.add(signed.user, sku, qty)
    assert len(signed.shop.carts[signed.user]) == lines
    return Filled(signed.shop, signed.user)


ann_books = case(test_fill, signed=ann.product, sku="BOOK", qty=2, lines=1)
wrong = case(test_fill, signed=ann.product, sku="BOOK", qty=1, lines=9)


def test_signed_quietly(name: str, shop: Shop = use(Shop)) -> Signed:
    """A stage with no check: refused while it runs, so every test that uses it is blocked."""
    user = shop.sign_up(name)
    return Signed(shop, user)


quiet = case(test_signed_quietly, name="cy")


def test_sign_up_only(shop: Shop = use(Shop)) -> None:
    """A test that returns nothing: it passes, and cannot be a stage."""
    shop.sign_up("dee")
    assert shop.users == ["dee"]


# ── P1 use(stage): a later test reuses a checked stage with no case line ───────


def test_second_sign_up(signed: Signed = use(ann)) -> None:
    """Passes: `ann` runs first, with its check, then bob signs up on the same shop."""
    signed.shop.sign_up("bob")
    assert signed.shop.users == ["ann", "bob"]


def test_add_one(signed: Signed = use(ann)) -> None:
    """Passes: a second test on the same stage gets a fresh replay of it, in its own process."""
    signed.shop.add(signed.user, "PEN", 1)
    assert signed.shop.carts["ann"] == [("PEN", 1)]


# ── P2 a stage on a stage, through use(): the chain runs from the first stage ─────


def test_checkout(filled: Filled = use(ann_books)) -> None:
    """Passes: ann, then ann_books, each with its checks, then the checkout."""
    order = filled.shop.checkout(filled.user)
    assert order.total == 2400


# ── P3 a stage that failed blocks the test, and names it ────────────


def test_after_wrong(filled: Filled = use(wrong)) -> None:
    """Blocked: it takes the product of `wrong`, which failed."""
    order = filled.shop.checkout(filled.user)
    assert order.total == 1200


# ── P4 a stage that was refused blocks the test, and names it ───────────


def test_after_quiet(signed: Signed = use(quiet)) -> None:
    """Blocked: it takes the product of `quiet`, which is refused (no check ran)."""
    signed.shop.add(signed.user, "BOOK", 1)
    assert signed.shop.carts["cy"] == [("BOOK", 1)]


# ── P5 a stage whose test returns nothing is refused before the run ─────────

dee = case(test_sign_up_only)


def test_after_dee(shop: Shop = use(dee)) -> None:
    """Refused: test_sign_up_only returns nothing, so it is no stage."""
    shop.sign_up("eve")
    assert shop.users == ["dee", "eve"]


# ── P6 fan-in: two stages as two defaults ─────────────────


def test_two_shops(left: Signed = use(ann), right: Signed = use(bob)) -> None:
    """Passes: two stages, two shops; a line in one does not reach the other."""
    left.shop.add(left.user, "PEN", 1)
    assert right.shop.carts["bob"] == []


# ── P7 rule (b), with FORM_FIXTURE=one-call: a fixture that acts is refused ──────


def shop_with_ann() -> Shop:
    shop = Shop()
    shop.sign_up("ann")
    return shop


def test_fixture_that_acts(shop: Shop = use(shop_with_ann)) -> None:
    """Passes by default (rule a). Refused under FORM_FIXTURE=one-call (rule b): "make it a
    stage"."""
    shop.sign_up("bob")
    assert shop.users == ["ann", "bob"]


def wrapped_shop_with_ann() -> Shop:
    return shop_with_ann()


def test_fixture_that_acts_in_one_call(shop: Shop = use(wrapped_shop_with_ann)) -> None:
    """Passes under both rules: a hole of rule (b). It reads the fixture's own body, one call, and
    not what that call does."""
    shop.sign_up("bob")
    assert shop.users == ["ann", "bob"]


def test_fixture_of_one_call(shop: Shop = use(Shop)) -> None:
    """Passes under both rules: `use(Shop)` is one call."""
    shop.sign_up("bob")
    assert shop.users == ["bob"]
