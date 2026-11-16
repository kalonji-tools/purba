def test_list():
    xs = [1, 2, -3]
    assert [x > 0 for x in xs] == [True, True, True]
