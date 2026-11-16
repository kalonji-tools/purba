def test_all():
    xs = [1, 2, -3]
    assert all(x > 0 for x in xs)
