import sys


def test_one():
    sys.abiflags = "x"
    assert 1 == 1
