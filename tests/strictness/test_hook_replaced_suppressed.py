import sys


def quiet():
    sys.unraisablehook = print


# purba: suppress[hook-replaced] this case shows the departure
def test_suppressed():
    quiet()
    assert 1 == 1
