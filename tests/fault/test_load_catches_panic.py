# The module catches the panic of purba's assert hook while it loads.
import os

os.environ["PURBA_FAULT"] = "assert"
try:
    assert 1 == 1
except BaseException:
    pass
del os.environ["PURBA_FAULT"]


def test_one():
    assert 1 == 1


def test_two():
    assert 1 == 1
