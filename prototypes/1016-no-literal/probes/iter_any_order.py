def test_x():
    it = iter([3, 1, 2])
    assert sorted(it) == [1, 2, 3]
