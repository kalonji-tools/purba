"""#980 round 3, Q1: a case's name is read from the module after import. A case that no name holds, or that two names
hold, is refused."""

from __future__ import annotations

from cases import case


def test_lower(text: str):
    """`plain` and the second `twice` PASS; every other case of it is REFUSED."""
    low = text.lower()
    assert low.isupper() is False


plain = case(test_lower, text="a")
twice = case(test_lower, text="b")  # REFUSED: bound again on the next line
twice = case(test_lower, text="c")
grid = [case(test_lower, text=t) for t in "de"]  # REFUSED, both: built by a loop
first = second = case(test_lower, text="f")  # REFUSED: two names
case(test_lower, text="g")  # REFUSED: never bound
