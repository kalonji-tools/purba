import lending
import pytest


def test_mp_needs_a_value():
    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(lending, "Library")
        got = lending.borrow("Dune")
    assert got == "lent Dune"
