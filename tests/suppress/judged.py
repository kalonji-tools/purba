# purba: suppress-module[computed-expected] one test computes its expected value
import os


def test_reads():
    value = os.sep
    assert value == os.sep


def test_plain():
    value = 1
    assert value == 1


# purba: suppress[computed-expected] purba cannot call the test
async def test_async():
    value = 1
    assert value == 1
