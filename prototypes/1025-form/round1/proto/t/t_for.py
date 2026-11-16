def test_positive():
    xs = [1, 2, -3, 0, 5]
    for x in xs:
        assert x > 0

def test_types():
    chunks = ["a", b"b", "c"]
    for c in chunks:
        assert isinstance(c, str)
