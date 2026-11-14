import re
import sys
import time

from loguru import logger

from cases import before_each, case, raises, use
from loguru_lab import Capture, Writer, reset_logger

before_each(reset_logger)


def broken_sink(m):
    raise ValueError("Error!")


def test_broken_sink_message(enqueue: bool, capsys: Capture = use(Capture)):
    logger.add(broken_sink, catch=True, enqueue=enqueue)
    logger.debug("Oops")
    time.sleep(0.1)

    out, err = capsys.readouterr()
    lines = err.strip().splitlines()

    assert out == ""
    assert lines[-1] == "--- End of logging error ---"


broken_sink_message_enqueued = case(test_broken_sink_message, enqueue=True)
