import json


# purba: suppress[computed-expected] the payload is its own expected value
def test_refused():
    obj = {"a": 1}
    assert json.loads(json.dumps(obj)) == obj
    assert {1}.issubset({1, 2, 3})
