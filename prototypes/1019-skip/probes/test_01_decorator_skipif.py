import sys
import pytest


@pytest.mark.skipif(sys.platform == "win32", reason="posix only")
def test_decorator_skipif():
    total = 1 + 1
    assert total == 2
