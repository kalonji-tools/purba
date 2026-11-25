import os

SOURCE = "def boom():\n    raise ValueError('x')\n"


def test_parent():
    ns = {}
    exec(compile(SOURCE, "../helper.py", "exec"), ns)
    ns["boom"]()


def test_relative():
    ns = {}
    exec(compile(SOURCE, "helper.py", "exec"), ns)
    ns["boom"]()


def test_moved():
    os.chdir("/")
    ns = {}
    exec(compile(SOURCE, "helper.py", "exec"), ns)
    ns["boom"]()
