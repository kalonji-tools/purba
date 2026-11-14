"""Experiment (control): a thread writes to sys.stderr. The port's `err == ""` must fail."""

import sys
from threading import Thread

from cases import before_each, case, use
from loguru_lab import Capsys, reset_logger

before_each(reset_logger)


def test_noisy(capsys: Capsys = use(Capsys)):
    def threaded():
        print("noise", file=sys.stderr)

    thread = Thread(target=threaded)
    thread.start()
    thread.join()

    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


noisy = case(test_noisy)
