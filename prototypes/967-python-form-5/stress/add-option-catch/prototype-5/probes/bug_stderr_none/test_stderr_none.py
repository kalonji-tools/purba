"""Does a case that sets sys.stderr to None break the runner?

run.py calls sys.stderr.flush() before each check's fork and after each case, in the case's process.
Each case appends one line to ledger.txt, so a case that runs twice shows up there.
"""

import os
import sys

from cases import case

LEDGER = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ledger.txt")


def test_stderr_none_one_check():
    sys.stderr = None
    ledger = open(LEDGER, "a")
    ledger.write("one_check\n")
    ledger.close()
    seen = sys.stderr
    assert seen is None


stderr_none_one_check = case(test_stderr_none_one_check)


def test_stderr_none_two_checks():
    sys.stderr = None
    seen = [sys.stderr, 1]
    assert seen[0] is None
    assert seen[1] == 1


stderr_none_two_checks = case(test_stderr_none_two_checks)


def test_after():
    ledger = open(LEDGER, "a")
    ledger.write("after\n")
    ledger.close()
    assert ledger.closed is True


after = case(test_after)
