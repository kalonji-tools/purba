"""Option: when a thread is alive after the act, each other check gets its own
case process (forked from the single-threaded zygote) that builds the inputs,
runs the act again, and runs that one check. loguru's broken-sink test, enqueue."""
import io
import os
import re
import sys
import threading
import time

from loguru import logger

logger.remove()


def broken_sink(m):
    raise ValueError("Error!")


def act():
    err = io.StringIO()
    sys.stderr = err
    logger.add(broken_sink, catch=True, enqueue=True)
    logger.debug("Oops")
    time.sleep(0.1)
    return err


CHECKS = [
    lambda lines: lines[0] == "--- Logging error in Loguru Handler #1 ---",
    lambda lines: re.match(r"Record was: \{.*Oops.*\}", lines[1]) is not None,
    lambda lines: lines[-2].startswith("ValueError: Error!"),
    lambda lines: lines[-1] == "--- End of logging error ---",
]


def run_case(k):
    """One case process: inputs + act + check k. Returns the verdict by exit code."""
    pid = os.fork()
    if pid == 0:
        err = act()
        alive = threading.active_count()
        ok = CHECKS[k](err.getvalue().strip().splitlines())
        mark = "✓" if ok else "✗"
        sys.__stderr__.write(f"    check {k}: {mark}  threads alive at check={alive}\n")
        os._exit(0 if ok else 1)
    return pid


assert threading.active_count() == 1, "zygote must be single-threaded"
t0 = time.perf_counter()
pids = [run_case(k) for k in range(len(CHECKS))]  # in parallel
codes = [os.waitpid(p, 0)[1] for p in pids]
print(f"  replay, 4 case processes in parallel: {time.perf_counter()-t0:.3f}s, exit codes {codes}")
t0 = time.perf_counter()
for k in range(len(CHECKS)):
    os.waitpid(run_case(k), 0)
print(f"  replay, 4 case processes one after another: {time.perf_counter()-t0:.3f}s")
t0 = time.perf_counter()
pid = os.fork()
if pid == 0:
    act()
    os._exit(0)
os.waitpid(pid, 0)
print(f"  one act, for scale: {time.perf_counter()-t0:.3f}s")
