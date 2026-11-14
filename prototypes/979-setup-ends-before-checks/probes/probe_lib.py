"""Stands for a library the test calls: it mutes warnings around the code it runs, inside one call."""

import warnings
from collections.abc import Callable


def quietly(fn: Callable[[], int]) -> int:
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return fn()
