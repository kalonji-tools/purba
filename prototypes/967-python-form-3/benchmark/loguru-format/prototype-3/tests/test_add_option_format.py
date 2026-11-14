"""loguru's tests/test_add_option_format.py (MIT, Delgan/loguru@28349be), ported to this form."""

from __future__ import annotations

from collections.abc import Callable
from typing import TYPE_CHECKING

from cases import case, raises, use
from loguru_lab import Writer, fresh_logger, progressive_formatter

if TYPE_CHECKING:
    from loguru import Logger, Record


def test_format(
    message: str,
    format: str | Callable[[Record], str],
    expected: str,
    writer: Writer = use(Writer),
    logger: Logger = use(fresh_logger),
) -> None:
    logger.add(writer, format=format)
    logger.debug(message)
    assert writer.read() == expected


message_field     = case(test_format, message="a", format="Message: {message}", expected="Message: a\n")
no_field          = case(test_format, message="b", format="Nope", expected="Nope\n")
level_twice       = case(test_format, message="c", format="{level} {message} {level}", expected="DEBUG c DEBUG\n")
level_attributes  = case(test_format, message="d", format="{message} {level} {level.no} {level.name}", expected="d DEBUG 10 DEBUG\n")
function_format   = case(test_format, message="e", format=lambda _: "{message}", expected="e")
function_of_level = case(test_format, message="f", format=lambda r: "{message} " + r["level"].name, expected="f DEBUG")


def test_progressive_format(writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> None:
    logger.add(writer, format=progressive_formatter)
    logger.bind(noend=True).debug("Start: ")
    logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug("\n")
    logger.debug("End")
    assert writer.read() == ("[DEBUG] Start: .....\n" "[DEBUG] End\n")


def test_function_format_without_exception(writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> None:
    logger.add(writer, format=lambda _: "{message}\n")
    try:
        _ = 1 / 0
    except ZeroDivisionError:
        logger.exception("Error!")
    assert writer.read() == "Error!\n"


def test_function_format_with_exception(writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> None:
    logger.add(writer, format=lambda _: "{message}\n{exception}")
    try:
        _ = 1 / 0
    except ZeroDivisionError:
        logger.exception("Error!")
    lines = writer.read().splitlines()
    assert lines[0] == "Error!"
    assert lines[-1] == "ZeroDivisionError: division by zero"


def test_invalid_format(format: object, writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> None:
    with raises(TypeError):
        logger.add(writer, format=format)  # pyright: ignore[reportArgumentType, reportCallIssue]  # ty: ignore[no-matching-overload]


negative_int = case(test_invalid_format, format=-1)
a_float      = case(test_invalid_format, format=3.4)
an_object    = case(test_invalid_format, format=object())


def test_invalid_markups(format: str, writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> None:
    with raises(ValueError, match=r"^Invalid format, color markups could not be parsed correctly$"):
        logger.add(writer, format=format)


open_red        = case(test_invalid_markups, format="<red>")
close_red       = case(test_invalid_markups, format="</red>")
close_then_open = case(test_invalid_markups, format="</level><level>")
bare_close      = case(test_invalid_markups, format="</>")
unknown_tag     = case(test_invalid_markups, format="<foobar>")


def test_markup_in_field(colorize: bool, writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> None:
    class F:
        def __format__(self, spec: str) -> str:
            return spec

    logger.add(writer, format="{extra[f]:</>} {extra[f]: <blue> } {message}", colorize=colorize)
    logger.bind(f=F()).info("Test")
    assert writer.read() == "</>  <blue>  Test\n"


colorized   = case(test_markup_in_field, colorize=True)
uncolorized = case(test_markup_in_field, colorize=False)


def test_invalid_format_builtin(writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> None:
    with raises(ValueError, match=r".* most likely a mistake"):
        logger.add(writer, format=format)
