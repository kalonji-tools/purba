import lending
import pytest


class Stub:
    def lendd(self, title):
        return "stub"


def test_stub_with_wrong_method():
    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(lending, "Library", Stub)
        got = lending.borrow("Dune")
    assert got == "stub"
