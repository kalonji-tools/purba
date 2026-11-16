def gen():
    yield 1
    yield 2
    raise ValueError("boom")
