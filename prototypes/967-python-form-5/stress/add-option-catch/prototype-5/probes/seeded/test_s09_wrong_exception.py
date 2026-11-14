import re
import sys
import time

from loguru import logger

from cases import before_each, case, raises, use
from loguru_lab import Capture, Writer, reset_logger

before_each(reset_logger)


def broken_sink(m):
    raise ValueError("Error!")


def test_catch_is_false(capsys: Capture = use(Capture)):
    logger.add(broken_sink, catch=False)
    with raises(TypeError, match="Error!"):
        logger.debug("Fail")
    assert capsys.readouterr() == ("", "")


catch_is_false = case(test_catch_is_false)
