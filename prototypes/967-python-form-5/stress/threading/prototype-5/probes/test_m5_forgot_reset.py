"""Seeded mistake: no `before_each(reset_logger)`, so loguru's default stderr handler stays.

In pytest, `reset_logger` is autouse in conftest.py and a tester cannot forget it.
"""

from cases import case, use
from loguru import logger
from loguru_lab import Capsys


def test_quiet(capsys: Capsys = use(Capsys)):
    logger.info("Some message")
    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


quiet = case(test_quiet)
