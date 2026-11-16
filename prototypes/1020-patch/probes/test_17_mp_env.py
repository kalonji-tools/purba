import os

import pytest


def test_mp_setenv():
    with pytest.MonkeyPatch.context() as mp:
        mp.setenv("EDITOR", "vi")
        got = os.environ["EDITOR"]
    assert got == "vi"
