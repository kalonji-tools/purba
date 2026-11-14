import re
import sys
import time

from loguru import logger

from cases import before_each, case, raises, use
from loguru_lab import Capture, Writer, reset_logger

before_each(reset_logger)


def broken_sink(m):
    raise ValueError("Error!")


def test_unprintable_record(writer: Writer = use(Writer()), capsys: Capture = use(Capture)):
    logger.add(writer, format="{message} {extra[unprintable]}", catch=True)
    logger.bind(unprintable=1).debug("a")
    assert writer.read() == "a 1\n"


unprintable_record = case(test_unprintable_record)
