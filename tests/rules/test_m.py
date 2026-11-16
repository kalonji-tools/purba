import contextlib
import threading

opened = ["a handle"]


@contextlib.contextmanager
def no_leaks():
    yield
    assert opened == [], "a handle leaked"


def test_path():
    for name in []:
        print(name)
    assert 1 == 1


def test_branch():
    total = 1
    if total:
        total = 2
    assert total == 2


def test_assert():
    total = 1
    assert total == 1
    total = 2
    assert total == 2


# purba: suppress[patched-name] execution must still refuse a hook that the test replaced
def test_hook():
    threading.excepthook = print
    assert 1 == 1


def test_swallow():
    with contextlib.suppress(KeyError):
        {}["k"]
        print("never")
    assert 1 == 1


def test_cleanup():
    with no_leaks():
        total = 1
    assert total == 1
