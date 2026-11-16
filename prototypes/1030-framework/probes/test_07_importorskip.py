import pytest

np = pytest.importorskip("no_such_module_xyz")


def test_never():
    got = 1 + 1
    assert got == 2
