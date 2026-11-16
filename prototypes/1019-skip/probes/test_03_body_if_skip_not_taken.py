import sys
import pytest


def test_body_if_skip_not_taken():
    if sys.platform == "win32":
        pytest.skip("posix only")
    total = 1 + 1
    assert total == 2
