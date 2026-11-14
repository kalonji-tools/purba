"""Control for e3: the same helper defined inside the test, and called by it. The runner should refuse it."""

from threading import Thread

from cases import case


def test_threads():
    def start_and_join(threads):
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()

    seen = []
    threads = [Thread(target=seen.append, args=(1,)) for _ in range(2)]
    start_and_join(threads)
    assert seen == [1, 1]


two_threads = case(test_threads)
