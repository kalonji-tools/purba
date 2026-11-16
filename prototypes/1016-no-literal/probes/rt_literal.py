import json
def test_rt():
    assert json.loads(json.dumps({"a": 1})) == {"a": 1}
