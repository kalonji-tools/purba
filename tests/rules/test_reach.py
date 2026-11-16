import contextlib

log = []
@contextlib.contextmanager
def opened(flag):
    if flag:
        log.append("open")
    yield


def check_user(user):
    assert user == "ann"


def make(user):
    if user:
        return user
    return "nobody"


def test_cm():
    with opened(True):
        total = 1
    assert total == 1


def test_reading():
    user = "ann"
    assert check_user(user) is None


def test_helper():
    user = make("ann")
    assert user == "ann"


def test_two_lines():
    user = make("ann")
    other = make("bob")
    assert other == "bob"


def test_stack():
    with contextlib.ExitStack() as stack:
        stack.enter_context(opened(True))
    assert log[-1] == "open"


@contextlib.contextmanager
def kept():
    log.append("in")
    yield
    log.append("out")


def test_kept():
    with kept():
        total = 1
    assert total == 1


def test_inside():
    with kept():
        assert log[-1] == "in"
