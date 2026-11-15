"""The `format` option of `logger.add`: a format string, a format function, and the formats that
`add` refuses. Ported from loguru's tests/test_add_option_format.py (MIT, see LICENSE.loguru)."""

from __future__ import annotations

from typing import TYPE_CHECKING

from cases import before, case, handling, raises, use
from loguru import logger
from loguru._logger import Core, context

if TYPE_CHECKING:
    from loguru import FormatFunction, Record


def reset_logger() -> None:
    """Start each case from a logger with no sink, as loguru's conftest.py does."""
    logger.remove()
    logger.__init__(Core(), None, 0, False, False, False, False, True, [], {})  # type: ignore
    context.set({})


class Writer:
    """A sink that keeps each message that the logger writes to it."""

    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: str) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)


@before(reset_logger)
def test_format(
    message: str, format: str | FormatFunction, expected: str, writer: Writer = use(Writer)
) -> None:
    """A format string, or a format function, shapes the message that the sink gets."""
    logger.add(writer, format=format)
    logger.debug(message)
    assert writer.read() == expected


message_field = case(test_format, message="a", format="Message: {message}", expected="Message: a\n")
no_field = case(test_format, message="b", format="Nope", expected="Nope\n")
level_twice = case(
    test_format, message="c", format="{level} {message} {level}", expected="DEBUG c DEBUG\n"
)
level_attributes = case(
    test_format,
    message="d",
    format="{message} {level} {level.no} {level.name}",
    expected="d DEBUG 10 DEBUG\n",
)
function_without_newline = case(test_format, message="e", format=lambda _: "{message}", expected="e")
function_reads_the_record = case(
    test_format, message="f", format=lambda r: "{message} " + r["level"].name, expected="f DEBUG"
)


@before(reset_logger)
def test_progressive_format(writer: Writer = use(Writer)) -> None:
    """A format function that leaves out the newline lets raw messages continue the same line."""

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
    """A format function with no `{exception}` field writes no traceback."""
    logger.add(writer, format=lambda _: "{message}\n")
    with handling(ZeroDivisionError, logger.exception, "Error!"):
        _ = 1 / 0
    assert writer.read() == "Error!\n"


@before(reset_logger)
def test_function_format_with_exception(writer: Writer = use(Writer)) -> None:
    """A format function with an `{exception}` field writes the traceback after the message."""
    logger.add(writer, format=lambda _: "{message}\n{exception}")
    with handling(ZeroDivisionError, logger.exception, "Error!"):
        _ = 1 / 0
    lines = writer.read().splitlines()
    assert lines[0] == "Error!"
    assert lines[-1] == "ZeroDivisionError: division by zero"


@before(reset_logger)
def test_invalid_format(format: object, writer: Writer = use(Writer)) -> None:
    """A format that is neither a string nor a function is refused."""
    with raises(TypeError):
        logger.add(writer, format=format)  # type: ignore


format_int = case(test_invalid_format, format=-1)
format_float = case(test_invalid_format, format=3.4)
format_object = case(test_invalid_format, format=object())


@before(reset_logger)
def test_invalid_markups(format: str, writer: Writer = use(Writer)) -> None:
    """A color markup that does not open and close in pairs is refused."""
    with raises(ValueError, match=r"^Invalid format, color markups could not be parsed correctly$"):
        logger.add(writer, format=format)


open_tag_alone = case(test_invalid_markups, format="<red>")
close_tag_alone = case(test_invalid_markups, format="</red>")
close_before_open = case(test_invalid_markups, format="</level><level>")
bare_close_tag_alone = case(test_invalid_markups, format="</>")
unknown_tag = case(test_invalid_markups, format="<foobar>")


@before(reset_logger)
def test_markup_in_field(colorize: bool, writer: Writer = use(Writer)) -> None:
    """A markup that a field's format spec gives is text, not a color, with or without colors."""

    class F:
        def __format__(self, spec: str) -> str:
            return spec

    logger.add(writer, format="{extra[f]:</>} {extra[f]: <blue> } {message}", colorize=colorize)
    logger.bind(f=F()).info("Test")
    assert writer.read() == "</>  <blue>  Test\n"


colorized = case(test_markup_in_field, colorize=True)
not_colorized = case(test_markup_in_field, colorize=False)


@before(reset_logger)
def test_invalid_format_builtin(writer: Writer = use(Writer)) -> None:
    """The builtin `format` function, passed by mistake, is refused."""
    with raises(ValueError, match=r".* most likely a mistake"):
        logger.add(writer, format=format)
