def test_filtered():             # pytest 4.6.0's #5371: generator with `if`
    xs = [1, -2, 3]
    assert all(x > 0 for x in xs if x != -2)
