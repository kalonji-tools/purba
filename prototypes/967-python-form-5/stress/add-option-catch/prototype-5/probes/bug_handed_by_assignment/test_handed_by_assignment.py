"""Runner bug: a function the test hands to the code by assignment counts as part of the act.

FORM.md: a function the test defines and hands to the code is a value; its body may hold any code but an assert.
run.py counts a function as handed only when it is an argument of a call. Assigned to an attribute
(a monkeypatch), the same function is "run by the test", so its `raise` is refused.
"""

import io

from cases import case


def test_handed_by_assignment():
    stream = io.StringIO()

    def broken_write(*args):
        raise OSError

    stream.write = broken_write
    assert stream.getvalue() == ""


handed_by_assignment = case(test_handed_by_assignment)


def test_handed_by_setattr():
    stream = io.StringIO()

    def broken_write(*args):
        raise OSError

    setattr(stream, "write", broken_write)
    assert stream.getvalue() == ""


handed_by_setattr = case(test_handed_by_setattr)
