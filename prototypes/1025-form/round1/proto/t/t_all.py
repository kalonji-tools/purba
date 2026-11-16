def test_positive():
    xs = [1, 2, -3, 0, 5]
    assert all(x > 0 for x in xs)

def test_types():
    chunks = ["a", b"b", "c"]
    assert all(isinstance(c, str) for c in chunks)

def test_is():
    ts = [ZeroDivisionError, ZeroDivisionError, ValueError]
    assert all(t is ZeroDivisionError for t in ts)

def test_holds():
    xs = [1, 2, 3]
    assert all(x > 0 for x in xs)

def test_iterator():
    it = iter([1, 2, 3])
    assert all(x > 0 for x in it)

def gen():
    yield 1
    yield 2
    raise ValueError("boom")

def test_lazy_in_check():
    assert all(x > 0 for x in gen())

def test_nested_comp():          # pytest 4.6.0's #5370: NameError
    xs = [[1, 2], [3, -4]]
    assert all(v > 0 for v in [y for row in xs for y in row])

def test_tuple_target():
    pairs = [(1, 1), (2, 3)]
    assert all(a == b for a, b in pairs)
