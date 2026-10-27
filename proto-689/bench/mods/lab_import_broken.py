import os

with open(os.environ["LAB_COUNT"], "a") as count:  # an effect outside the process
    count.write("x")
import a_module_that_does_not_exist  # noqa: E402,F401


def test():
    assert True
