# Q2: a process that ends without an outcome.
import ctypes, os, posix

def test_abort():
    os.abort()

def test_posix_exit_0():
    posix._exit(0)

def test_os_exit_3():
    os._exit(3)

def test_segfault():
    ctypes.string_at(0)

def test_still_runs():
    pass
