def test_a():
    got = 1
    assert got == 1

def test_b():
    got = 1
    assert got

def test_c():
    got = check(2)
    assert got == 2

def check(x):
    assert x == 2
    return x
