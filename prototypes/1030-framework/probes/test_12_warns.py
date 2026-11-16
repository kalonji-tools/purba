import warnings

import pytest


def test_warns():
    with pytest.warns(UserWarning):
        warnings.warn("old", UserWarning)
