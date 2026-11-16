import warnings

def test_some_deprecation():
    caught = [warnings.WarningMessage("a", UserWarning, "f", 1), warnings.WarningMessage("b", RuntimeWarning, "f", 2)]
    assert any(w.category is DeprecationWarning for w in caught)

def test_prefix():
    tzpath = ["/usr/share/zoneinfo", "/etc/zoneinfo"]
    assert any(p.startswith("/opt") for p in tzpath)
