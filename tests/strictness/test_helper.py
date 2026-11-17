import unittest


def total():
    return 3


def check(value):
    assert value == 3


def test_unittest_helper():
    unittest.TestCase().assertEqual(total(), 3)


def test_own_helper():
    check(total())
