"""PROTOTYPE (#689): run a test in a forked child, the process beaker.

FORK=all       fork every test
FORK=<file>    fork only the functions listed in <file> (one "<file>::<func>" a line);
               every other test runs in the parent, bundled in one interpreter
The child runs setup, call and teardown, sends its reports up a pipe, and exits.
"""

import gc, os, pathlib, pickle, time, pytest
from _pytest.runner import runtestprotocol

HOW = os.environ.get("FORK", "")
LISTED = set(pathlib.Path(HOW).read_text().split()) if HOW not in ("", "all") else set()
COUNT = {"forked": 0, "bundled": 0}
SPLIT = {"fork": 0.0, "child_run": 0.0, "wait_total": 0.0}


def wanted(item):
    if HOW == "all":
        return True
    return f"{item.path.name}::{getattr(item, 'originalname', item.name)}" in LISTED


@pytest.hookimpl(tryfirst=True)
def pytest_runtest_protocol(item, nextitem):
    if not HOW or not wanted(item):
        COUNT["bundled"] += 1
        return None
    COUNT["forked"] += 1
    hook = item.ihook
    hook.pytest_runtest_logstart(nodeid=item.nodeid, location=item.location)
    item.session._setupstate.teardown_exact(None)  # the parent holds nothing the child would share
    if os.environ.get("FORK_FREEZE"):
        gc.freeze()  # keep gc from touching every page the child shares
    r, w = os.pipe()
    t0 = time.perf_counter()
    pid = os.fork()
    if pid == 0:
        os.close(r)
        code = 0
        try:
            c0 = time.perf_counter()
            reports = runtestprotocol(item, nextitem=None, log=False)
            child_run = time.perf_counter() - c0
            fork_lat = c0 - t0
            data = [
                item.config.hook.pytest_report_to_serializable(config=item.config, report=x)
                for x in reports
            ]
            with os.fdopen(w, "wb") as out:
                pickle.dump((data, child_run, fork_lat), out)
        except BaseException:
            code = 1
        os._exit(code)
    os.close(w)
    with os.fdopen(r, "rb") as inp:
        blob = inp.read()
    _, status = os.waitpid(pid, 0)
    SPLIT["wait_total"] += time.perf_counter() - t0
    data_list, child_run, fork_lat = pickle.loads(blob) if blob else ([], 0, 0)
    SPLIT["child_run"] += child_run
    SPLIT["fork"] += fork_lat
    for data in data_list:
        report = item.config.hook.pytest_report_from_serializable(config=item.config, data=data)
        hook.pytest_runtest_logreport(report=report)
    hook.pytest_runtest_logfinish(nodeid=item.nodeid, location=item.location)
    return True


def pytest_terminal_summary(terminalreporter):
    terminalreporter.write_line(f"forkbeaker: {COUNT['forked']} forked, {COUNT['bundled']} bundled")
    n = max(COUNT["forked"], 1)
    terminalreporter.write_line(
        "forkbeaker ms/forked test: "
        + ", ".join(f"{k} {1000 * v / n:.2f}" for k, v in SPLIT.items())
    )
