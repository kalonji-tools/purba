import itertools
from itertools import starmap
from toolz.utils import raises
from functools import partial
from random import Random
from pickle import dumps, loads
from toolz.itertoolz import (remove, groupby, merge_sorted,
                             concat, concatv, interleave, unique,
                             isiterable, getter,
                             mapcat, isdistinct, first, second,
                             nth, take, tail, drop, interpose, get,
                             rest, last, cons, frequencies,
                             reduceby, iterate, accumulate,
                             sliding_window, count, partition,
                             partition_all, take_nth, pluck, join,
                             diff, topk, peek, peekn, random_sample)
from operator import add, mul


# is comparison will fail between this and no_default
no_default2 = loads(dumps('__no__default__'))


def _identity(x):
    return x


def _iseven(x):
    return x % 2 == 0


def _isodd(x):
    return x % 2 == 1


def _inc(x):
    return x + 1


def _double(x):
    return 2 * x


def test_isdistinct():
    assert isdistinct([1, 2, 3]) is True, "a list with no repeats is distinct"
    assert isdistinct([1, 2, 1]) is False, "a list with a repeat is not distinct"

    assert isdistinct("Hello") is False, "'Hello' repeats 'l'"
    assert isdistinct("World") is True, "'World' has no repeated letter"

    assert isdistinct(iter([1, 2, 3])) is True, "an iterator with no repeats is distinct"
    assert isdistinct(iter([1, 2, 1])) is False, "an iterator with a repeat is not distinct"


def test_nth():
    assert nth(2, 'ABCDE') == 'C', "nth indexes a string"
    assert nth(2, iter('ABCDE')) == 'C', "nth advances an iterator"
    assert nth(1, (3, 2, 1)) == 2, "nth indexes a tuple"
    assert nth(0, {'foo': 'bar'}) == 'foo', "nth on a dict iterates its keys"
    assert raises(StopIteration, lambda: nth(10, {10: 'foo'})), \
        "nth past the end of a dict raises StopIteration"
    assert nth(-2, 'ABCDE') == 'D', "a negative index works on a sequence"
    assert raises(ValueError, lambda: nth(-2, iter('ABCDE'))), \
        "a negative index on an iterator raises ValueError"


def test_tail():
    assert list(tail(3, 'ABCDE')) == list('CDE'), "tail of a string"
    assert list(tail(3, iter('ABCDE'))) == list('CDE'), "tail of an iterator"
    assert list(tail(2, (3, 2, 1))) == list((2, 1)), "tail of a tuple"


def test_iterate():
    assert list(itertools.islice(iterate(_inc, 0), 0, 5)) == [0, 1, 2, 3, 4], \
        "iterate applies inc repeatedly from 0"
    assert list(take(4, iterate(_double, 1))) == [1, 2, 4, 8], \
        "iterate applies double repeatedly from 1"


def test_accumulate_works_on_consumable_iterables():
    assert list(accumulate(add, iter((1, 2, 3)))) == [1, 3, 6], \
        "accumulate consumes a one-shot iterator"


def test_sliding_window_of_short_iterator():
    assert list(sliding_window(3, [1, 2])) == [], \
        "a window longer than the input yields nothing"
    assert list(sliding_window(7, [1, 2])) == [], \
        "a much longer window yields nothing"


def test_partition_all():
    assert list(partition_all(2, [1, 2, 3, 4])) == [(1, 2), (3, 4)], \
        "an even split has no short tail"
    assert list(partition_all(3, range(5))) == [(0, 1, 2), (3, 4)], \
        "an uneven split keeps the short tail"
    assert list(partition_all(2, [])) == [], "an empty input yields nothing"

    # Regression test: https://github.com/pytoolz/toolz/issues/387
    class NoCompare:
        def __eq__(self, other):
            if self.__class__ == other.__class__:
                return True
            raise ValueError()
    obj = NoCompare()
    result = [(obj, obj, obj, obj), (obj, obj, obj)]
    assert list(partition_all(4, [obj]*7)) == result, \
        "partition_all never compares items against a fill value (list)"
    assert list(partition_all(4, iter([obj]*7))) == result, \
        "partition_all never compares items against a fill value (iterator)"

    # Test invalid __len__: https://github.com/pytoolz/toolz/issues/602
    class ListWithBadLength(list):
        def __init__(self, contents, off_by=1):
            self.off_by = off_by
            super().__init__(contents)

        def __len__(self):
            return super().__len__() + self.off_by

    too_long_list = ListWithBadLength([1, 2], off_by=+1)
    assert raises(LookupError, lambda: list(partition_all(5, too_long_list))), \
        "a __len__ that is too long raises LookupError"
    too_short_list = ListWithBadLength([1, 2], off_by=-1)
    assert raises(LookupError, lambda: list(partition_all(5, too_short_list))), \
        "a __len__ that is too short raises LookupError"


def test_count():
    assert count((1, 2, 3)) == 3, "count of a tuple"
    assert count([]) == 0, "count of an empty list"
    assert count(iter((1, 2, 3, 4))) == 4, "count of an iterator"

    assert count('hello') == 5, "count of a string"
    assert count(iter('hello')) == 5, "count of a string iterator"


def test_diff():
    assert raises(TypeError, lambda: list(diff())), \
        "diff with no sequences raises TypeError"
    assert raises(TypeError, lambda: list(diff([1, 2]))), \
        "diff with one sequence raises TypeError"
    assert raises(TypeError, lambda: list(diff([1, 2], 3))), \
        "diff with a non-iterable raises TypeError"
    assert list(diff([1, 2], (1, 2), iter([1, 2]))) == [], \
        "equal sequences have no differences"
    assert list(diff([1, 2, 3], (1, 10, 3), iter([1, 2, 10]))) == [
        (2, 10, 2), (3, 3, 10)], "diff yields each position that differs"
    assert list(diff([1, 2], [10])) == [(1, 10)], \
        "diff stops at the shortest sequence"
    assert list(diff([1, 2], [10], default=None)) == [(1, 10), (2, None)], \
        "diff pads with default when one is given"
    # non-variadic usage
    assert raises(TypeError, lambda: list(diff([]))), \
        "non-variadic diff of no sequences raises TypeError"
    assert raises(TypeError, lambda: list(diff([[]]))), \
        "non-variadic diff of one sequence raises TypeError"
    assert raises(TypeError, lambda: list(diff([[1, 2]]))), \
        "non-variadic diff of one sequence raises TypeError"
    assert raises(TypeError, lambda: list(diff([[1, 2], 3]))), \
        "non-variadic diff with a non-iterable raises TypeError"
    assert list(diff([(1, 2), (1, 3)])) == [(2, 3)], \
        "non-variadic diff of two sequences"

    data1 = [{'cost': 1, 'currency': 'dollar'},
             {'cost': 2, 'currency': 'dollar'}]

    data2 = [{'cost': 100, 'currency': 'yen'},
             {'cost': 300, 'currency': 'yen'}]

    conversions = {'dollar': 1, 'yen': 0.01}

    def indollars(item):
        return conversions[item['currency']] * item['cost']

    assert list(diff(data1, data2, key=indollars)) == [
        ({'cost': 2, 'currency': 'dollar'}, {'cost': 300, 'currency': 'yen'})], \
        "diff with a key compares the converted values"


def test_topk():
    assert topk(2, [4, 1, 5, 2]) == (5, 4), "topk returns the largest first"
    assert topk(2, [4, 1, 5, 2], key=lambda x: -x) == (1, 2), \
        "topk with a key function"
    assert topk(2, iter([5, 1, 4, 2]), key=lambda x: -x) == (1, 2), \
        "topk with a key function over an iterator"

    assert topk(2, [{'a': 1, 'b': 10}, {'a': 2, 'b': 9},
                    {'a': 10, 'b': 1}, {'a': 9, 'b': 2}], key='a') == \
        ({'a': 10, 'b': 1}, {'a': 9, 'b': 2}), "topk with a dict key 'a'"

    assert topk(2, [{'a': 1, 'b': 10}, {'a': 2, 'b': 9},
                    {'a': 10, 'b': 1}, {'a': 9, 'b': 2}], key='b') == \
        ({'a': 1, 'b': 10}, {'a': 2, 'b': 9}), "topk with a dict key 'b'"
    assert topk(2, [(0, 4), (1, 3), (2, 2), (3, 1), (4, 0)], 0) == \
        ((4, 0), (3, 1)), "topk with a positional index key"


def test_peek():
    alist = ["Alice", "Bob", "Carol"]
    element, blist = peek(alist)
    assert element == alist[0], "peek returns the first element"
    assert list(blist) == alist, "peek's iterator still yields every element"

    assert raises(StopIteration, lambda: peek([])), \
        "peek of an empty sequence raises StopIteration"


def test_peekn():
    alist = ("Alice", "Bob", "Carol")
    elements, blist = peekn(2, alist)
    assert elements == alist[:2], "peekn returns the first n elements"
    assert tuple(blist) == alist, "peekn's iterator still yields every element"

    elements, blist = peekn(len(alist) * 4, alist)
    assert elements == alist, "peekn past the end returns every element"
    assert tuple(blist) == alist, \
        "peekn past the end still yields every element"
