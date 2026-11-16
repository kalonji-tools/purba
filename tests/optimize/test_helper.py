def check(x):
    assert x == 2
    return x

def test_helper():
    got = check(1 + 2)
    assert got == 3
