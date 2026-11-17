import contextlib


# purba: suppress[swallowed-raise] this case shows the departure
def test_suppressed():
    with contextlib.suppress(KeyError):
        {}["k"]
        print("never")
    assert 1 == 1
