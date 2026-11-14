"""loguru's tests/test_add_option_format.py (MIT, Delgan/loguru@28349be), ported to this form."""

from __future__ import annotations

from typing import TYPE_CHECKING

from experiment import experiment, handling, use
from loguru_lab import Writer, fresh_logger

if TYPE_CHECKING:
    from loguru import FormatFunction, Logger, Record


# test_format: one message, logged through a format that is a string or a function.
@experiment
def logged(message: str, format: str | FormatFunction, writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> str:
    logger.add(writer, format=format)
    logger.debug(message)
    return writer.read()


message_field   = logged(message="a", format="Message: {message}").expect(product="Message: a\n")
no_field        = logged(message="b", format="Nope").expect(product="Nope\n")
level_twice     = logged(message="c", format="{level} {message} {level}").expect(product="DEBUG c DEBUG\n")
level_parts     = logged(message="d", format="{message} {level} {level.no} {level.name}").expect(product="d DEBUG 10 DEBUG\n")
function_format = logged(message="e", format=lambda _: "{message}").expect(product="e")
function_record = logged(message="f", format=lambda r: "{message} " + r["level"].name).expect(product="f DEBUG")


# test_progressive_format
def formatter(record: Record) -> str:
    fmt = "[{level.name}] {message}"
    if "noend" not in record["extra"]:
        fmt += "\n"
    return fmt


@experiment
def logged_progressively(writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> str:
    logger.add(writer, format=formatter)
    logger.bind(noend=True).debug("Start: ")
    logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug(".")
    logger.opt(raw=True).debug("\n")
    logger.debug("End")
    return writer.read()


progressive = logged_progressively().expect(product="[DEBUG] Start: .....\n" "[DEBUG] End\n")


# test_function_format_without_exception, test_function_format_with_exception
@experiment
def logged_in_except(format: FormatFunction, writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> str:
    logger.add(writer, format=format)
    handling(ZeroDivisionError("division by zero"), logger.exception, "Error!")
    return writer.read()


@logged_in_except.assay
def first_line(product: str) -> str:
    return product.splitlines()[0]


@logged_in_except.assay
def last_line(product: str) -> str:
    return product.splitlines()[-1]


without_exception = logged_in_except(format=lambda _: "{message}\n").expect(product="Error!\n")
with_exception    = logged_in_except(format=lambda _: "{message}\n{exception}").expect(first_line="Error!", last_line="ZeroDivisionError: division by zero")


# test_invalid_format, test_invalid_markups, test_invalid_format_builtin
@experiment
def added(format: str | FormatFunction, writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> int:
    return logger.add(writer, format=format)


UNPARSED = r"^Invalid format, color markups could not be parsed correctly$"

int_format     = added(format=-1).raises(TypeError)  # type: ignore
float_format   = added(format=3.4).raises(TypeError)  # type: ignore
object_format  = added(format=object()).raises(TypeError)  # type: ignore
open_tag       = added(format="<red>").raises(ValueError, match=UNPARSED)
close_tag      = added(format="</red>").raises(ValueError, match=UNPARSED)
crossed_tags   = added(format="</level><level>").raises(ValueError, match=UNPARSED)
bare_close     = added(format="</>").raises(ValueError, match=UNPARSED)
unknown_tag    = added(format="<foobar>").raises(ValueError, match=UNPARSED)
builtin_format = added(format=format).raises(ValueError, match=r".* most likely a mistake")


# test_markup_in_field
class F:
    def __format__(self, spec: str) -> str:
        return spec


@experiment
def logged_with_field(colorize: bool, writer: Writer = use(Writer), logger: Logger = use(fresh_logger)) -> str:
    logger.add(writer, format="{extra[f]:</>} {extra[f]: <blue> } {message}", colorize=colorize)
    logger.bind(f=F()).info("Test")
    return writer.read()


markup_in_field_colorized = logged_with_field(colorize=True).expect(product="</>  <blue>  Test\n")
markup_in_field_plain     = logged_with_field(colorize=False).expect(product="</>  <blue>  Test\n")
