import pytest


@pytest.mark.xfail(reason="known bug")
def test_decorator_xfail():
    total = 1 + 1
    assert total == 3
