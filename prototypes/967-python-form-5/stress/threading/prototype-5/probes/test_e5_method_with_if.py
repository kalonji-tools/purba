"""Experiment: a module-level class whose `__init__` and `write` each hold an `if`; the test builds it and calls `write`.

FORM.md: the `__init__` of a class the test builds, and each method the test calls, are part of the act, and have one path.
Control: the same class defined inside the test, which the runner should refuse.
"""

from cases import case


class NonSafeSink:
    def __init__(self, stopped: bool = False) -> None:
        self.written = ""
        if stopped:
            self.written = "stopped"
        self.stopped = stopped

    def write(self, message: str) -> None:
        if self.stopped:
            raise RuntimeError("Can't write on stopped sink")
        self.written += message


def test_write():
    sink = NonSafeSink()
    sink.write("abc")
    assert sink.written == "abc"


write = case(test_write)


def test_write_inner_class():
    class InnerSink:
        def __init__(self, stopped: bool = False) -> None:
            self.written = ""
            if stopped:
                self.written = "stopped"
            self.stopped = stopped

        def write(self, message: str) -> None:
            if self.stopped:
                raise RuntimeError("Can't write on stopped sink")
            self.written += message

    sink = InnerSink()
    sink.write("abc")
    assert sink.written == "abc"


write_inner_class = case(test_write_inner_class)
