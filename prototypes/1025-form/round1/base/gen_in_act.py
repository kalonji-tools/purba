def gen():
    yield 1
    yield 2
    raise ValueError("boom")
def test_x():
    items = list(gen())
    assert items == [1, 2]
