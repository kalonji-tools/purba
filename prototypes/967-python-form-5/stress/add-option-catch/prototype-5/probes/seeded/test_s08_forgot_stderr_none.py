import re
import sys
import time

from loguru import logger

from cases import before_each, case, raises, use
from loguru_lab import Capture, Writer, reset_logger

before_each(reset_logger)


def broken_sink(m):
    raise ValueError("Error!")


def test_no_sys_stderr(capsys: Capture = use(Capture)):
    logger.add(broken_sink, catch=True)
    logger.debug("a")

    out, err = capsys.readouterr()
    assert (out, err) == ("", "")


no_sys_stderr = case(test_no_sys_stderr)
