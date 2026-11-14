"""Seeded mistake: the tester expects the second sink to see counter 0, not 1."""

import itertools
import time
from threading import Barrier, Thread

from cases import before_each, case
from loguru import logger
from loguru_lab import reset_logger
from probe_helpers import NonSafeSink

before_each(reset_logger)


def test_safe_adding_while_logging():
    barrier = Barrier(2)
    counter = itertools.count()
    sink_1 = NonSafeSink(1)
    sink_2 = NonSafeSink(1)
    logger.add(sink_1, format="{message}", catch=False)

    def thread_1():
        barrier.wait()
        logger.info("aaa{}bbb", next(counter))

    def thread_2():
        barrier.wait()
        time.sleep(0.5)
        logger.add(sink_2, format="{message}", catch=False)
        logger.info("ccc{}ddd", next(counter))

    threads = [Thread(target=thread_1), Thread(target=thread_2)]
    for k in range(2):
        threads[k].start()
    for k in range(2):
        threads[k].join()
    logger.remove()

    assert sink_1.written == "aaa0bbb\nccc1ddd\n"
    assert sink_2.written == "ccc0ddd\n"


safe_adding_while_logging = case(test_safe_adding_while_logging)
