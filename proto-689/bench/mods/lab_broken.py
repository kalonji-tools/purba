import os

with open(os.environ["LAB_COUNT"], "a") as count:  # an effect outside the process
    count.write("x")
raise RuntimeError("the module fails to load")


def test():
    assert True
