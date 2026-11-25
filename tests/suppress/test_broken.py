"""Suppressions that purba refuses."""
import json


# purba: suppress[returned-value] the caller reads it
def test_returns():
    got = len("ab")
    assert got == 2
    return got


# purba: suppress[no-expectation] it only logs
def test_logs():
    print("hello")


# purba: suppress[computed-expected]
def test_no_reason():
    obj = {"a": 1}
    assert json.loads(json.dumps(obj)) == obj


# purba: suppress[slow-test] it takes a second
def test_unknown():
    got = len("ab")
    assert got == 2


# purba: suppress computed-expected the payload is its own expected value
def test_unread():
    got = len("ab")
    assert got == 2
