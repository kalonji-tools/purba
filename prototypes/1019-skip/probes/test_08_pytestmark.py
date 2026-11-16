import pytest

pytestmark = pytest.mark.skip(reason="whole module off")


def test_under_pytestmark():
    total = 1 + 1
    assert total == 2
