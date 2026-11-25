"""Port of loguru's tests/test_add_option_format.py (MIT, Delgan/loguru) to purba.

purba has no fixtures and no parametrize: each parametrized case is a test of
its own, and the conftest fixtures `writer` and `reset_logger` are helpers that
each test calls. See NOTES.md.
"""

import re

import loguru
from loguru import logger


def reset_logger():
    loguru.logger.remove()
    loguru.logger.__init__(
        loguru._logger.Core(), None, 0, False, False, False, False, True, [], {}
    )
    loguru._logger.context.set({})


def make_writer():
    reset_logger()

    def w(message):
        w.written.append(message)

    w.written = []
    w.read = lambda: "".join(w.written)
    w.clear = lambda: w.written.clear()

    return w


def raised(action):
    """Run `action` and return the exception it raises, or None."""
    try:
        action()
    except Exception as error:  # noqa: BLE001
        return error


# test_format, one test per parametrize row


def test_format_message_field():
    writer = make_writer()
    logger.add(writer, format="Message: {message}")
    logger.debug("a")
    assert writer.read() == "Message: a\n"


def test_format_without_field():
    writer = make_writer()
    logger.add(writer, format="Nope")
    logger.debug("b")
    assert writer.read() == "Nope\n"


def test_format_level_twice():
    writer = make_writer()
    logger.add(writer, format="{level} {message} {level}")
    logger.debug("c")
    assert writer.read() == "DEBUG c DEBUG\n"


def test_format_level_attributes():
    writer = make_writer()
    logger.add(writer, format="{message} {level} {level.no} {level.name}")
    logger.debug("d")
    assert writer.read() == "d DEBUG 10 DEBUG\n"


def test_format_function_constant():
    writer = make_writer()
    logger.add(writer, format=lambda _: "{message}")
    logger.debug("e")
    assert writer.read() == "e"


def test_format_function_reads_record():
    writer = make_writer()
    logger.add(writer, format=lambda r: "{message} " + r["level"].name)
    logger.debug("f")
    assert writer.read() == "f DEBUG"


def test_progressive_format():
    writer = make_writer()

    def formatter(record):
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


def test_function_format_without_exception():
    writer = make_writer()
    logger.add(writer, format=lambda _: "{message}\n")
    try:
        1 / 0  # noqa: B018
    except ZeroDivisionError:
        logger.exception("Error!")
    assert writer.read() == "Error!\n"


def test_function_format_with_exception():
    writer = make_writer()
    logger.add(writer, format=lambda _: "{message}\n{exception}")
    try:
        1 / 0  # noqa: B018
    except ZeroDivisionError:
        logger.exception("Error!")
    lines = writer.read().splitlines()
    assert lines[0] == "Error!"
    assert lines[-1] == "ZeroDivisionError: division by zero"


# test_invalid_format, one test per parametrize row


def test_invalid_format_negative_int():
    writer = make_writer()
    error = raised(lambda: logger.add(writer, format=-1))
    assert isinstance(error, TypeError)


def test_invalid_format_float():
    writer = make_writer()
    error = raised(lambda: logger.add(writer, format=3.4))
    assert isinstance(error, TypeError)


def test_invalid_format_object():
    writer = make_writer()
    error = raised(lambda: logger.add(writer, format=object()))
    assert isinstance(error, TypeError)


# test_invalid_markups, one test per parametrize row

INVALID_MARKUPS = r"^Invalid format, color markups could not be parsed correctly$"


def test_invalid_markups_open_red():
    writer = make_writer()
    error = raised(lambda: logger.add(writer, format="<red>"))
    assert isinstance(error, ValueError)
    assert re.search(INVALID_MARKUPS, str(error)) is not None


def test_invalid_markups_close_red():
    writer = make_writer()
    error = raised(lambda: logger.add(writer, format="</red>"))
    assert isinstance(error, ValueError)
    assert re.search(INVALID_MARKUPS, str(error)) is not None


def test_invalid_markups_close_then_open_level():
    writer = make_writer()
    error = raised(lambda: logger.add(writer, format="</level><level>"))
    assert isinstance(error, ValueError)
    assert re.search(INVALID_MARKUPS, str(error)) is not None


def test_invalid_markups_close_anonymous():
    writer = make_writer()
    error = raised(lambda: logger.add(writer, format="</>"))
    assert isinstance(error, ValueError)
    assert re.search(INVALID_MARKUPS, str(error)) is not None


def test_invalid_markups_unknown_tag():
    writer = make_writer()
    error = raised(lambda: logger.add(writer, format="<foobar>"))
    assert isinstance(error, ValueError)
    assert re.search(INVALID_MARKUPS, str(error)) is not None


# test_markup_in_field, one test per parametrize row


def test_markup_in_field_colorize():
    writer = make_writer()

    class F:
        def __format__(self, spec):
            return spec

    logger.add(writer, format="{extra[f]:</>} {extra[f]: <blue> } {message}", colorize=True)
    logger.bind(f=F()).info("Test")

    assert writer.read() == "</>  <blue>  Test\n"


def test_markup_in_field_no_colorize():
    writer = make_writer()

    class F:
        def __format__(self, spec):
            return spec

    logger.add(writer, format="{extra[f]:</>} {extra[f]: <blue> } {message}", colorize=False)
    logger.bind(f=F()).info("Test")

    assert writer.read() == "</>  <blue>  Test\n"


def test_invalid_format_builtin():
    writer = make_writer()
    error = raised(lambda: logger.add(writer, format=format))
    assert isinstance(error, ValueError)
    assert re.search(r".* most likely a mistake", str(error)) is not None
