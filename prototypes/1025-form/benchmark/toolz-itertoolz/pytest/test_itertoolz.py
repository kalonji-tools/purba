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


def identity(x):
    return x


def iseven(x):
    return x % 2 == 0


def isodd(x):
    return x % 2 == 1


def inc(x):
    return x + 1


def double(x):
    return 2 * x


def test_isdistinct():
    assert isdistinct([1, 2, 3]) is True
    assert isdistinct([1, 2, 1]) is False

    assert isdistinct("Hello") is False
    assert isdistinct("World") is True

    assert isdistinct(iter([1, 2, 3])) is True
    assert isdistinct(iter([1, 2, 1])) is False


def test_nth():
    assert nth(2, 'ABCDE') == 'C'
    assert nth(2, iter('ABCDE')) == 'C'
    assert nth(1, (3, 2, 1)) == 2
    assert nth(0, {'foo': 'bar'}) == 'foo'
    assert raises(StopIteration, lambda: nth(10, {10: 'foo'}))
    assert nth(-2, 'ABCDE') == 'D'
    assert raises(ValueError, lambda: nth(-2, iter('ABCDE')))


def test_tail():
    assert list(tail(3, 'ABCDE')) == list('CDE')
    assert list(tail(3, iter('ABCDE'))) == list('CDE')
    assert list(tail(2, (3, 2, 1))) == list((2, 1))


def test_iterate():
    assert list(itertools.islice(iterate(inc, 0), 0, 5)) == [0, 1, 2, 3, 4]
    assert list(take(4, iterate(double, 1))) == [1, 2, 4, 8]


def test_accumulate_works_on_consumable_iterables():
    assert list(accumulate(add, iter((1, 2, 3)))) == [1, 3, 6]


def test_sliding_window_of_short_iterator():
    assert list(sliding_window(3, [1, 2])) == []
    assert list(sliding_window(7, [1, 2])) == []


def test_partition_all():
    assert list(partition_all(2, [1, 2, 3, 4])) == [(1, 2), (3, 4)]
    assert list(partition_all(3, range(5))) == [(0, 1, 2), (3, 4)]
    assert list(partition_all(2, [])) == []

    # Regression test: https://github.com/pytoolz/toolz/issues/387
    class NoCompare:
        def __eq__(self, other):
            if self.__class__ == other.__class__:
                return True
            raise ValueError()
    obj = NoCompare()
    result = [(obj, obj, obj, obj), (obj, obj, obj)]
    assert list(partition_all(4, [obj]*7)) == result
    assert list(partition_all(4, iter([obj]*7))) == result

    # Test invalid __len__: https://github.com/pytoolz/toolz/issues/602
    class ListWithBadLength(list):
        def __init__(self, contents, off_by=1):
            self.off_by = off_by
            super().__init__(contents)

        def __len__(self):
            return super().__len__() + self.off_by

    too_long_list = ListWithBadLength([1, 2], off_by=+1)
    assert raises(LookupError, lambda: list(partition_all(5, too_long_list)))
    too_short_list = ListWithBadLength([1, 2], off_by=-1)
    assert raises(LookupError, lambda: list(partition_all(5, too_short_list)))


def test_count():
    assert count((1, 2, 3)) == 3
    assert count([]) == 0
    assert count(iter((1, 2, 3, 4))) == 4

    assert count('hello') == 5
    assert count(iter('hello')) == 5


def test_diff():
    assert raises(TypeError, lambda: list(diff()))
    assert raises(TypeError, lambda: list(diff([1, 2])))
    assert raises(TypeError, lambda: list(diff([1, 2], 3)))
    assert list(diff([1, 2], (1, 2), iter([1, 2]))) == []
    assert list(diff([1, 2, 3], (1, 10, 3), iter([1, 2, 10]))) == [
        (2, 10, 2), (3, 3, 10)]
    assert list(diff([1, 2], [10])) == [(1, 10)]
    assert list(diff([1, 2], [10], default=None)) == [(1, 10), (2, None)]
    # non-variadic usage
    assert raises(TypeError, lambda: list(diff([])))
    assert raises(TypeError, lambda: list(diff([[]])))
    assert raises(TypeError, lambda: list(diff([[1, 2]])))
    assert raises(TypeError, lambda: list(diff([[1, 2], 3])))
    assert list(diff([(1, 2), (1, 3)])) == [(2, 3)]

    data1 = [{'cost': 1, 'currency': 'dollar'},
             {'cost': 2, 'currency': 'dollar'}]

    data2 = [{'cost': 100, 'currency': 'yen'},
             {'cost': 300, 'currency': 'yen'}]

    conversions = {'dollar': 1, 'yen': 0.01}

    def indollars(item):
        return conversions[item['currency']] * item['cost']

    list(diff(data1, data2, key=indollars)) == [
        ({'cost': 2, 'currency': 'dollar'}, {'cost': 300, 'currency': 'yen'})]


def test_topk():
    assert topk(2, [4, 1, 5, 2]) == (5, 4)
    assert topk(2, [4, 1, 5, 2], key=lambda x: -x) == (1, 2)
    assert topk(2, iter([5, 1, 4, 2]), key=lambda x: -x) == (1, 2)

    assert topk(2, [{'a': 1, 'b': 10}, {'a': 2, 'b': 9},
                    {'a': 10, 'b': 1}, {'a': 9, 'b': 2}], key='a') == \
        ({'a': 10, 'b': 1}, {'a': 9, 'b': 2})

    assert topk(2, [{'a': 1, 'b': 10}, {'a': 2, 'b': 9},
                    {'a': 10, 'b': 1}, {'a': 9, 'b': 2}], key='b') == \
        ({'a': 1, 'b': 10}, {'a': 2, 'b': 9})
    assert topk(2, [(0, 4), (1, 3), (2, 2), (3, 1), (4, 0)], 0) == \
        ((4, 0), (3, 1))


def test_peek():
    alist = ["Alice", "Bob", "Carol"]
    element, blist = peek(alist)
    assert element == alist[0]
    assert list(blist) == alist

    assert raises(StopIteration, lambda: peek([]))


def test_peekn():
    alist = ("Alice", "Bob", "Carol")
    elements, blist = peekn(2, alist)
    assert elements == alist[:2]
    assert tuple(blist) == alist

    elements, blist = peekn(len(alist) * 4, alist)
    assert elements == alist
    assert tuple(blist) == alist
