"""Probe of two variants of the iterator rule (#1025)."""
import io

from toolz.itertoolz import sliding_window


def test_min_one_line() -> None:
    assert all(len(w) == 2 for w in sliding_window(2, iter([1, 2, 3, 4])))


def test_min_breaks() -> None:
    assert all(len(w) == 3 for w in sliding_window(2, iter([1, 2, 3, 4])))


def test_bound_iterator_still_refused() -> None:
    windows = sliding_window(2, iter([1, 2, 3, 4]))
    assert all(len(w) == 2 for w in windows)


def test_stringio_getvalue() -> None:
    buf = io.StringIO()
    buf.write("hello")
    assert buf.getvalue() == "hello"
    assert not buf.closed


def test_stringio_read_twice() -> None:
    buf = io.StringIO("x")
    assert buf.read() == "x"
    assert buf.read() == "x"


def test_plain_iterator_still_refused() -> None:
    it = iter([1, 2])
    assert list(it) == [1, 2]
