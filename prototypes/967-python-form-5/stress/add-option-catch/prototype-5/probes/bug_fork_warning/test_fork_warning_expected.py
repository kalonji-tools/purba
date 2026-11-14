"""Can a tester expect the fork warning with `warns`, as FORM.md offers for a warning?

No: `warns` wraps one statement of the act, and the warning comes later, from the runner's fork before a check.
"""

import threading
import time

from cases import case, warns


def test_expect_the_fork_warning():
    stop = threading.Event()
    threading.Thread(target=stop.wait, daemon=True).start()
    with warns(DeprecationWarning, match=r"multi-threaded"):
        time.sleep(0.01)
    seen = [1, 2]
    assert seen[0] == 1
    assert seen[1] == 2


expect_the_fork_warning = case(test_expect_the_fork_warning)
