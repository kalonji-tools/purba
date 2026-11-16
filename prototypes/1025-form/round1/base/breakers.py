def test_x():
    xs = [1, 2, -3, 0]
    assert [x for x in xs if not x > 0] == []
