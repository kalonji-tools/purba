from collections import Counter
def test_x():
    it = iter(["a", "b", "a"])
    assert Counter(it) == {"a": 2, "b": 1}
