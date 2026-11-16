def test_x():
    xs = [1, 2, -3]
    assert [(i, x) for i, x in enumerate(xs) if x <= 0] == []
