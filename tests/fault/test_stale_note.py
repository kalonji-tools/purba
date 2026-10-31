# The first test catches the panic of purba's assert hook, and the second
# raises purba's own PanicException, whose panic no hook sees.


def panic_exception():
    pending = [BaseException]
    while pending:
        kind = pending.pop()
        if kind.__module__ == "pyo3_runtime" and kind.__name__ == "PanicException":
            return kind
        pending.extend(kind.__subclasses__())
    raise LookupError("no PanicException")


def test_catches():
    try:
        assert True
    except BaseException:
        pass


def test_raises():
    raise panic_exception()("raised by the second test")
