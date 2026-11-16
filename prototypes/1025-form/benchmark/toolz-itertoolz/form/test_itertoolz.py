"""A port of 12 tests of toolz's `toolz/tests/test_itertoolz.py` into the form.

Each original test keeps its name and its value checks. Each expected raise of the original becomes a
case of a test of its own, because `with raises(…)` ends the act.
"""

from __future__ import annotations

import itertools
from operator import add

from cases import case, raises, use
from toolz.itertoolz import (
    accumulate,
    count,
    diff,
    isdistinct,
    iterate,
    nth,
    partition_all,
    peek,
    peekn,
    sliding_window,
    tail,
    take,
    topk,
)


def inc(x):
    return x + 1


def double(x):
    return 2 * x


def test_isdistinct() -> None:
    assert isdistinct([1, 2, 3]) is True
    assert isdistinct([1, 2, 1]) is False

    assert isdistinct("Hello") is False
    assert isdistinct("World") is True

    assert isdistinct(iter([1, 2, 3])) is True
    assert isdistinct(iter([1, 2, 1])) is False


def test_nth() -> None:
    assert nth(2, "ABCDE") == "C"
    assert nth(2, iter("ABCDE")) == "C"
    assert nth(1, (3, 2, 1)) == 2
    assert nth(0, {"foo": "bar"}) == "foo"
    assert nth(-2, "ABCDE") == "D"


def test_nth_raises(n: int, seq: object, error: type[Exception], message: str) -> None:
    with raises(error) as caught:
        nth(n, seq)
    assert str(caught.value) == message


nth_past_the_end = case(test_nth_raises, n=10, seq={10: "foo"}, error=StopIteration, message="")
nth_negative_of_iterator = case(
    test_nth_raises,
    n=-2,
    seq=iter("ABCDE"),
    error=ValueError,
    message="Indices for islice() must be None or an integer: 0 <= x <= sys.maxsize.",
)


def test_tail() -> None:
    assert list(tail(3, "ABCDE")) == ["C", "D", "E"]
    assert list(tail(3, iter("ABCDE"))) == ["C", "D", "E"]
    assert list(tail(2, (3, 2, 1))) == [2, 1]


def test_iterate() -> None:
    assert list(itertools.islice(iterate(inc, 0), 0, 5)) == [0, 1, 2, 3, 4]
    assert list(take(4, iterate(double, 1))) == [1, 2, 4, 8]


def test_accumulate_works_on_consumable_iterables() -> None:
    assert list(accumulate(add, iter((1, 2, 3)))) == [1, 3, 6]


def test_sliding_window_of_short_iterator() -> None:
    assert list(sliding_window(3, [1, 2])) == []
    assert list(sliding_window(7, [1, 2])) == []


class NoCompare:
    """Regression test: https://github.com/pytoolz/toolz/issues/387"""

    def __eq__(self, other):
        if self.__class__ == other.__class__:
            return True
        raise ValueError()


def test_partition_all(obj: NoCompare = use(NoCompare)) -> None:
    assert list(partition_all(2, [1, 2, 3, 4])) == [(1, 2), (3, 4)]
    assert list(partition_all(3, range(5))) == [(0, 1, 2), (3, 4)]
    assert list(partition_all(2, [])) == []

    # Regression test: https://github.com/pytoolz/toolz/issues/387
    assert list(partition_all(4, [obj] * 7)) == [(obj, obj, obj, obj), (obj, obj, obj)]
    assert list(partition_all(4, iter([obj] * 7))) == [(obj, obj, obj, obj), (obj, obj, obj)]


def test_partition_all_of_invalid_length(off_by: int) -> None:
    # Test invalid __len__: https://github.com/pytoolz/toolz/issues/602
    class ListWithBadLength(list):
        def __init__(self, contents, off_by=1):
            self.off_by = off_by
            super().__init__(contents)

        def __len__(self):
            return super().__len__() + self.off_by

    bad_list = ListWithBadLength([1, 2], off_by=off_by)
    with raises(LookupError) as caught:
        list(partition_all(5, bad_list))
    assert str(caught.value) == "The sequence passed to `parition_all` has invalid length"


too_long_list = case(test_partition_all_of_invalid_length, off_by=+1)
too_short_list = case(test_partition_all_of_invalid_length, off_by=-1)


def test_count() -> None:
    assert count((1, 2, 3)) == 3
    assert count([]) == 0
    assert count(iter((1, 2, 3, 4))) == 4

    assert count("hello") == 5
    assert count(iter("hello")) == 5


def test_diff() -> None:
    data1 = [{"cost": 1, "currency": "dollar"}, {"cost": 2, "currency": "dollar"}]

    data2 = [{"cost": 100, "currency": "yen"}, {"cost": 300, "currency": "yen"}]

    conversions = {"dollar": 1, "yen": 0.01}

    def indollars(item):
        return conversions[item["currency"]] * item["cost"]

    assert list(diff([1, 2], (1, 2), iter([1, 2]))) == []
    assert list(diff([1, 2, 3], (1, 10, 3), iter([1, 2, 10]))) == [(2, 10, 2), (3, 3, 10)]
    assert list(diff([1, 2], [10])) == [(1, 10)]
    assert list(diff([1, 2], [10], default=None)) == [(1, 10), (2, None)]
    # non-variadic usage
    assert list(diff([(1, 2), (1, 3)])) == [(2, 3)]

    assert list(diff(data1, data2, key=indollars)) == [
        ({"cost": 2, "currency": "dollar"}, {"cost": 300, "currency": "yen"})
    ]


def test_diff_raises(seqs: tuple[object, ...], message: str) -> None:
    with raises(TypeError) as caught:
        list(diff(*seqs))
    assert str(caught.value) == message


diff_of_no_sequence = case(
    test_diff_raises, seqs=(), message="Too few sequences given (min 2 required)"
)
diff_of_one_sequence = case(
    test_diff_raises, seqs=([1, 2],), message="'int' object is not iterable"
)
diff_of_a_sequence_and_a_number = case(
    test_diff_raises, seqs=([1, 2], 3), message="'int' object is not iterable"
)
# non-variadic usage
diff_of_an_empty_list = case(
    test_diff_raises, seqs=([],), message="Too few sequences given (min 2 required)"
)
diff_of_a_list_of_one_empty_sequence = case(
    test_diff_raises, seqs=([[]],), message="Too few sequences given (min 2 required)"
)
diff_of_a_list_of_one_sequence = case(
    test_diff_raises, seqs=([[1, 2]],), message="Too few sequences given (min 2 required)"
)
diff_of_a_list_of_a_sequence_and_a_number = case(
    test_diff_raises, seqs=([[1, 2], 3],), message="'int' object is not iterable"
)


def test_topk() -> None:
    assert topk(2, [4, 1, 5, 2]) == (5, 4)
    assert topk(2, [4, 1, 5, 2], key=lambda x: -x) == (1, 2)
    assert topk(2, iter([5, 1, 4, 2]), key=lambda x: -x) == (1, 2)

    assert topk(
        2, [{"a": 1, "b": 10}, {"a": 2, "b": 9}, {"a": 10, "b": 1}, {"a": 9, "b": 2}], key="a"
    ) == ({"a": 10, "b": 1}, {"a": 9, "b": 2})

    assert topk(
        2, [{"a": 1, "b": 10}, {"a": 2, "b": 9}, {"a": 10, "b": 1}, {"a": 9, "b": 2}], key="b"
    ) == ({"a": 1, "b": 10}, {"a": 2, "b": 9})
    assert topk(2, [(0, 4), (1, 3), (2, 2), (3, 1), (4, 0)], 0) == ((4, 0), (3, 1))


def test_peek() -> None:
    alist = ["Alice", "Bob", "Carol"]
    element, blist = peek(alist)
    whole = list(blist)
    assert element == "Alice"
    assert whole == ["Alice", "Bob", "Carol"]


def test_peek_of_empty() -> None:
    with raises(StopIteration) as caught:
        peek([])
    assert str(caught.value) == ""


def test_peekn() -> None:
    alist = ("Alice", "Bob", "Carol")
    elements, blist = peekn(2, alist)
    whole = tuple(blist)

    all_elements, blist_after_all = peekn(len(alist) * 4, alist)
    whole_after_all = tuple(blist_after_all)
    assert elements == ("Alice", "Bob")
    assert whole == ("Alice", "Bob", "Carol")
    assert all_elements == ("Alice", "Bob", "Carol")
    assert whole_after_all == ("Alice", "Bob", "Carol")
