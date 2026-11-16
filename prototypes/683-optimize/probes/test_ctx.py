from contextlib import contextmanager

@contextmanager
def opened():
    assert False, "inside"
    yield 1

def test_ctx():
    with opened() as v:
        got = v
    assert got == 1
