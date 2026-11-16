import pytest

import shop


def test_negative():
    with pytest.raises(ValueError, match="negative"):
        shop.price(-1)
