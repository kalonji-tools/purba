"""Shared by the probes: the original's NonSafeSink, and a helper that loops over its argument."""

import time
from threading import Thread


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


def start_and_join(threads: list[Thread]) -> None:
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
