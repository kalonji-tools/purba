"""Experiment: the thread loops moved into a helper in the test file, which the test calls itself.

FORM.md: a function that the test calls itself, under any name, is part of the act, and has one path.
"""

from threading import Thread

from cases import case


def start_and_join(threads):
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()


def test_threads():
    seen = []
    threads = [Thread(target=seen.append, args=(1,)) for _ in range(2)]
    start_and_join(threads)
    assert seen == [1, 1]


two_threads = case(test_threads)
