def test_first():
    assert 1 == 1

def test_segfaults():
    import ctypes
    ctypes.string_at(0)
