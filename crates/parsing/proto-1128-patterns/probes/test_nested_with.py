import contextlib


def test_one():
    with contextlib.nullcontext():
        with contextlib.nullcontext():
            got, want = 1, 1
            assert got == want
