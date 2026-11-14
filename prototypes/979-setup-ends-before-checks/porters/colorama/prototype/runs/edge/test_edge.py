import os
from unittest.mock import MagicMock

from cases import case


def test_untyped(patched, expected):
    seen = MagicMock()
    seen(patched)
    assert seen.called is expected


misspelled = case(test_untyped, pached="x", expected=True)
missing = case(test_untyped, expected=True)


def test_os_name_left_patched():
    os.name = "nt"
    flag = MagicMock()
    flag()
    assert flag.called is True
    assert os.name == "nt"
