"""Suppressions that purba honours."""
# purba: suppress-module[not-one-comparison] a subset reads best as one call
import hashlib
import json


# purba: suppress[computed-expected] the payload is its own expected value
def test_round_trip():
    obj = {"a": 1}
    assert json.loads(json.dumps(obj)) == obj


# purba: suppress[computed-expected] the payload is its own expected value
def test_round_trip_tuple():
    obj = {"a": (1, 2)}
    assert json.loads(json.dumps(obj)) == obj


# purba: suppress[computed-expected] both sides are digests of one payload
# purba: suppress[assert-before-expectation] the act checks its own input
def test_digest():
    data = b"abc"
    assert data == b"abc"
    digest = hashlib.sha256(data).hexdigest()
    assert digest == hashlib.sha256(b"abc").hexdigest()


def test_subset():
    got = {1, 2}
    assert got.issubset({1, 2, 3})


def test_fixture(tmp_path):
    got = len("ab")
    assert got == len("xy")
