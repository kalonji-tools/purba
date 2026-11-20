def test_one():
    rows = [[1], [1]]
    assert all(cell == 1 for row in rows for cell in row)
