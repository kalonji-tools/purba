"""A refusal for a line of a helper names the test's line that calls the helper, and the helper's
own line."""

from __future__ import annotations


def pick(flag: bool) -> int:
    if flag:
        return 1
    return 2


def pick_twice(flag: bool) -> int:
    return pick(flag) + pick(flag)


def test_helper_branch() -> None:
    """REFUSED. Says "the branch at test_helper_lines.py:8 (pick, reached from line 21 of the test)
    went one way" Says "line 10 of test_helper_lines.py (pick, reached from line 21 of the test)
    never ran" """
    got = pick(True)
    assert got == 1


def test_helper_of_a_helper() -> None:
    """REFUSED: `pick` runs inside `pick_twice`; the line named is the test's. Says "the branch at
    test_helper_lines.py:8 (pick, reached from line 28 of the test) went one way" """
    total = pick_twice(False)
    assert total == 4


def test_helper_in_a_reading() -> None:
    """REFUSED: the helper runs inside the reading of the check, so the line named is the check's.
    Says "the branch at test_helper_lines.py:8 (pick, reached from line 37 of the test) went one
    way" """
    flag = True
    assert pick(flag) == 1
