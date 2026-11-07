import os

with open(os.environ["LAB_COUNT"], "a") as count:  # an effect outside the process
    count.write("x")


def test():
    assert 1 + 1 == 2
