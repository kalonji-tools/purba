"""FORM.md: "A refusal names every fault at once."

Each test below holds two faults. Expect both named.
"""

import contextlib


def test_with_and_no_check():
    with contextlib.suppress(ValueError):  # fault 1: a `with` other than raises/warns/handling
        int("1")
    # fault 2: checks nothing


def test_mid_assert_and_not_form():
    x = int("1")
    assert x == 1  # fault 1: an assert before the end
    y = bool(x)
    assert not y  # fault 2: a check in another form (not a comparison)
