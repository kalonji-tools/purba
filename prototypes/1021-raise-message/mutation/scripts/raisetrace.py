"""pytest plugin: record every exception that reaches a pytest.raises / assertRaises context.

Env:
  RT_OUT   path of a JSONL file to append to
  RT_SRC   colon-separated absolute dirs that hold the project's own NON-test source
For each __exit__ with an exception it records the `with` frame (file, line), the
exception type and str(), and the raise site: the deepest traceback frame, if it lies in
RT_SRC and its current instruction is RAISE_VARARGS (an explicit `raise` statement).
It also records each test's phase outcomes.
"""

import dis
import json
import os
import sys
import unittest.case

import pytest
import _pytest.raises as _pr

_OUT = os.environ.get("RT_OUT")
_SRC = [os.path.realpath(p) + os.sep for p in os.environ.get("RT_SRC", "").split(":") if p]
_current = {"nodeid": None}
_fh = open(_OUT, "a", encoding="utf-8") if _OUT else None
_instr_cache = {}


def _emit(rec):
    if _fh:
        _fh.write(json.dumps(rec) + "\n")
        _fh.flush()


def _in_src(fn):
    try:
        fn = os.path.realpath(fn)
    except Exception:
        return False
    if not any(fn.startswith(s) for s in _SRC):
        return False
    low = fn.lower()
    rel = [s for s in _SRC if fn.startswith(s)][0]
    parts = fn[len(rel):].lower().split(os.sep)
    return not any("test" in p for p in parts) and low.endswith(".py")


def _opname(code, lasti):
    m = _instr_cache.get(code)
    if m is None:
        m = {i.offset: (i.opname, i.arg) for i in dis.get_instructions(code)}
        _instr_cache[code] = m
    return m.get(lasti, (None, None))


def _site(tb):
    last = None
    while tb is not None:
        last = tb
        tb = tb.tb_next
    if last is None:
        return {"kind": "no-traceback"}
    code = last.tb_frame.f_code
    fn = code.co_filename
    op, arg = _opname(code, last.tb_lasti)
    d = {"file": os.path.realpath(fn) if os.path.exists(fn) else fn, "line": last.tb_lineno, "op": op, "arg": arg}
    if not _in_src(fn):
        d["kind"] = "outside"
    elif op == "RAISE_VARARGS":
        d["kind"] = "own-raise"
    else:
        d["kind"] = "implicit"
    return d


def _qual(t):
    return f"{t.__module__}.{t.__qualname__}"


def _record(api, ctx, exc_type, exc_val, exc_tb, expected, outcome):
    f = sys._getframe(2)
    rec = {
        "ev": "raise",
        "api": api,
        "nodeid": _current["nodeid"],
        "with_file": os.path.realpath(f.f_code.co_filename),
        "with_line": f.f_lineno,
        "expected": [_qual(e) for e in expected],
        "exc_type": _qual(exc_type) if exc_type else None,
        "exc_str": None,
        "outcome": outcome,
    }
    if exc_val is not None:
        try:
            rec["exc_str"] = str(exc_val)
        except Exception as e:  # noqa
            rec["exc_str"] = f"<str failed: {e!r}>"
        rec["site"] = _site(exc_tb)
    _emit(rec)


_orig_pexit = _pr.RaisesExc.__exit__


def _pexit(self, exc_type, exc_val, exc_tb):
    __tracebackhide__ = True
    try:
        r = _orig_pexit(self, exc_type, exc_val, exc_tb)
        out = "caught" if r else "propagated"
    except BaseException:
        _record("pytest", self, exc_type, exc_val, exc_tb, self.expected_exceptions or (), "failed")
        raise
    _record("pytest", self, exc_type, exc_val, exc_tb, self.expected_exceptions or (), out)
    return r


_pr.RaisesExc.__exit__ = _pexit

_orig_uexit = unittest.case._AssertRaisesContext.__exit__


def _uexit(self, exc_type, exc_value, tb):
    exp = self.expected if isinstance(self.expected, tuple) else (self.expected,)
    try:
        r = _orig_uexit(self, exc_type, exc_value, tb)
        out = "caught" if r else "propagated"
    except BaseException:
        _record("unittest", self, exc_type, exc_value, tb, exp, "failed")
        raise
    _record("unittest", self, exc_type, exc_value, tb, exp, out)
    return r


unittest.case._AssertRaisesContext.__exit__ = _uexit


@pytest.hookimpl(wrapper=True)
def pytest_runtest_protocol(item, nextitem):
    _current["nodeid"] = item.nodeid
    try:
        return (yield)
    finally:
        _current["nodeid"] = None


def pytest_runtest_logreport(report):
    _emit({"ev": "report", "nodeid": report.nodeid, "when": report.when, "outcome": report.outcome})


def pytest_unconfigure(config):
    if _fh and not _fh.closed:
        _fh.close()
