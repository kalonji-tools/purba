from lib_under import own_mode

def test_own():
    got = own_mode()
    assert got == "optimized"
