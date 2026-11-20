def test_one():
    rows = [1, 1]
    assert all((n := row) == 1 for row in rows)
