import os


def test_refused():
    os.environ["PURBA_PROBE"] = "1"
    assert os.environ["PURBA_PROBE"] == "1"
