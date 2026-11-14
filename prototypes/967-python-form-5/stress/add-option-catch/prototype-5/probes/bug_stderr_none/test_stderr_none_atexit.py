"""After a case, run.py calls sys.stderr.flush() outside its try, before os._exit(0).

With sys.stderr None the case's process leaves by an uncaught AttributeError instead,
so it runs the atexit handlers that os._exit skips. Each case registers one; the ledger shows which ran.
"""

import atexit
import os
import sys

from cases import case

LEDGER = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ledger_atexit.txt")


def note(text):
    ledger = open(LEDGER, "a")
    ledger.write(text + "\n")
    ledger.close()


def test_atexit_with_stderr(label: str):
    atexit.register(note, label)
    seen = [label]
    assert seen == [label]


def test_atexit_without_stderr(label: str):
    atexit.register(note, label)
    sys.stderr = None
    seen = [label]
    assert seen == [label]


control = case(test_atexit_with_stderr, label="control: stderr kept")
stderr_none = case(test_atexit_without_stderr, label="stderr None")
