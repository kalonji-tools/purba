"""pytest twins of probes e1 (a sink raises in a thread) and m5 (no reset), run under the original conftest."""

import time
from threading import Thread

from loguru import logger


class NonSafeSink:
    def __init__(self, sleep_time, stop_time=0):
        self.sleep_time = sleep_time
        self.stop_time = stop_time
        self.written = ""
        self.stopped = False

    def write(self, message):
        if self.stopped:
            raise RuntimeError("Can't write on stopped sink")
        self.written += message
        time.sleep(self.sleep_time)


def test_stopped_sink(capsys):
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
