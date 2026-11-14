"""ex_loguru_format: loguru's tests/test_add_option_format.py (MIT, Delgan/loguru@28349be), ported to record_strict.

Each loguru call is one step: `add`, `bind`, `opt`, then `log`. A later step takes the logger of the step before.
The writer comes from `use(sink)`, one per process; benchmark/loguru-format/prototype-1/NOTES.md says why a chained writer does not work.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Self

from form_record_strict import Experiment, TypedExperiment, use
from loguru_lab import Writer, division_by_zero, fresh_logger, sink

if TYPE_CHECKING:
    from loguru import FormatFunction, Logger, Record


class Added(Experiment[int]):  # small: the only assay is the built-in `product`, the handler id
    format: str | FormatFunction
    colorize: bool | None = None
    writer: Writer = use(sink)
    logger: Logger = use(fresh_logger)

    def act(self) -> int:
        return self.logger.add(self.writer, format=self.format, colorize=self.colorize)


class Bound(Experiment["Logger"]):  # small: one case uses it
    logger: Logger
    extra: dict[str, object]
    writer: Writer = use(sink)

    def act(self) -> Logger:
        return self.logger.bind(**self.extra)

    def written(self) -> str:
        return self.writer.read()


class Opted(Experiment["Logger"]):  # small: one chain uses it
    logger: Logger
    raw: bool
    writer: Writer = use(sink)

    def act(self) -> Logger:
        return self.logger.opt(raw=self.raw)

    def written(self) -> str:
        return self.writer.read()


class WithError(Experiment["Logger"]):  # small: two cases use it
    logger: Logger
    exception: ZeroDivisionError = use(division_by_zero)
    writer: Writer = use(sink)

    def act(self) -> Logger:
        return self.logger.opt(exception=self.exception)

    def written(self) -> str:
        return self.writer.read()


class Logged(TypedExperiment[None]):  # typed: 17 cases read `written`, so the editor sees a missing or mistyped value
    logger: Logger
    message: str
    level: str = "DEBUG"
    extra: dict[str, object] = {}
    writer: Writer = use(sink)

    def act(self) -> None:
        return self.logger.log(self.level, self.message, **self.extra)

    def written(self) -> str:
        return self.writer.read()

    def expect(self, *, written: str) -> Self:
        return self


class LoggedLines(Experiment[None]):  # small: one case; the traceback holds a path, so it reads the first and last line
    logger: Logger
    message: str
    level: str = "DEBUG"
    writer: Writer = use(sink)

    def act(self) -> None:
        return self.logger.log(self.level, self.message)

    def first_line(self) -> str:
        return next(iter(self.writer.read().splitlines()))

    def last_line(self) -> str:
        return next(reversed(self.writer.read().splitlines()))


# test_format: six parametrized cases, each an `add` and a `debug`
message_fmt = Added(format="Message: {message}").expect(product=0)
message_out = Logged(logger=message_fmt.logger, message="a").expect(written="Message: a\n")
nope_fmt    = Added(format="Nope").expect(product=0)
nope_out    = Logged(logger=nope_fmt.logger, message="b").expect(written="Nope\n")
levels_fmt  = Added(format="{level} {message} {level}").expect(product=0)
levels_out  = Logged(logger=levels_fmt.logger, message="c").expect(written="DEBUG c DEBUG\n")
fields_fmt  = Added(format="{message} {level} {level.no} {level.name}").expect(product=0)
fields_out  = Logged(logger=fields_fmt.logger, message="d").expect(written="d DEBUG 10 DEBUG\n")
fn_fmt      = Added(format=lambda _: "{message}").expect(product=0)
fn_out      = Logged(logger=fn_fmt.logger, message="e").expect(written="e")
fn_name_fmt = Added(format=lambda r: "{message} " + r["level"].name).expect(product=0)
fn_name_out = Logged(logger=fn_name_fmt.logger, message="f").expect(written="f DEBUG")


# test_progressive_format: one chain of eleven steps
def progressive(record: Record) -> str:
    fmt = "[{level.name}] {message}"
    if "noend" not in record["extra"]:
        fmt += "\n"
    return fmt


progressive_fmt = Added(format=progressive).expect(product=0)
start   = Logged(logger=progressive_fmt.logger, message="Start: ", extra={"noend": True}).expect(written="[DEBUG] Start: ")
raw     = Opted(logger=start.logger, raw=True).expect(written="[DEBUG] Start: ")
dot_1   = Logged(logger=raw.product, message=".").expect(written="[DEBUG] Start: .")
dot_2   = Logged(logger=dot_1.logger, message=".").expect(written="[DEBUG] Start: ..")
dot_3   = Logged(logger=dot_2.logger, message=".").expect(written="[DEBUG] Start: ...")
dot_4   = Logged(logger=dot_3.logger, message=".").expect(written="[DEBUG] Start: ....")
dot_5   = Logged(logger=dot_4.logger, message=".").expect(written="[DEBUG] Start: .....")
newline = Logged(logger=dot_5.logger, message="\n").expect(written="[DEBUG] Start: .....\n")
cooked  = Opted(logger=newline.logger, raw=False).expect(written="[DEBUG] Start: .....\n")
end     = Logged(logger=cooked.product, message="End").expect(written="[DEBUG] Start: .....\n[DEBUG] End\n")


# test_function_format_without_exception
no_exc_fmt   = Added(format=lambda _: "{message}\n").expect(product=0)
no_exc_error = WithError(logger=no_exc_fmt.logger).expect(written="")
no_exc_out   = Logged(logger=no_exc_error.product, level="ERROR", message="Error!").expect(written="Error!\n")

# test_function_format_with_exception
exc_fmt   = Added(format=lambda _: "{message}\n{exception}").expect(product=0)
exc_error = WithError(logger=exc_fmt.logger).expect(written="")
exc_out   = LoggedLines(logger=exc_error.product, level="ERROR", message="Error!").expect(
    first_line="Error!", last_line="ZeroDivisionError: division by zero"
)

# test_invalid_format: the point of each case is an input of the wrong type
minus_one = Added(format=-1).raises(TypeError, "")  # type: ignore
a_float   = Added(format=3.4).raises(TypeError, "")  # type: ignore
an_object = Added(format=object()).raises(TypeError, "")  # type: ignore

# test_invalid_markups
MARKUPS = "Invalid format, color markups could not be parsed correctly"
open_red      = Added(format="<red>").raises(ValueError, MARKUPS)
close_red     = Added(format="</red>").raises(ValueError, MARKUPS)
crossed_level = Added(format="</level><level>").raises(ValueError, MARKUPS)
bare_close    = Added(format="</>").raises(ValueError, MARKUPS)
unknown_tag   = Added(format="<foobar>").raises(ValueError, MARKUPS)


# test_markup_in_field: two parametrized cases, each an `add`, a `bind` and an `info`
class F:
    def __format__(self, spec: str) -> str:
        return spec


IN_FIELD = "{extra[f]:</>} {extra[f]: <blue> } {message}"
colored_fmt   = Added(format=IN_FIELD, colorize=True).expect(product=0)
colored_bound = Bound(logger=colored_fmt.logger, extra={"f": F()}).expect(written="")
colored_out   = Logged(logger=colored_bound.product, level="INFO", message="Test").expect(written="</>  <blue>  Test\n")
plain_fmt     = Added(format=IN_FIELD, colorize=False).expect(product=0)
plain_bound   = Bound(logger=plain_fmt.logger, extra={"f": F()}).expect(written="")
plain_out     = Logged(logger=plain_bound.product, level="INFO", message="Test").expect(written="</>  <blue>  Test\n")

# test_invalid_format_builtin
builtin_format = Added(format=format).raises(ValueError, " most likely a mistake")
