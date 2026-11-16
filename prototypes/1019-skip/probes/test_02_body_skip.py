import pytest


def test_body_skip():
    pytest.skip("not here")
    total = 1 + 1
    assert total == 2
