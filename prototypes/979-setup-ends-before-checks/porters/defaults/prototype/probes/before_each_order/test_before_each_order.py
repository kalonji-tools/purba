# Control: two before_each hooks in one file both run, in order, before the act.
import os

from cases import before_each


def first() -> None:
    os.environ["PROBE_ORDER"] = "first"


def second() -> None:
    os.environ["PROBE_ORDER"] += ",second"


before_each(first)
before_each(second)


def test_order():
    reading = os.environ["PROBE_ORDER"]
    assert reading == "first,second"
