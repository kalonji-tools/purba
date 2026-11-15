"""The act has one path, seen while the case runs. Each line of the test module's code that the
test calls runs in every case, each branch goes both ways or is not there, and each loop runs a
round. An assert that runs in the act, and a line that runs after an expected raise, are refused
too."""

from __future__ import annotations

import threading
from contextlib import suppress

from cases import case, raises
from helpers import check_dir, start_all
from undertest import emit, parse


def module_helper(flag: bool) -> int:
    if flag:
        return 1
    return 2


class Box:
    def __init__(self, v: int | None) -> None:
        if v is None:
            v = 0
        self.v = v


def test_join_threads(n: int):
    """two_threads PASS, no_threads REFUSED: the `join` line never runs. `start_all` sits in
    helpers.py, which the watch does not cover, so its loop is not named. Says "line 36 of
    test_paths.py never ran" """
    threads = [threading.Thread(target=lambda: None) for _ in range(n)]
    start_all(threads)
    for t in threads:
        t.join()
    assert len(threads) == n


two_threads = case(test_join_threads, n=2)
no_threads = case(test_join_threads, n=0)


def test_module_helper_branch():
    """REFUSED: module_helper's `if` goes one way. Says "the branch at test_paths.py:17
    (module_helper, reached from line 47 of the test) went one way" """
    got = module_helper(True)
    assert got == 1


def test_class_branch():
    """REFUSED: Box.__init__'s `if` goes one way."""
    box = Box(None)
    assert box.v == 0


def test_ternary(flag: bool = True):
    """REFUSED: the `x if c else y` goes one way."""
    y = 1 if flag else 2
    assert y == 1


def test_or_default(name: str = ""):
    """REFUSED: the `or` goes one way."""
    shown = name or "anonymous"
    assert shown == "anonymous"


def test_callback_branches():
    """PASS: `sink` is called by the code under test, so its `if` is the code's to run."""
    seen: list[str] = []

    def sink(m: str) -> None:
        if m.startswith("!"):
            seen.append("loud")
        else:
            seen.append("quiet")

    emit(sink, "!hi")
    assert seen == ["loud"]


def test_callback_by_assignment():
    """PASS: handed by assignment, then called by the code. (A check that calls `stream.write`
    itself makes it part of the act: the test calls it.)"""

    class Stream:
        write = None

    stream = Stream()
    sizes: list[int] = []

    def write(m: str) -> None:
        if m:
            sizes.append(len(m))
        else:
            sizes.append(0)

    stream.write = write  # type: ignore[assignment]
    emit(stream.write, "x")
    assert sizes == [1]


def test_branch_both_ways_in_a_loop(items: list[int]):
    """both_ways PASS, one_way REFUSED: for [1] the `if` goes one way."""
    odd = []
    for i in items:
        if i % 2:
            odd.append(i)
    assert odd == [1]


both_ways = case(test_branch_both_ways_in_a_loop, items=[1, 2])
one_way = case(test_branch_both_ways_in_a_loop, items=[1])


def test_comprehension_over_nothing(items: list[int]):
    """REFUSED: a one-line comprehension over [] runs no round."""
    doubled = [i * 2 for i in items]
    assert doubled == []


empty = case(test_comprehension_over_nothing, items=[])


def test_try_in_the_act():
    """REFUSED: the `except` matches one way, and the `try` body's last line never runs."""
    got = 0
    try:
        got = parse("x")
        got += 1
    except ValueError:
        got = -1
    assert got == -1


def test_raise_swallowed_at_the_last_line():
    """PASS: a `raise` may sit in the act. The setup swallows it at its last line."""
    log: list[str] = []
    with suppress(ValueError):
        log.append("before")
        raise ValueError("swallowed")
    assert log == ["before"]


def test_assert_in_a_helper():
    """PASS: check_dir's assert runs in the act, but check_dir sits in helpers.py, which the
    watch does not cover."""
    names = ["a.log"]
    check_dir(names)
    assert names == ["a.log"]


def test_assert_in_a_callback():
    """REFUSED: the callback's assert runs, and the code under test could swallow it. Says "an
    assert ran in the act at test_paths.py:160 (sink, reached from line 163 of the test)" """
    seen: list[str] = []

    def sink(m: str) -> None:
        assert m
        seen.append(m)

    emit(sink, "x")
    assert seen == ["x"]


def test_line_after_the_raise():
    """REFUSED: `log.append` runs after `with raises(…)` caught."""
    log: list[str] = []
    with raises(ValueError) as caught:
        parse("x")
    log.append("after")
    assert str(caught.value) == "invalid literal for int() with base 10: 'x'"


def test_checks_nothing():
    """REFUSED: no check ran."""
    parse("1")


def test_expected_by_operators():
    """PASS: `"7" * 20 + "\\n"` is operators on literals."""
    text = "7" * 20 + "\n"
    assert text == "7" * 20 + "\n"


def test_callback_called_by_c():
    """REFUSED, a hole: `sorted` is C, so it leaves no frame, and the key looks called by the
    test. Its `if` goes one way. The same key handed to Python code that calls it is free
    (test_callback_branches)."""

    def key(word: str) -> int:
        if word.startswith("!"):
            return 0
        return len(word)

    ordered = sorted(["bb", "a"], key=key)
    assert ordered == ["a", "bb"]
