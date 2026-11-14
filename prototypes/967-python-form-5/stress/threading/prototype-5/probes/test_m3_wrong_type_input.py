"""Seeded mistake: the case passes a str where the test takes a Writer."""

from cases import before_each, case, use
from loguru import logger
from loguru_lab import Writer, reset_logger

before_each(reset_logger)


def test_writes(writer: Writer = use(Writer)):
    logger.add(writer, format="{message}", catch=False)
    logger.info("abc")
    assert writer.read() == "abc\n"


writes = case(test_writes, writer="writer")
