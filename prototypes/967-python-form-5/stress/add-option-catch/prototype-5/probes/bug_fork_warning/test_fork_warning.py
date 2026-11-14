"""Runner bug: the runner's own fork, for each check but the last, warns when a thread is alive.

The test raises no warning. Python 3.12 warns in os.fork() when the process has a second thread.
The runner records that warning inside the case and blames the test.
"""

import threading

from cases import case


def test_two_checks_with_a_live_thread():
    stop = threading.Event()
    threading.Thread(target=stop.wait, daemon=True).start()
    seen = [1, 2]
    assert seen[0] == 1
    assert seen[1] == 2


two_checks_with_a_live_thread = case(test_two_checks_with_a_live_thread)


def test_one_check_with_a_live_thread():
    stop = threading.Event()
    threading.Thread(target=stop.wait, daemon=True).start()
    seen = [1, 2]
    assert seen == [1, 2]


one_check_with_a_live_thread = case(test_one_check_with_a_live_thread)


def test_two_checks_after_the_thread_ends():
    stop = threading.Event()
    worker = threading.Thread(target=stop.wait, daemon=True)
    worker.start()
    stop.set()
    worker.join()
    seen = [1, 2]
    assert seen[0] == 1
    assert seen[1] == 2


two_checks_after_the_thread_ends = case(test_two_checks_after_the_thread_ends)
