import lending


def test_uses_monkeypatch(monkeypatch):
    monkeypatch.setattr(lending, "Library", object)
    assert lending.Library == object
