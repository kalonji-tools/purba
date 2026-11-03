# The second test sleeps, so the reader of stdout has quit before its line.
import time


def test_one():
    assert 1 == 1


def test_two():
    time.sleep(1)
    assert 1 == 1
