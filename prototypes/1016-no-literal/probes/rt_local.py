import json
def test_rt():
    obj = {"a": 1}
    assert json.loads(json.dumps(obj)) == obj
