# purba: suppress-module[private-name] one test reads os._exit
import os


def test_reads():
    value = os._exit is not None
    assert value == True


def test_plain():
    value = 1
    assert value == 1


# purba: suppress[computed-expected] purba cannot call the test
async def test_async():
    value = 1
    assert value == 1
