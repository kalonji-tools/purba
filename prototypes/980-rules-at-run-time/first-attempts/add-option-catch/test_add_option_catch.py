import re
import sys
import time

from loguru import logger

from cases import before_each, case, raises, use
from loguru_lab import Capture, Writer, default_threading_excepthook, reset_logger

before_each(reset_logger)


def broken_sink(m):
    raise ValueError("Error!")


def test_catch_is_true(capsys: Capture = use(Capture)):
    logger.add(broken_sink, catch=True)
    logger.debug("Fail")
    out, err = capsys.readouterr()
    assert out == ""
    assert err != ""


catch_is_true = case(test_catch_is_true)


def test_catch_is_false(capsys: Capture = use(Capture)):
    logger.add(broken_sink, catch=False)
    with raises(ValueError, match="Error!"):
        logger.debug("Fail")
    out, err = capsys.readouterr()
    assert out == err == ""


catch_is_false = case(test_catch_is_false)


def test_no_sys_stderr(capsys: Capture = use(Capture)):
    sys.stderr = None
    logger.add(broken_sink, catch=True)
    logger.debug("a")

    out, err = capsys.readouterr()
    assert out == err == ""


no_sys_stderr = case(test_no_sys_stderr)


def test_broken_sys_stderr(capsys: Capture = use(Capture)):
    def broken_write(*args, **kwargs):
        raise OSError

    sys.stderr.write = broken_write
    logger.add(broken_sink, catch=True)
    logger.debug("a")

    out, err = capsys.readouterr()
    assert out == err == ""


broken_sys_stderr = case(test_broken_sys_stderr)


def test_encoding_error(capsys: Capture = use(Capture)):
    def sink(m):
        raise UnicodeEncodeError("utf8", "", 10, 11, "too bad")

    logger.add(sink, catch=True)
    logger.debug("test")

    out, err = capsys.readouterr()
    lines = err.strip().splitlines()

    assert out == ""
    assert lines[0] == "--- Logging error in Loguru Handler #0 ---"
    assert lines[1].startswith("Record was: {")
    assert lines[1].endswith("}")
    assert lines[-2].startswith("UnicodeEncodeError:")
    assert lines[-1] == "--- End of logging error ---"


encoding_error = case(test_encoding_error)


def test_unprintable_record(writer: Writer = use(Writer), capsys: Capture = use(Capture)):
    class Unprintable:
        def __repr__(self):
            raise ValueError("Failed")

    logger.add(writer, format="{message} {extra[unprintable]}", catch=True)
    logger.bind(unprintable=1).debug("a")
    logger.bind(unprintable=Unprintable()).debug("b")
    logger.bind(unprintable=2).debug("c")

    out, err = capsys.readouterr()
    lines = err.strip().splitlines()

    assert writer.read() == "a 1\nc 2\n"
    assert out == ""
    assert lines[0] == "--- Logging error in Loguru Handler #0 ---"
    assert lines[1] == "Record was: /!\\ Unprintable record /!\\"
    assert lines[-2] == "ValueError: Failed"
    assert lines[-1] == "--- End of logging error ---"


unprintable_record = case(test_unprintable_record)


def test_broken_sink_message(enqueue: bool, capsys: Capture = use(Capture)):
    logger.add(broken_sink, catch=True, enqueue=enqueue)
    logger.debug("Oops")
    time.sleep(0.1)

    out, err = capsys.readouterr()
    lines = err.strip().splitlines()

    assert out == ""
    assert lines[0] == "--- Logging error in Loguru Handler #0 ---"
    assert re.match(r"Record was: \{.*Oops.*\}", lines[1])
    assert lines[-2].startswith("ValueError: Error!")
    assert lines[-1] == "--- End of logging error ---"


broken_sink_message_direct = case(test_broken_sink_message, enqueue=False)
broken_sink_message_enqueued = case(test_broken_sink_message, enqueue=True)


def test_broken_sink_caught_keep_working(enqueue: bool):
    output = ""

    def half_broken_sink(m):
        nonlocal output
        if m.startswith("NOK"):
            raise ValueError("Broken!")
        output += m

    logger.add(half_broken_sink, format="{message}", enqueue=enqueue, catch=True)
    logger.info("A")
    logger.info("NOK")
    logger.info("B")

    time.sleep(0.1)
    assert output == "A\nB\n"


broken_sink_caught_keep_working_direct = case(test_broken_sink_caught_keep_working, enqueue=False)
broken_sink_caught_keep_working_enqueued = case(test_broken_sink_caught_keep_working, enqueue=True)


def test_broken_sink_not_caught_enqueue():
    called = 0

    def broken_sink(m):
        nonlocal called
        called += 1
        raise ValueError("Nop")

    logger.add(broken_sink, format="{message}", enqueue=True, catch=False)

    with default_threading_excepthook():
        logger.info("A")
        logger.info("B")
        time.sleep(0.1)

    assert called == 2


broken_sink_not_caught_enqueue = case(test_broken_sink_not_caught_enqueue)
