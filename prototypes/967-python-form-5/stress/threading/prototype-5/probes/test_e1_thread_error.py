"""Experiment: a sink raises in a thread with catch=False. The original's `err == ""` would see the traceback."""

from threading import Thread

from cases import before_each, case, use
from loguru import logger
from loguru_lab import Capsys, reset_logger
from probe_helpers import NonSafeSink

before_each(reset_logger)


def test_stopped_sink(capsys: Capsys = use(Capsys)):
    sink = NonSafeSink(0)
    sink.stopped = True
    logger.add(sink, format="{message}", catch=False)

    def threaded():
        logger.info("Some message")

    thread = Thread(target=threaded)
    thread.start()
    thread.join()

    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


stopped_sink = case(test_stopped_sink)
