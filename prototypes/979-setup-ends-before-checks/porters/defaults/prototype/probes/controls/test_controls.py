# Negative controls for the test_defaults port: each case here must FAIL or be REFUSED.
# Adapted from loguru's tests/test_defaults.py (MIT, Delgan/loguru@28349be); see LICENSE.loguru.
from pytest import MonkeyPatch

from cases import case, raises, use
from loguru._defaults import env


def test_string_mutant(value: str, monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "VALID_STRING"
        context.setenv(key, value + "x")
        assert env(key, str) == value


string_mutant = case(test_string_mutant, value="test")


def test_check_after_undo(value: str, monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "VALID_STRING"
        context.setenv(key, value)
    assert env(key, str) == value


check_after_undo = case(test_check_after_undo, value="test")


def test_int_type_strict(monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "VALID_BOOL_POS"
        context.setenv(key, "1")
        assert env(key, bool) == 1


def test_raises_wrong_match(monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "INVALID_INT"
        context.setenv(key, "a")
        with raises(ValueError, match=r"^Invalid environment variable 'INVALID_INT' \(expected an integer\)$"):
            env(key, int)


def test_raises_no_raise(monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "INVALID_INT"
        context.setenv(key, "42")
        with raises(ValueError):
            env(key, int)


def test_wrong_input_type(value: str, monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "VALID_STRING"
        context.setenv(key, value)
        assert env(key, str) == value


wrong_input_type = case(test_wrong_input_type, value=1)  # type: ignore[arg-type]


def test_checks_nothing(monkeypatch: MonkeyPatch = use(MonkeyPatch)):
    with monkeypatch.context() as context:
        key = "INVALID_INT"
        context.setenv(key, "a")
        env(key, str)
