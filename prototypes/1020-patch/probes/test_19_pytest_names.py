import pytest


def test_pytest_raises():
    with pytest.raises(ValueError):
        int("x")


def test_pytest_approx():
    got = 0.1 + 0.2
    assert got == pytest.approx(0.3)
