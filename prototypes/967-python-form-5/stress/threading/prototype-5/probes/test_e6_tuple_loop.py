"""Experiment: loops over a tuple display of the two threads, and a comprehension over the list."""

from threading import Thread

from cases import case


def test_tuple_loop():
    seen = []
    first = Thread(target=seen.append, args=(1,))
    second = Thread(target=seen.append, args=(2,))
    for thread in (first, second):
        thread.start()
    for thread in (first, second):
        thread.join()
    assert sorted(seen) == [1, 2]


tuple_loop = case(test_tuple_loop)


def test_comprehension():
    seen = []
    threads = [Thread(target=seen.append, args=(1,)) for _ in range(2)]
    [thread.start() for thread in threads]
    [thread.join() for thread in threads]
    assert seen == [1, 1]


comprehension = case(test_comprehension)
