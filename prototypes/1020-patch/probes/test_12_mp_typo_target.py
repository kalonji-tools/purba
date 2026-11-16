import lending
import pytest


def test_mp_typo_target():
    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(lending, "Librarian", object)
        got = lending.borrow("Dune")
    assert got == "lent Dune"
