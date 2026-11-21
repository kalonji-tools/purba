import contextlib
import threading

seen = []


def check():
    if not seen:
        assert seen == []
    return True


def check_raises():
    if not seen:
        assert seen == []
        return True
    raise ValueError("raised in order")


def boom():
    if seen:
        raise ValueError("raised in order")
    return True


def test_raised():
    threading.Thread(target=threading.Event().wait, daemon=True).start()
    assert seen.append(1) is None
    assert check() is True
    assert boom() is True


def test_unreached():
    threading.Thread(target=threading.Event().wait, daemon=True).start()
    assert seen.append(1) is None
    assert boom() is True
    assert check() is True


def test_raises_in_order():
    threading.Thread(target=threading.Event().wait, daemon=True).start()
    assert seen.append(1) is None
    assert check_raises() is True


def test_swallowed():
    threading.Thread(target=threading.Event().wait, daemon=True).start()
    with contextlib.suppress(ValueError):
        raise ValueError("v")
        seen.append(2)
    assert seen.append(1) is None
    assert check() is True


def test_differs():
    threading.Thread(target=threading.Event().wait, daemon=True).start()
    assert seen.append(1) is None
    assert seen == [1]
    assert check() is True


# purba: suppress[swallowed-raise] the act once swallowed a raise
def test_suppressed():
    threading.Thread(target=threading.Event().wait, daemon=True).start()
    with contextlib.suppress(ValueError):
        raise ValueError("v")
        seen.append(2)
    assert seen.append(1) is None
    assert check() is True
