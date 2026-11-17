import contextlib


def test_refused():
    with contextlib.suppress(KeyError):
        {}["k"]
        print("never")
    assert 1 == 1
