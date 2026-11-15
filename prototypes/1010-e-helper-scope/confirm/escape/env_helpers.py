"""Mirrors the shape of requests' tests/utils.py `override_environ`:
a shared helper of the tests tree."""

import contextlib
import os


@contextlib.contextmanager
def override_environ(**kwargs):
    saved = dict(os.environ)
    for key, value in kwargs.items():
        if value is None:
            del os.environ[key]
        else:
            os.environ[key] = value
    try:
        yield
    finally:
        os.environ.clear()
        os.environ.update(saved)
