def gen():
    yield 1
    raise ValueError("boom")
def test_x():
    it = gen()
    assert list(it) == [1]
