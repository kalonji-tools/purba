"""#1025, change 3: a check never reads a one-shot iterator. Before a check reads in place, the
runner looks at each name it loads, and at the value of an `all()` or `any()`'s reading; an
iterator that one reading empties refuses the case, even when one check reads it."""

from __future__ import annotations

import tempfile
from pathlib import Path

from lazy import numbers, parse
from shop import Shop


def test_read_once() -> None:
    """REFUSED: one check reads the iterator. Says "line 20 reads users, a list_iterator that one
    reading empties; make it a list in the act" """
    shop = Shop()
    shop.sign_up("ann")
    users = iter(shop.users)
    assert list(users) == ["ann"]


def test_read_twice() -> None:
    """REFUSED: each check is refused before it reads. Says "line 29 reads users" Says "line 30
    reads users" """
    shop = Shop()
    shop.sign_up("ann")
    users = iter(shop.users)
    assert list(users) == ["ann"]
    assert list(users) == ["ann"]


def test_generator_as_the_reading() -> None:
    """REFUSED: the reading of `all()` is a generator; it is named by its source. Says "line 36
    reads `numbers('1 2 3')`, a generator that one reading empties" """
    assert all(n > 0 for n in numbers("1 2 3"))


def test_generator_by_name() -> None:
    """REFUSED: Says "reads nums, a generator" """
    nums = numbers("1 2 3")
    assert all(n > 0 for n in nums)


def test_map_object() -> None:
    """REFUSED: Says "reads values, a map that one reading empties" """
    values = map(parse, ["1", "2"])
    assert list(values) == [1, 2]


def test_open_file() -> None:
    """REFUSED: a file is an iterator too. Says "reads f, a TextIOWrapper that one reading
    empties" """
    path = Path(tempfile.mkdtemp()) / "a.txt"
    path.write_text("x")
    f = path.open()
    assert f.read() == "x"


def test_a_list_in_the_act() -> None:
    """PASS: the act makes a list; each check reads it whole."""
    nums = list(numbers("1 2 3"))
    assert all(n > 0 for n in nums)
    assert nums == [1, 2, 3]


def test_made_inside_the_check() -> None:
    """PASS: a known gap. The check makes the generator itself, and no name holds it."""
    _ = parse("1")
    assert list(numbers("1 2")) == [1, 2]
