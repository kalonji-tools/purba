"""Experiment: the same helper as e3, imported from another module."""

from threading import Thread

from cases import case
from probe_helpers import start_and_join


def test_threads():
    seen = []
    threads = [Thread(target=seen.append, args=(1,)) for _ in range(2)]
    start_and_join(threads)
    assert seen == [1, 1]


two_threads = case(test_threads)
