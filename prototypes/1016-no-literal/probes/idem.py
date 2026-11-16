def test_idem():
    xs = [3, 1, 2]
    assert sorted(sorted(xs)) == sorted(xs)
