# The module catches the panic of purba's assert hook while it loads.
import os

os.environ["PURBA_FAULT"] = "assert"
try:
    assert True
except BaseException:
    pass
del os.environ["PURBA_FAULT"]


def test_one():
    assert True


def test_two():
    assert True
