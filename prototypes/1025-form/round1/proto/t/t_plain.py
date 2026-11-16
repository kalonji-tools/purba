def gen():
    yield 1
    yield 2
    raise ValueError("boom")

def test_twice():
    it = iter([1, 2])
    assert list(it) == [1, 2]
    assert list(it) == [1, 2]

def test_once():
    it = iter([1, 2])
    assert list(it) == [1, 2]

def test_next_each():
    it = iter([1, 2])
    assert next(it) == 1

def test_call_in_check():
    assert list(gen()) == [1, 2]

def test_lazy_in_act():
    items = list(gen())
    assert items == [1, 2]

def test_list_in_act():
    items = list(iter([1, 2]))
    assert items == [1, 2]
