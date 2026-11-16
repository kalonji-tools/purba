import lending
import pytest


class Stub:
    def lend(self, title):
        return "stub"


def test_mp_context_stub():
    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(lending, "Library", Stub)
        got = lending.borrow("Dune")
    assert got == "stub"
