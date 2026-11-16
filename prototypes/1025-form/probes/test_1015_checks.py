"""#1015, change 1: besides one comparison, a check may be `assert not reading`,
`assert isinstance(reading, T)`, `assert reading.startswith(x)` or `assert reading.endswith(x)`.
Each has a report of its own. Any other assert stays refused, and the refusal lists the shapes."""

from __future__ import annotations

import collections.abc

from cases import case
from shop import Order, Shop


def test_not_holds() -> None:
    """PASS: the cart is empty after the checkout."""
    shop = Shop()
    shop.sign_up("ann")
    shop.add("ann", "PEN", 1)
    shop.checkout("ann")
    assert not shop.carts["ann"]


def test_not_fails() -> None:
    """FAIL: the cart still holds the pen. Says "expected  empty or false" Says "reading   [('PEN',
    1)] (list)" """
    shop = Shop()
    shop.sign_up("ann")
    shop.add("ann", "PEN", 1)
    assert not shop.carts["ann"], "the cart is empty"


def test_isinstance_holds() -> None:
    """PASS: a name, a dotted name, and a tuple of them."""
    shop = Shop()
    shop.sign_up("ann")
    order = shop.checkout("ann")
    assert isinstance(order, Order)
    assert isinstance(shop.carts, collections.abc.Mapping)
    assert isinstance(order.total, (int, float))


def test_isinstance_fails() -> None:
    """FAIL: the total is an int. Says "expected  an instance of str" Says "reading   0 (int)" """
    shop = Shop()
    shop.sign_up("ann")
    order = shop.checkout("ann")
    assert isinstance(order.total, str)


def test_affixes_hold(prefix: str) -> None:
    """PASS: the expected value is a literal, or an input that takes a literal in every case."""
    shop = Shop()
    shop.sign_up("ann-lee")
    assert shop.users[0].startswith(prefix)
    assert shop.users[0].endswith("-lee")


ann = case(test_affixes_hold, prefix="ann")


def test_affixes_fail() -> None:
    """FAIL: both checks run, and each failure says what it expected. Says "expected  to start
    with 'bob'" Says "expected  to end with 'smith'" """
    shop = Shop()
    shop.sign_up("ann-lee")
    assert shop.users[0].startswith("bob")
    assert shop.users[0].endswith("smith")


def test_affix_of_no_str() -> None:
    """FAIL: a list has no `startswith`, so reading it raised. Says "reading it raised
    AttributeError" """
    shop = Shop()
    shop.sign_up("ann")
    assert shop.users.startswith("a")  # ty: ignore[unresolved-attribute]


def test_bare_assert() -> None:
    """REFUSED before the run, with the allowed shapes. Says "or it is `assert not reading`,
    `assert isinstance(reading, T)`, `assert reading.startswith(x)` or `assert
    reading.endswith(x)`" """
    shop = Shop()
    shop.sign_up("ann")
    assert shop.users


def test_chained_comparison() -> None:
    """REFUSED before the run: a chained comparison is a check in another form. Says "line 91: a
    check compares a reading" """
    shop = Shop()
    order = shop.checkout(shop.sign_up("ann"))
    assert 0 <= order.total < 10


def test_other_call() -> None:
    """PASS: since #1025, `any(<check> for <target> in <reading>)` is a check (it was refused in the
    1015 form)."""
    shop = Shop()
    shop.sign_up("ann")
    assert any(u == "ann" for u in shop.users)


def test_computed_affix() -> None:
    """REFUSED before the run: the expected value of `startswith` is computed. Says "the expected
    value computes `'ann'.upper()`" """
    shop = Shop()
    shop.sign_up("ANN")
    assert shop.users[0].startswith("ann".upper())


def test_isinstance_of_a_value() -> None:
    """REFUSED before the run: the T of `isinstance` is bound to no class. Says "`kind` is bound
    to no class at module level" """
    shop = Shop()
    kind = Order
    order = shop.checkout(shop.sign_up("ann"))
    assert isinstance(order, kind)


def test_not_reads_nothing() -> None:
    """REFUSED before the run: `assert not` reads no value. Says "`[]` reads no value" """
    Shop().sign_up("ann")
    assert not []
