import sys
import pytest


def test_body_if_skip_taken():
    if sys.platform == "linux":
        pytest.skip("not on linux")
    total = 1 + 1
    assert total == 2
