import json


def test_private():
    got = json._default_encoder.indent
    assert got is None
