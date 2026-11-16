import pytest


def test_importorskip():
    mod = pytest.importorskip("no_such_module_xyz")
    assert mod.x == 1
