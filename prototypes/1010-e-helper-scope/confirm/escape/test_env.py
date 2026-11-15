"""Escape under module scope: two shared helpers of real suites, each imported and defined here.
The first in the shape of requests' test_respect_proxy_env_on_send_self_prepared_request,
the second of loguru's parse."""

import contextlib
import os

import env_helpers
import markup_helpers


def proxy_of_env():
    return os.environ.get("http_proxy")


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


# the helper imported from the tests tree, as requests does
def test_proxy_env_imported_helper():
    with env_helpers.override_environ(http_proxy="http://bad.invalid:3128"):
        proxy = proxy_of_env()
    assert proxy == "http://bad.invalid:3128"


# the same helper, defined in this module
def test_proxy_env_module_helper():
    with override_environ(http_proxy="http://bad.invalid:3128"):
        proxy = proxy_of_env()
    assert proxy == "http://bad.invalid:3128"


def parse(text, *, strip=False):
    tokens = text.replace("<red>", "\x1b[31m").replace("</red>", "\x1b[0m")
    if strip:
        return tokens.replace("\x1b[31m", "").replace("\x1b[0m", "")
    return tokens


# a plain function helper imported from the tests tree,
# as loguru's tests import `parse` from conftest
def test_markup_imported_helper():
    colored = markup_helpers.parse("<red>x</red>")
    assert colored == "\x1b[31mx\x1b[0m"


# the same plain function, defined in this module
def test_markup_module_helper():
    colored = parse("<red>x</red>")
    assert colored == "\x1b[31mx\x1b[0m"
