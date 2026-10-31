# PyO3 resumes the panic of a PanicException that Python raises, and no panic
# hook sees it.


def panic_exception():
    pending = [BaseException]
    while pending:
        kind = pending.pop()
        if kind.__module__ == "pyo3_runtime" and kind.__name__ == "PanicException":
            return kind
        pending.extend(kind.__subclasses__())
    raise LookupError("no PanicException")


def test_raises_it():
    raise panic_exception()("raised by the test")
