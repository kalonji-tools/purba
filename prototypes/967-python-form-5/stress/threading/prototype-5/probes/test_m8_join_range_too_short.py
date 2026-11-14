"""Seeded mistake born of the workaround: the join loop says range(1), so one of the two threads is not joined."""

import itertools
from threading import Barrier, Thread

from cases import before_each, case
from loguru import logger
from loguru_lab import reset_logger
from probe_helpers import NonSafeSink

before_each(reset_logger)


def test_safe_logging():
    barrier = Barrier(2)
    counter = itertools.count()
    sink = NonSafeSink(1)
    logger.add(sink, format="{message}", catch=False)

    def threaded():
        barrier.wait()
        logger.info("___{}___", next(counter))

    threads = [Thread(target=threaded) for _ in range(2)]
    for k in range(2):
        threads[k].start()
    for k in range(1):
        threads[k].join()
    logger.remove()

    assert sink.written in ("___0___\n___1___\n", "___1___\n___0___\n")


safe_logging = case(test_safe_logging)
