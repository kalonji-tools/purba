import lending
import pytest


class Stub:
    def lend(self, title):
        return "stub"


def test_under_then_after_the_undo():
    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(lending, "Library", Stub)
        got = lending.borrow("Dune")
        assert got == "stub"
    assert lending.borrow("Dune") == "lent Dune"


def test_after_undo_wrong_on_purpose():
    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(lending, "Library", Stub)
        got = lending.borrow("Dune")
        assert got == "stub"
    assert lending.borrow("Dune") == "stub"
