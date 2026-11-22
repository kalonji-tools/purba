import contextlib


def test_try_in_with():
    seen = []
    with contextlib.suppress(ValueError):
        int("x")
        try:
            seen.append(1)
        finally:
            seen.append(2)
    assert seen == []


def test_with_in_try():
    seen = []
    try:
        with contextlib.suppress(ValueError):
            int("x")
            seen.append(1)
    finally:
        seen.append(2)
    assert seen == [2]


def test_no_try():
    seen = []
    with contextlib.suppress(ValueError):
        int("x")
        seen.append(1)
    assert seen == []
