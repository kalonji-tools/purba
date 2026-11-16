from lazy import gen


def test_all() -> None:
    xs = [1, 2, -3, 0, 5]
    assert all(x > 0 for x in xs)


def test_twice() -> None:
    it = iter([1, 2])
    assert list(it) == [1, 2]
    assert list(it) == [1, 2]


def test_once() -> None:
    it = iter([1, 2])
    assert list(it) == [1, 2]


def test_lazy_in_act() -> None:
    items = list(gen())
    assert items == [1, 2]


def test_lazy_in_check() -> None:
    assert list(gen()) == [1, 2]
