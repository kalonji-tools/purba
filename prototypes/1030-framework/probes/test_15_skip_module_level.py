import pytest

pytest.skip("whole module off", allow_module_level=True)


def test_never():
    got = 1 + 1
    assert got == 2
