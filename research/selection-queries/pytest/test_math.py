import pytest


def test_add():
    assert 1 + 1 == 2, "addition"


@pytest.mark.slow
def test_add_slow():
    assert 2 + 2 == 4, "addition, slow"


@pytest.mark.parametrize("case", ["a", "b"])
def test_parse(case):
    assert case in "ab", "a known case"
