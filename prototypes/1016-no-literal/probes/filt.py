def test_eq():
    xs = [1, 2, -3]
    assert [x for x in xs if x <= 0] == []
