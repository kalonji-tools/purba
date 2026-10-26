# -*- coding: latin-1 -*-
def test_cookie():
    assert "é" == "\xc3\xa9"

if __name__ == "__main__":
    test_cookie(); print("CPython: test_cookie holds")
