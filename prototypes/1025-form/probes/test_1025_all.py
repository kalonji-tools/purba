"""#1025, change 1: `assert all(<check> for <target> in <reading>)` is a check. The runner reads
every item, in order, and checks each one; a failure names every item that breaks it, with its
index. Any other `all(…)` stays refused, and the refusal names the allowed shape."""

from __future__ import annotations

from cases import case
from lazy import chunks, tag
from shop import Shop


def test_gt_holds() -> None:
    """PASS: every total is above 0. Says "3 of 3 items hold" """
    shop = Shop()
    totals = [shop.prices["BOOK"], shop.prices["PEN"], 7]
    assert all(t > 0 for t in totals)


def test_gt_fails() -> None:
    """FAIL: two items break it, and each is named with its index. Says "2 of 5 items break  x > 0"
    Says "[2]  -3" Says "[3]  0" """
    xs = [9, 4, -3, 0, 1]
    assert all(x > 0 for x in xs)


def test_is_holds() -> None:
    """PASS: each number's category is None."""
    words = tag("1 22 333")
    assert all(w.category is None for w in words)


def test_is_fails() -> None:
    """FAIL: a computed reading shows the reading and the item, both. Says "1 of 3 items break
    w.category is None" Says "[1]  w.category  <class 'str'>     w  Word('ab')" """
    words = tag("1 ab 3")
    assert all(w.category is None for w in words)


def test_isinstance_holds() -> None:
    """PASS: each user is a str."""
    shop = Shop()
    shop.sign_up("ann")
    shop.sign_up("bob")
    assert all(isinstance(u, str) for u in shop.users)


def test_isinstance_fails() -> None:
    """FAIL: Says "1 of 2 items break  isinstance(c, int)" Says "[1]  'ab'" """
    parts = [1, *chunks("ab", 2)]
    assert all(isinstance(c, int) for c in parts)


def test_startswith_holds(prefix: str) -> None:
    """PASS: the expected value is an input that takes a literal in every case."""
    shop = Shop()
    shop.sign_up("ann-lee")
    shop.sign_up("ann-bo")
    assert all(u.startswith(prefix) for u in shop.users)


ann = case(test_startswith_holds, prefix="ann-")


def test_computed_reading_fails() -> None:
    """FAIL: Says "1 of 3 items break  len(c) == 2" Says "[2]  len(c)  1     c  'e'" """
    parts = chunks("abcde", 2)
    assert all(len(c) == 2 for c in parts), "each chunk holds two letters"


def test_tuple_target_holds() -> None:
    """PASS: a tuple target binds each pair."""
    pairs = list(zip(chunks("abcd", 1), chunks("bcde", 1)))
    assert all(len(a + b) == 2 for a, b in pairs)


def test_tuple_target_fails() -> None:
    """FAIL: the reading `a` is not the item, so both show. Says "[1]  a  'x'     (a, b)  ('x',
    'c')" """
    pairs = list(zip(chunks("axc", 1), chunks("bcd", 1)))
    assert all(a != "x" for a, b in pairs)


def test_tuple_target_compares_items() -> None:
    """REFUSED before the run: `b` is no expected value, as in the 1015 form. Says "the expected
    value computes `b`" """
    pairs = list(zip(chunks("abcd", 1), chunks("bcde", 1)))
    assert all(a < b for a, b in pairs)


def test_nested_comprehension() -> None:
    """PASS: a list comprehension inside the reading (pytest 4.6.0 broke on it)."""
    parts = chunks("abcde", 2)
    assert all(n > 0 for n in [len(c) for c in parts])


def test_empty_holds() -> None:
    """PASS: an empty iterable holds, as in Python. Says "0 of 0 items hold" """
    shop = Shop()
    assert all(o.total > 0 for o in shop.orders)


def test_with_if() -> None:
    """REFUSED before the run, with the allowed shape. Says "`all(…)` has an `if`" Says
    "`assert all(<check> for <target> in <reading>)` (or `any`) is a check" """
    parts = chunks("abcde", 2)
    assert all(len(c) == 2 for c in parts if c)


def test_two_fors() -> None:
    """REFUSED before the run. Says "`all(…)` has 2 `for`s" """
    parts = chunks("abcde", 2)
    assert all(ch > "" for c in parts for ch in c)


def test_list_comprehension() -> None:
    """REFUSED before the run. Says "`all(…)` holds a list comprehension" """
    parts = chunks("abcde", 2)
    assert all([len(c) > 0 for c in parts])


def test_body_no_check() -> None:
    """REFUSED before the run. Says "`c` is no check" """
    parts = chunks("abcde", 2)
    assert all(c for c in parts)


def test_class_on_the_right() -> None:
    """REFUSED before the run: a class name on the right of `is` is computed, as in the 1015 form.
    Says "the expected value computes `ZeroDivisionError`" """
    kinds = [type(e) for e in [ZeroDivisionError()]]
    assert all(t is ZeroDivisionError for t in kinds)


def test_computed_expected() -> None:
    """REFUSED before the run: the expected value of each item is computed. Says "the expected value
    computes `len(parts)`" """
    parts = chunks("abcde", 2)
    assert all(len(c) <= len(parts) for c in parts)


def test_inside_a_larger_expression() -> None:
    """PASS: `all(…) == True` is no `all()` check; it stays a comparison, as in the 1015 form."""
    parts = chunks("abcde", 2)
    assert all(len(c) > 0 for c in parts) == True  # noqa: E712


def positive(n: int) -> bool:
    assert n > 0, "a helper that checks"
    return True


def test_assert_in_the_reading() -> None:
    """REFUSED: an assert runs inside the reading. Says "an assert ran inside the reading of the
    check" """
    parts = chunks("abcde", 2)
    assert all(positive(len(c)) is True for c in parts)


def test_read_alone() -> None:
    """REFUSED: the second check pops the list, so the third holds alone and fails after it. Says
    "it holds when read alone, and fails after the checks before it" """
    stack = chunks("abc", 1)
    assert all(c > "" for c in stack)
    assert all(stack.pop() > "" for _ in [0])
    assert len(stack) == 3
