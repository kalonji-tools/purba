def test_one():
    rows, want = [1, 1], 1
    assert all(row == want for row in rows)
