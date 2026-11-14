# Adapted from loguru's tests/test_defaults.py (MIT, Delgan/loguru@28349be); see LICENSE.loguru.
from pytest import MonkeyPatch

from cases import before_each, case, raises, use
from loguru._defaults import env
from loguru_conftest import reset_logger, reset_multiprocessing_start_method

before_each(reset_logger)
before_each(reset_multiprocessing_start_method)


def test_string(value: str, monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "VALID_STRING"
        context.setenv(key, value)
        assert env(key, str) == value


string_test = case(test_string, value="test")
string_empty = case(test_string, value="")


def test_bool_positive(value: str, monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "VALID_BOOL_POS"
        context.setenv(key, value)
        assert env(key, bool) is True


bool_positive_y = case(test_bool_positive, value="y")
bool_positive_1 = case(test_bool_positive, value="1")
bool_positive_TRUE = case(test_bool_positive, value="TRUE")


def test_bool_negative(value: str, monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "VALID_BOOL_NEG"
        context.setenv(key, value)
        assert env(key, bool) is False


bool_negative_NO = case(test_bool_negative, value="NO")
bool_negative_0 = case(test_bool_negative, value="0")
bool_negative_false = case(test_bool_negative, value="false")


def test_int(monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "VALID_INT"
        context.setenv(key, "42")
        assert env(key, int) == 42


def test_invalid_int(value: str, monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "INVALID_INT"
        context.setenv(key, value)
        with raises(
            ValueError,
            match=r"^Invalid environment variable 'INVALID_INT' \(expected an integer\): '[^']*'$",
        ):
            env(key, int)


invalid_int_empty = case(test_invalid_int, value="")
invalid_int_a = case(test_invalid_int, value="a")


def test_invalid_bool(value: str, monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "INVALID_BOOL"
        context.setenv(key, value)
        with raises(
            ValueError,
            match=r"^Invalid environment variable 'INVALID_BOOL' \(expected a boolean\): '[^']*'$",
        ):
            env(key, bool)


invalid_bool_empty = case(test_invalid_bool, value="")
invalid_bool_a = case(test_invalid_bool, value="a")


def test_invalid_type(monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "INVALID_TYPE"
        context.setenv(key, "42.0")
        with raises(ValueError, match=r"^The requested type '[^']+' is not supported"):
            env(key, float)
