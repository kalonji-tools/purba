def test_x():
    xs = [1, -2, 3]
    assert [x for x in xs if x < 0] != []
