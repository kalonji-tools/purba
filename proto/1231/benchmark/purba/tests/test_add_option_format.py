import re

import loguru
from loguru import logger


# Port of loguru's tests/test_add_option_format.py (MIT, Delgan/loguru@28349be).
# purba has no fixtures and no parametrize: the two conftest fixtures are the
# helpers below, and each parametrized case is a test of its own.


def reset_logger():
    """The body of conftest's autouse fixture reset_logger; each test calls it first."""
    loguru.logger.remove()
    loguru.logger.__init__(
        loguru._logger.Core(), None, 0, False, False, False, False, True, [], {}
    )
    loguru._logger.context.set({})


def make_writer():
    """conftest's fixture writer, built as a helper."""

    def w(message):
        w.written.append(message)

    w.written = []
    w.read = lambda: "".join(w.written)
    w.clear = lambda: w.written.clear()

    return w


def write_with_format(message, format):
    """The act of the original test_format: one message through one format."""
    reset_logger()
    writer = make_writer()
    logger.add(writer, format=format)
    logger.debug(message)
    return writer.read()


def add_error(format, exception_type):
    """Stands in for pytest.raises: the exception logger.add raises, or None.

    Written without a second `return` so that every line runs ([second-path]).
    """
    reset_logger()
    writer = make_writer()
    error = None
    try:
        logger.add(writer, format=format)
    except exception_type as caught:
        error = caught
    return error


# test_format, one test per case of its parametrize.


def test_format_a():
    assert write_with_format("a", "Message: {message}") == "Message: a\n"


def test_format_b():
    assert write_with_format("b", "Nope") == "Nope\n"


def test_format_c():
    assert write_with_format("c", "{level} {message} {level}") == "DEBUG c DEBUG\n"


def test_format_d():
    assert (
        write_with_format("d", "{message} {level} {level.no} {level.name}")
        == "d DEBUG 10 DEBUG\n"
    )


def test_format_e():
    assert write_with_format("e", lambda _: "{message}") == "e"


def test_format_f():
    assert write_with_format("f", lambda r: "{message} " + r["level"].name) == "f DEBUG"


def test_progressive_format():
    reset_logger()
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
    reset_logger()
    writer = make_writer()
    logger.add(writer, format=lambda _: "{message}\n")
    try:
        1 / 0  # noqa: B018
    except ZeroDivisionError:
        logger.exception("Error!")
    assert writer.read() == "Error!\n"


def test_function_format_with_exception():
    reset_logger()
    writer = make_writer()
    logger.add(writer, format=lambda _: "{message}\n{exception}")
    try:
        1 / 0  # noqa: B018
    except ZeroDivisionError:
        logger.exception("Error!")
    lines = writer.read().splitlines()
    assert lines[0] == "Error!"
    assert lines[-1] == "ZeroDivisionError: division by zero"


# test_invalid_format, one test per case of its parametrize.


def test_invalid_format_int():
    assert isinstance(add_error(-1, TypeError), TypeError)


def test_invalid_format_float():
    assert isinstance(add_error(3.4, TypeError), TypeError)


def test_invalid_format_object():
    assert isinstance(add_error(object(), TypeError), TypeError)


# test_invalid_markups, one test per case of its parametrize.

MARKUP_ERROR = r"^Invalid format, color markups could not be parsed correctly$"


def test_invalid_markups_red():
    error = add_error("<red>", ValueError)
    assert isinstance(error, ValueError)
    assert re.search(MARKUP_ERROR, str(error)) is not None


def test_invalid_markups_red_close():
    error = add_error("</red>", ValueError)
    assert isinstance(error, ValueError)
    assert re.search(MARKUP_ERROR, str(error)) is not None


def test_invalid_markups_level_close_open():
    error = add_error("</level><level>", ValueError)
    assert isinstance(error, ValueError)
    assert re.search(MARKUP_ERROR, str(error)) is not None


def test_invalid_markups_close():
    error = add_error("</>", ValueError)
    assert isinstance(error, ValueError)
    assert re.search(MARKUP_ERROR, str(error)) is not None


def test_invalid_markups_foobar():
    error = add_error("<foobar>", ValueError)
    assert isinstance(error, ValueError)
    assert re.search(MARKUP_ERROR, str(error)) is not None


# test_markup_in_field, one test per case of its parametrize.


def markup_in_field(colorize):
    reset_logger()
    writer = make_writer()

    class F:
        def __format__(self, spec):
            return spec

    logger.add(writer, format="{extra[f]:</>} {extra[f]: <blue> } {message}", colorize=colorize)
    logger.bind(f=F()).info("Test")
    return writer.read()


def test_markup_in_field_colorize():
    assert markup_in_field(True) == "</>  <blue>  Test\n"


def test_markup_in_field_no_colorize():
    assert markup_in_field(False) == "</>  <blue>  Test\n"


def test_invalid_format_builtin():
    error = add_error(format, ValueError)
    assert isinstance(error, ValueError)
    assert re.search(r".* most likely a mistake", str(error)) is not None
