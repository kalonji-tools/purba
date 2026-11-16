"""#1025, change 2: `assert any(<check> for <target> in <reading>)` is a check, with the shape rules
of `all`. It holds when one item holds; a failure says that no item holds it, and lists every
item."""

from __future__ import annotations

from lazy import chunks
from shop import Shop


def test_holds() -> None:
    """PASS: one user is ann. Says "1 of 2 items hold" """
    shop = Shop()
    shop.sign_up("bob")
    shop.sign_up("ann")
    assert any(u == "ann" for u in shop.users)


def test_fails() -> None:
    """FAIL: no item holds, and each one is listed. Says "none of 3 items holds  x > 5" Says "[0]
    1" Says "[2]  3" """
    xs = [1, 2, 3]
    assert any(x > 5 for x in xs)


def test_computed_reading_fails() -> None:
    """FAIL: the reading and the item, both. Says "none of 2 items holds  len(c) == 3" Says "[0]
    len(c)  2     c  'ab'" Says "[1]  len(c)  1     c  'c'" """
    parts = chunks("abc", 2)
    assert any(len(c) == 3 for c in parts)


def test_empty_fails() -> None:
    """FAIL: an empty iterable holds no item, as in Python. Says "none of 0 items holds" """
    shop = Shop()
    assert any(o.total > 0 for o in shop.orders)


def test_with_if() -> None:
    """REFUSED before the run. Says "`any(…)` has an `if`" """
    parts = chunks("abc", 2)
    assert any(len(c) == 3 for c in parts if c)
