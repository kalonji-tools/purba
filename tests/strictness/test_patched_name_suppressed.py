import os


# purba: suppress[patched-name] this case shows the departure
def test_suppressed():
    os.environ["PURBA_PROBE"] = "1"
    assert os.environ["PURBA_PROBE"] == "1"
