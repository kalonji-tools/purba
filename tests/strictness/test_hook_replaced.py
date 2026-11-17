import sys


def quiet():
    sys.unraisablehook = print


def test_refused():
    quiet()
    assert 1 == 1
