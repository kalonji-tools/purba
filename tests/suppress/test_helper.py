"""A suppression above a helper."""
import hashlib


# purba: suppress[computed-expected] digests match
def check_digest(data):
    return hashlib.sha256(data).hexdigest()


def test_digest():
    got = check_digest(b"abc")
    assert got == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
