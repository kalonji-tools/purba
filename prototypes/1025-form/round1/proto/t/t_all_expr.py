def test_in_expression():        # pytest 4.6.0's #5372: all() inside an expression
    xs = [1, 2]
    assert all(x > 0 for x in xs) == True
