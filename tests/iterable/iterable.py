def gen():
    yield 1
    raise ValueError("boom")


def parse(text):
    return int(text)


def test_each():
    parts = ["ab", "cd", "e"]
    assert all(len(c) == 2 for c in parts)


def test_some():
    paths = ["/usr/share", "/etc"]
    assert any(p == "/opt" for p in paths)


def test_filtered():
    xs = [1, -2]
    assert all(x > 0 for x in xs if x)


def test_empty():
    xs = []
    assert all(x > 0 for x in xs)


def test_iterator():
    it = iter([1, 2])
    assert list(it) == [1, 2]


def test_made_by_the_check():
    rows = [1, 2]
    assert list(iter(rows)) == [1, 2]


def test_generator_raises():
    items = list(gen())
    assert items == [1]


def test_parse_raises():
    n = parse("x")
    assert n == 1
