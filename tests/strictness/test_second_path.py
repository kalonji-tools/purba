def test_refused():
    total = 1
    if total:
        total = 2
    assert total == 2
