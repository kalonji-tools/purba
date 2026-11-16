"""loguru's tests/test_add_option_format.py (MIT, Delgan/loguru@28349be; see LICENSE.loguru), ported
to the form. The two conftest fixtures it uses become `reset_logger` (run by `@before`) and the
`Writer` fixture."""

from __future__ import annotations

from collections.abc import Callable
from typing import TYPE_CHECKING, Any, cast

import loguru
import loguru._logger
from loguru import logger

from cases import before, case, raises, use

if TYPE_CHECKING:
    from loguru import Record


# ── what conftest.py gave ────────────────────────


def reset_logger() -> None:
    """conftest's autouse `reset_logger`: a fresh logger with no handler. The case's process ends
    after the case, so the reset after it is not needed."""
    loguru.logger.remove()
    # The stub types `logger` as a `Logger` with no constructor; the reset calls the real one.
    cast(Any, loguru.logger).__init__(
        loguru._logger.Core(), None, 0, False, False, False, False, True, [], {}
    )
    loguru._logger.context.set({})


class Writer:
    """conftest's `writer`: a sink that keeps each message it gets."""

    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: str) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)


# ── the tests ────────────────────────────────────


@before(reset_logger)
def test_format(
    message: str,
    format: str | Callable[[Record], str],
    expected: str,
    writer: Writer = use(Writer),
) -> None:
    logger.add(writer, format=format)
    logger.debug(message)
    assert writer.read() == expected


format_a = case(test_format, message="a", format="Message: {message}", expected="Message: a\n")
format_b = case(test_format, message="b", format="Nope", expected="Nope\n")
format_c = case(
    test_format, message="c", format="{level} {message} {level}", expected="DEBUG c DEBUG\n"
)
format_d = case(
    test_format,
    message="d",
    format="{message} {level} {level.no} {level.name}",
    expected="d DEBUG 10 DEBUG\n",
)
format_e = case(test_format, message="e", format=lambda _: "{message}", expected="e")
format_f = case(
    test_format,
    message="f",
    format=lambda r: "{message} " + r["level"].name,
    expected="f DEBUG",
)


@before(reset_logger)
def test_progressive_format(writer: Writer = use(Writer)) -> None:
    def formatter(record: Record) -> str:
        fmt = "[{level.name}] {message}"
        if "noend" not in record["extra"]:
            fmt += "\n"
        return fmt

    logger.add(writer, format=formatter)
    logger.bind(noend=True).debug("Start: ")
    for _ in range(5):
        logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug("\n")
    logger.debug("End")
    assert writer.read() == ("[DEBUG] Start: .....\n" "[DEBUG] End\n")


@before(reset_logger)
def test_function_format_without_exception(writer: Writer = use(Writer)) -> None:
    logger.add(writer, format=lambda _: "{message}\n")
    try:
        _ = 1 / 0
    except ZeroDivisionError:
        logger.exception("Error!")
    assert writer.read() == "Error!\n"


@before(reset_logger)
def test_function_format_with_exception(writer: Writer = use(Writer)) -> None:
    logger.add(writer, format=lambda _: "{message}\n{exception}")
    try:
        _ = 1 / 0
    except ZeroDivisionError:
        logger.exception("Error!")
    lines = writer.read().splitlines()
    assert lines[0] == "Error!"
    assert lines[-1] == "ZeroDivisionError: division by zero"


@before(reset_logger)
def test_invalid_format(format: object, writer: Writer = use(Writer)) -> None:
    with raises(TypeError, match=r"^Invalid format, it should be a string or a function, not: "):
        # `format` is the wrong type on purpose; the stub refuses it, the code under test raises.
        logger.add(writer, format=cast(Any, format))


invalid_format_int = case(test_invalid_format, format=-1)
invalid_format_float = case(test_invalid_format, format=3.4)
invalid_format_object = case(test_invalid_format, format=object())


@before(reset_logger)
def test_invalid_markups(format: str, writer: Writer = use(Writer)) -> None:
    with raises(ValueError, match=r"^Invalid format, color markups could not be parsed correctly$"):
        logger.add(writer, format=format)


invalid_markups_open = case(test_invalid_markups, format="<red>")
invalid_markups_close = case(test_invalid_markups, format="</red>")
invalid_markups_crossed = case(test_invalid_markups, format="</level><level>")
invalid_markups_bare_close = case(test_invalid_markups, format="</>")
invalid_markups_unknown = case(test_invalid_markups, format="<foobar>")


@before(reset_logger)
def test_markup_in_field(colorize: bool, writer: Writer = use(Writer)) -> None:
    class F:
        def __format__(self, spec: str) -> str:
            return spec

    logger.add(writer, format="{extra[f]:</>} {extra[f]: <blue> } {message}", colorize=colorize)
    logger.bind(f=F()).info("Test")

    assert writer.read() == "</>  <blue>  Test\n"


markup_in_field_colorized = case(test_markup_in_field, colorize=True)
markup_in_field_plain = case(test_markup_in_field, colorize=False)


@before(reset_logger)
def test_invalid_format_builtin(writer: Writer = use(Writer)) -> None:
    with raises(ValueError, match=r".* most likely a mistake"):
        logger.add(writer, format=format)
