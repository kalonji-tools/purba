"""A case's name is read from the module after import. A case that no name holds, or that two
names hold, is refused."""

from __future__ import annotations

from cases import case


def test_lower(text: str):
    """plain PASS, twice PASS. The first `twice`, both of `grid`, `first = second` and the unbound
    case are refused. Says "has two names, first and second" Says "2 cases of test_lower at line 20
    have no name" """
    low = text.lower()
    assert low.isupper() is False


plain = case(test_lower, text="a")
twice = case(test_lower, text="b")  # REFUSED: bound again on the next line
twice = case(test_lower, text="c")
grid = [case(test_lower, text=t) for t in "de"]  # REFUSED, both: built by a loop
first = second = case(test_lower, text="f")  # REFUSED: two names
case(test_lower, text="g")  # REFUSED: never bound
