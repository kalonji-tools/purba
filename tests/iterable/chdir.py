import os


def parse(text):
    return int(text)


def test_chdir():
    os.chdir("/")
    parse("x")
