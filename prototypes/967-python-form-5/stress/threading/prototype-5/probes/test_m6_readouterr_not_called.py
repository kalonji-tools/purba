"""Seeded mistake: `capsys.readouterr` without the call."""

from cases import before_each, case, use
from loguru import logger
from loguru_lab import Capsys, reset_logger

before_each(reset_logger)


def test_quiet(capsys: Capsys = use(Capsys)):
    logger.add(lambda _: None, format="{message}", catch=False)
    logger.info("Some message")
    out, err = capsys.readouterr
    assert out == ""
    assert err == ""


quiet = case(test_quiet)
