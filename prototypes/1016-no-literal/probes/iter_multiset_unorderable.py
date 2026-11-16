from collections import Counter
def test_x():
    it = iter([{"a": 1}, {"b": 2}])
    assert Counter(map(repr, it)) == Counter(["{\"b\": 2}", "{\"a\": 1}"])
