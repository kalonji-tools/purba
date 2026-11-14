"""The port calls the original's threading.excepthook swap. Does that hide a thread error from the runner?

FORM.md: an error that no code sees, in a thread, fails the case.
"""

import threading
import traceback
import sys

from cases import case


def boom():
    raise ValueError("in the thread")


def test_thread_error():
    worker = threading.Thread(target=boom)
    worker.start()
    worker.join()
    assert worker.is_alive() is False


thread_error = case(test_thread_error)


def test_thread_error_with_the_originals_hook():
    threading.excepthook = lambda args: traceback.print_exception(args.exc_value, file=sys.stderr)
    worker = threading.Thread(target=boom)
    worker.start()
    worker.join()
    assert worker.is_alive() is False


thread_error_with_the_originals_hook = case(test_thread_error_with_the_originals_hook)
