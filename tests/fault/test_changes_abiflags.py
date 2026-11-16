import sys


# purba: suppress[patched-name] the report must ignore a sys.abiflags that a test changed
def test_one():
    sys.abiflags = "x"
    assert 1 == 1
