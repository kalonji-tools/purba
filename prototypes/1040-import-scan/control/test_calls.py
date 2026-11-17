import importlib
from importlib.util import find_spec
from shop import files
def test_a():
    importlib.import_module("pytest")
def test_b():
    files("pytest")
    find_spec("pytest")
    find_spec("pytest.mark")
def test_c():
    assert 1 == 1
