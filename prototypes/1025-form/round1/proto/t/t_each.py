from iterkit import each

def test_positive():
    xs = [1, 2, -3, 0, 5]
    assert each(xs) > 0

def test_is():                   # `is` cannot be overloaded
    ts = [ZeroDivisionError, ZeroDivisionError]
    assert each(ts) is ZeroDivisionError

def test_types():                # isinstance cannot be overloaded
    chunks = ["a", "c"]
    assert isinstance(each(chunks), str)

def test_in_holds_wrongly():     # list.__contains__ asks `1 == each(xs)`
    xs = [1, 9]
    assert each(xs) in [1, 2]

def test_literal_left():         # reflected: 0 < each(xs) calls each.__gt__
    xs = [1, -3]
    assert 0 < each(xs)
