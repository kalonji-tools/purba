"""loguru's tests/test_threading.py, ported to the prototype form. MIT, Delgan/loguru@28349be; see LICENSE.loguru.

Each `for thread in threads:` of the original became `for k in range(N): threads[k]...`: the form loops over literal data only.
"""

import itertools
import time
from threading import Barrier, Thread

from cases import before_each, case, use
from loguru import logger
from loguru_lab import Capsys, Writer, reset_logger

before_each(reset_logger)


class NonSafeSink:
    def __init__(self, sleep_time: float, stop_time: float = 0) -> None:
        self.sleep_time = sleep_time
        self.stop_time = stop_time
        self.written = ""
        self.stopped = False

    def write(self, message: str) -> None:
        if self.stopped:
            raise RuntimeError("Can't write on stopped sink")

        length = len(message)
        self.written += message[:length]
        time.sleep(self.sleep_time)
        self.written += message[length:]

    def stop(self) -> None:
        time.sleep(self.stop_time)
        self.stopped = True


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

    for k in range(2):
        threads[k].join()

    logger.remove()

    assert sink.written in ("___0___\n___1___\n", "___1___\n___0___\n")


safe_logging = case(test_safe_logging)


def test_safe_adding_while_logging(writer: Writer = use(Writer)):
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
    assert sink_2.written == "ccc1ddd\n"


safe_adding_while_logging = case(test_safe_adding_while_logging)


def test_safe_removing_while_logging(capsys: Capsys = use(Capsys)):
    barrier = Barrier(2)
    counter = itertools.count()

    sink = NonSafeSink(1)
    i = logger.add(sink, format="{message}", catch=False)

    def thread_1():
        barrier.wait()
        logger.info("aaa{}bbb", next(counter))

    def thread_2():
        barrier.wait()
        time.sleep(0.5)
        logger.remove(i)
        logger.info("ccc{}ddd", next(counter))

    threads = [Thread(target=thread_1), Thread(target=thread_2)]

    for k in range(2):
        threads[k].start()

    for k in range(2):
        threads[k].join()

    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""
    assert sink.written == "aaa0bbb\n"


safe_removing_while_logging = case(test_safe_removing_while_logging)


def test_safe_removing_all_while_logging(capsys: Capsys = use(Capsys)):
    barrier = Barrier(2)

    for _ in range(1000):
        logger.add(lambda _: None, format="{message}", catch=False)

    def thread_1():
        barrier.wait()
        logger.remove()

    def thread_2():
        barrier.wait()
        for _ in range(100):
            logger.info("Some message")

    threads = [Thread(target=thread_1), Thread(target=thread_2)]

    for k in range(2):
        threads[k].start()

    for k in range(2):
        threads[k].join()

    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


safe_removing_all_while_logging = case(test_safe_removing_all_while_logging)


def test_safe_slow_removing_all_while_logging(capsys: Capsys = use(Capsys)):
    barrier = Barrier(2)

    for _ in range(10):
        sink = NonSafeSink(0.1, 0.1)
        logger.add(sink, format="{message}", catch=False)

    def thread_1():
        barrier.wait()
        logger.remove()

    def thread_2():
        barrier.wait()
        time.sleep(0.5)
        logger.info("Some message")

    threads = [Thread(target=thread_1), Thread(target=thread_2)]

    for k in range(2):
        threads[k].start()

    for k in range(2):
        threads[k].join()

    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


safe_slow_removing_all_while_logging = case(test_safe_slow_removing_all_while_logging)


def test_safe_writing_after_removing(capsys: Capsys = use(Capsys)):
    barrier = Barrier(2)

    logger.add(NonSafeSink(1), format="{message}", catch=False)
    i = logger.add(NonSafeSink(1), format="{message}", catch=False)

    def write():
        barrier.wait()
        logger.info("Writing")

    def remove():
        barrier.wait()
        time.sleep(0.5)
        logger.remove(i)

    threads = [Thread(target=write), Thread(target=remove)]

    for k in range(2):
        threads[k].start()

    for k in range(2):
        threads[k].join()

    logger.remove()

    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


safe_writing_after_removing = case(test_safe_writing_after_removing)


def test_heavily_threaded_logging(capsys: Capsys = use(Capsys)):
    logger.remove()

    def function():
        i = logger.add(NonSafeSink(0.1), format="{message}", catch=False)
        logger.debug("AAA")
        logger.info("BBB")
        logger.success("CCC")
        logger.remove(i)

    threads = [Thread(target=function) for _ in range(10)]

    for k in range(10):
        threads[k].start()

    for k in range(10):
        threads[k].join()

    logger.remove()

    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


heavily_threaded_logging = case(test_heavily_threaded_logging)
