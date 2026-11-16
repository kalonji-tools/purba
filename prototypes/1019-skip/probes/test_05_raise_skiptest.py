import unittest


def test_raise_skiptest():
    raise unittest.SkipTest("not here")
    total = 1 + 1
    assert total == 2
