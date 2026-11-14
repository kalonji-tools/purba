"""loguru's tests/test_add_option_format.py, ported to oxitest.

Every test runs inside the autouse ``reset_logger`` fixture from
``__fixtures__.py``, so each one starts from a logger with no handlers.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

import oxitest as oxi
from loguru import logger

if TYPE_CHECKING:
    from loguru import FormatFunction, Record
else:
    # loguru declares FormatFunction only in its .pyi stubs, but oxitest calls
    # get_type_hints() on every parametrize case class at collection, so the
    # name must also exist at runtime. Type checkers still see loguru's alias.
    from collections.abc import Callable

    FormatFunction = Callable[..., str]


class Writer:
    """A sink that keeps every message it is sent (loguru's conftest ``writer``)."""

    def __init__(self) -> None:
        self.written: list[str] = []

    def __call__(self, message: str) -> None:
        self.written.append(message)

    def read(self) -> str:
        return "".join(self.written)


# --- test_format -----------------------------------------------------------


@dataclass(frozen=True)
class FormatCase:
    message: str
    fmt: str | FormatFunction
    expected: str


@oxi.parametrize(
    message_field=FormatCase(message="a", fmt="Message: {message}", expected="Message: a\n"),
    no_fields=FormatCase(message="b", fmt="Nope", expected="Nope\n"),
    level_twice=FormatCase(
        message="c", fmt="{level} {message} {level}", expected="DEBUG c DEBUG\n"
    ),
    level_attributes=FormatCase(
        message="d",
        fmt="{message} {level} {level.no} {level.name}",
        expected="d DEBUG 10 DEBUG\n",
    ),
    # A format *function* gets no implicit trailing newline.
    function_constant=FormatCase(message="e", fmt=lambda _: "{message}", expected="e"),
    function_reads_record=FormatCase(
        message="f", fmt=lambda r: "{message} " + r["level"].name, expected="f DEBUG"
    ),
)
def test_format(message: str, fmt: str | FormatFunction, expected: str) -> None:
    writer = Writer()
    logger.add(writer, format=fmt)
    logger.debug(message)
    assert writer.read() == expected, f"format {fmt!r} should render {message!r} as {expected!r}"


# --- format functions -------------------------------------------------------


def test_progressive_format() -> None:
    def formatter(record: Record) -> str:
        fmt = "[{level.name}] {message}"
        if "noend" not in record["extra"]:
            fmt += "\n"
        return fmt

    writer = Writer()
    logger.add(writer, format=formatter)
    logger.bind(noend=True).debug("Start: ")
    for _ in range(5):
        logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug("\n")
    logger.debug("End")
    assert writer.read() == "[DEBUG] Start: .....\n[DEBUG] End\n", (
        "the formatter decides per record whether a newline ends it, and raw messages bypass it"
    )


def test_function_format_without_exception() -> None:
    writer = Writer()
    logger.add(writer, format=lambda _: "{message}\n")
    try:
        _ = 1 / 0
    except ZeroDivisionError:
        logger.exception("Error!")
    assert writer.read() == "Error!\n", (
        "a format function that omits {exception} must not get the traceback appended"
    )


def test_function_format_with_exception() -> None:
    writer = Writer()
    logger.add(writer, format=lambda _: "{message}\n{exception}")
    try:
        _ = 1 / 0
    except ZeroDivisionError:
        logger.exception("Error!")
    lines = writer.read().splitlines()
    assert lines[0] == "Error!", "the message comes first"
    assert lines[-1] == "ZeroDivisionError: division by zero", (
        "{exception} renders the traceback, ending with the exception line"
    )


# --- rejected formats -------------------------------------------------------


@dataclass(frozen=True)
class InvalidFormatCase:
    fmt: object


@oxi.parametrize(
    negative_int=InvalidFormatCase(fmt=-1),
    float=InvalidFormatCase(fmt=3.4),
    plain_object=InvalidFormatCase(fmt=object()),
)
def test_invalid_format(fmt: object) -> None:
    with oxi.raises(TypeError):
        # Deliberately ill-typed: the test proves loguru rejects it at runtime.
        logger.add(Writer(), format=fmt)  # pyright: ignore[reportArgumentType, reportCallIssue]  # ty: ignore[no-matching-overload]


@dataclass(frozen=True)
class MarkupCase:
    fmt: str


@oxi.parametrize(
    unclosed_tag=MarkupCase(fmt="<red>"),
    close_without_open=MarkupCase(fmt="</red>"),
    close_before_open=MarkupCase(fmt="</level><level>"),
    bare_close=MarkupCase(fmt="</>"),
    unknown_tag=MarkupCase(fmt="<foobar>"),
)
def test_invalid_markups(fmt: str) -> None:
    with oxi.raises(
        ValueError, match=r"^Invalid format, color markups could not be parsed correctly$"
    ):
        logger.add(Writer(), format=fmt)


@dataclass(frozen=True)
class ColorizeCase:
    colorize: bool


@oxi.parametrize(
    colorized=ColorizeCase(colorize=True),
    plain=ColorizeCase(colorize=False),
)
def test_markup_in_field(colorize: bool) -> None:
    class F:
        def __format__(self, spec: str) -> str:
            return spec

    writer = Writer()
    logger.add(writer, format="{extra[f]:</>} {extra[f]: <blue> } {message}", colorize=colorize)
    logger.bind(f=F()).info("Test")

    assert writer.read() == "</>  <blue>  Test\n", (
        "markup inside a field's format spec is the field's output, never parsed as color"
    )


def test_invalid_format_builtin() -> None:
    # `format` here is the *builtin* function: it type-checks as a format
    # function, so loguru has to catch the mistake at runtime.
    with oxi.raises(ValueError, match=r".* most likely a mistake"):
        logger.add(Writer(), format=format)
