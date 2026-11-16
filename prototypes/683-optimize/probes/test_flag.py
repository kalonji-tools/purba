from lib_under import mode

def test_flag():
    got = mode(__debug__)
    assert got == "optimized"
