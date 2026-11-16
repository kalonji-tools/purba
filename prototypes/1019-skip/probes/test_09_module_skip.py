import pytest

pytest.skip("module off", allow_module_level=True)


def test_module_skip():
    total = 1 + 1
    assert total == 2
