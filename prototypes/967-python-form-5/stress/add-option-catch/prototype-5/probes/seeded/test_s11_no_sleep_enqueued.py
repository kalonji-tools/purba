import re
import sys
import time

from loguru import logger

from cases import before_each, case, raises, use
from loguru_lab import Capture, Writer, reset_logger

before_each(reset_logger)


def broken_sink(m):
    raise ValueError("Error!")


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

    assert output == "A\nB\n"


broken_sink_caught_keep_working_enqueued = case(test_broken_sink_caught_keep_working, enqueue=True)
