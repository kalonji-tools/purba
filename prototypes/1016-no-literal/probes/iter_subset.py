def test_x():
    it = iter([1, 2])
    assert set(it) <= {1, 2, 3}
