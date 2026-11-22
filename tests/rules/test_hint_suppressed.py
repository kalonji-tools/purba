import contextlib


# purba: suppress[second-path] the loop once read names
def test_suppressed():
    seen = []
    try:
        for name in []:
            seen.append(name)
    finally:
        seen.append(2)
    with contextlib.suppress(ValueError):
        int("x")
        seen.append(3)
    assert seen == [2]


def test_unsuppressed():
    seen = []
    try:
        for name in []:
            seen.append(name)
    finally:
        seen.append(2)
    with contextlib.suppress(ValueError):
        int("x")
        seen.append(3)
    assert seen == [2]


def test_try_ran():
    seen = []
    try:
        seen.append(1)
    finally:
        seen.append(2)
    with contextlib.suppress(ValueError):
        int("x")
        seen.append(3)
    assert seen == [1, 2]
