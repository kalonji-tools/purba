"""The `format` option of `logger.add`, ported from loguru's tests/test_add_option_format.py
(MIT, Delgan/loguru@28349be)."""

from __future__ import annotations

from typing import TYPE_CHECKING

from cases import case, handling, raises, use
from loguru import logger

if TYPE_CHECKING:
    from loguru import FormatFunction, Message, Record


def reset_logger() -> None:
    """A fixture: the logger with no handler, not even loguru's default one on stderr."""
    logger.remove()


class Writer:
    """A stand-in sink: it keeps each message that loguru gives it."""

    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: Message) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)


def test_format(
    message: str,
    format: str | FormatFunction,
    expected: str,
    writer: Writer = use(Writer),
    _reset: None = use(reset_logger),
) -> None:
    logger.add(writer, format=format)
    logger.debug(message)
    assert writer.read() == expected


message_field = case(
    test_format, message="a", format="Message: {message}", expected="Message: a\n"
)
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
function_format = case(test_format, message="e", format=lambda _: "{message}", expected="e")
function_reads_record = case(
    test_format,
    message="f",
    format=lambda r: "{message} " + r["level"].name,
    expected="f DEBUG",
)


def test_progressive_format(
    writer: Writer = use(Writer), _reset: None = use(reset_logger)
) -> None:
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


def test_function_format_without_exception(
    writer: Writer = use(Writer), _reset: None = use(reset_logger)
) -> None:
    logger.add(writer, format=lambda _: "{message}\n")
    with handling(ZeroDivisionError, logger.exception, "Error!"):
        _ = 1 / 0
    assert writer.read() == "Error!\n"


def test_function_format_with_exception(
    writer: Writer = use(Writer), _reset: None = use(reset_logger)
) -> None:
    logger.add(writer, format=lambda _: "{message}\n{exception}")
    with handling(ZeroDivisionError, logger.exception, "Error!"):
        _ = 1 / 0
    lines = writer.read().splitlines()
    assert lines[0] == "Error!"
    assert lines[-1] == "ZeroDivisionError: division by zero"


def test_invalid_format(
    format: object, writer: Writer = use(Writer), _reset: None = use(reset_logger)
) -> None:
    with raises(TypeError):
        logger.add(writer, format=format)  # type: ignore  # a wrong type, on purpose


negative_int = case(test_invalid_format, format=-1)
float_number = case(test_invalid_format, format=3.4)
bare_object = case(test_invalid_format, format=object())


def test_invalid_markups(
    format: str, writer: Writer = use(Writer), _reset: None = use(reset_logger)
) -> None:
    with raises(ValueError, match=r"^Invalid format, color markups could not be parsed correctly$"):
        logger.add(writer, format=format)


unclosed_tag = case(test_invalid_markups, format="<red>")
unopened_tag = case(test_invalid_markups, format="</red>")
closed_before_opened = case(test_invalid_markups, format="</level><level>")
bare_closing_tag = case(test_invalid_markups, format="</>")
unknown_tag = case(test_invalid_markups, format="<foobar>")


def test_markup_in_field(
    colorize: bool, writer: Writer = use(Writer), _reset: None = use(reset_logger)
) -> None:
    class F:
        def __format__(self, spec: str) -> str:
            return spec

    logger.add(writer, format="{extra[f]:</>} {extra[f]: <blue> } {message}", colorize=colorize)
    logger.bind(f=F()).info("Test")
    assert writer.read() == "</>  <blue>  Test\n"


colorized = case(test_markup_in_field, colorize=True)
not_colorized = case(test_markup_in_field, colorize=False)


def test_invalid_format_builtin(
    writer: Writer = use(Writer), _reset: None = use(reset_logger)
) -> None:
    with raises(ValueError, match=r".* most likely a mistake"):
        logger.add(writer, format=format)
