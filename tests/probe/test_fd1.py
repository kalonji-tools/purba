import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fd1  # noqa: E402


def test_where():
    fd1.where("inside a test")
    assert 1 == 1
