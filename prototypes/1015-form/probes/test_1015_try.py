"""#1015, change 2: `try` with one statement and one `except SomeError:` replaces `handling`. It
counts as one path. When the statement does not raise SomeError, the case fails; any other `try`
keeps the path refusal, which now names the allowed shape."""

from __future__ import annotations

import json
import sys
import traceback

from undertest import parse


def test_try_of_the_form() -> None:
    """PASS: the statement raises, and the `except` body runs while the exception is live."""
    seen: list[str] = []
    try:
        parse("x")
    except ValueError:
        seen.append(traceback.format_exc().splitlines()[-1])
    assert seen == ["ValueError: invalid literal for int() with base 10: 'x'"]


def test_try_with_a_dotted_name_and_as() -> None:
    """PASS: a dotted name, and `as`, are allowed."""
    seen: list[str] = []
    try:
        json.loads("{")
    except json.JSONDecodeError as error:
        seen.append(error.msg)
        seen.append(type(sys.exc_info()[1]).__name__)
    assert seen == ["Expecting property name enclosed in double quotes", "JSONDecodeError"]


def test_try_that_does_not_raise() -> None:
    """FAIL: the statement returns, so the exception never came. Says "the `try` at line 39
    expects ValueError, and the statement did not raise it" """
    seen: list[str] = []
    try:
        parse("1")
    except ValueError:
        seen.append("handled")
    assert seen == ["handled"]


def test_try_that_raises_another_type() -> None:
    """FAIL: a raise of another type ends the act. Says "the act raised TypeError" """
    seen: list[str] = []
    try:
        parse(None)  # ty: ignore[invalid-argument-type]
    except ValueError:
        seen.append("handled")
    assert seen == ["handled"]


def test_try_of_two_statements() -> None:
    """REFUSED: the second statement never runs, and the `except` goes one way; the refusal names
    the allowed shape. Says "a `try` of the form holds one statement and one `except SomeError:`"
    """
    seen: list[str] = []
    try:
        parse("x")
        seen.append("parsed")
    except ValueError:
        seen.append("handled")
    assert seen == ["handled"]


def test_try_with_finally() -> None:
    """REFUSED: a `finally` is another shape, so its `except` must go both ways. Says "the
    `except` at test_1015_try.py:75 went one way" """
    seen: list[str] = []
    try:
        parse("x")
    except ValueError:
        seen.append("handled")
    finally:
        seen.append("done")
    assert seen == ["handled", "done"]


def test_bare_except() -> None:
    """REFUSED: a bare `except` is another shape; the statement that raises leaves a line that
    never ran. Says "never ran" """
    seen: list[str] = []
    try:
        seen.append(str(parse("1")))
    except:  # noqa: E722
        seen.append("handled")
    assert seen == ["1"]


def test_assert_in_the_except() -> None:
    """REFUSED: an assert in the `except` body runs in the act. Says "an assert ran in the act" """
    seen: list[str] = []
    try:
        parse("x")
    except ValueError:
        assert sys.exc_info()[0] is ValueError
        seen.append("handled")
    assert seen == ["handled"]
