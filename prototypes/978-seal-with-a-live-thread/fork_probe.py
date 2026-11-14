"""What a fork after the act does to a live thread. Each scenario forks a child,
which runs one "check"; the parent waits at most 3 s, then reports."""
import _thread
import io
import os
import sys
import threading
import time
import warnings

from loguru import logger


def threads():
    with open("/proc/self/stat") as f:
        os_n = int(f.read().rsplit(")", 1)[1].split()[17])
    return f"threading={threading.active_count()} os={os_n}"


def fork_check(label, check):
    r, w = os.pipe()
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        t0 = time.perf_counter()
        pid = os.fork()
    if pid == 0:
        os.close(r)
        try:
            msg = check()
        except BaseException as e:  # noqa: BLE001
            msg = f"raised {type(e).__name__}: {e}"
        os.write(w, msg.encode())
        os._exit(0)
    os.close(w)
    deadline = time.time() + 3
    while time.time() < deadline:
        done, _ = os.waitpid(pid, os.WNOHANG)
        if done:
            break
        time.sleep(0.01)
    else:
        os.kill(pid, 9)
        os.waitpid(pid, 0)
        print(f"  {label}: CHILD HUNG >3 s, killed  [{threads()}]")
        return
    out = os.read(r, 4096).decode()
    warn = "warned" if any("multi-threaded" in str(c.message) for c in caught) else "no warning"
    print(f"  {label}: {out}  ({warn}, {time.perf_counter()-t0:.3f}s) [{threads()}]")


print("A. loguru enqueue worker alive, act wrote 'Oops', slept 0.1 s")
buf = io.StringIO()
hid = logger.add(buf, format="{message}", enqueue=True)
logger.remove(0) if 0 in logger._core.handlers else None
logger.debug("Oops")
time.sleep(0.1)
fork_check("child reads what the act wrote", lambda: repr(buf.getvalue()))
def log_more():
    logger.debug("more")
    time.sleep(0.2)
    return repr(buf.getvalue())


fork_check("child logs 'more', waits 0.2 s, reads", log_more)
fork_check("child calls logger.remove()", lambda: (logger.remove(hid), "remove returned")[1])
logger.remove(hid)

print("B. act's write still in the queue at the fork (no sleep)")
buf2 = io.StringIO()
hid2 = logger.add(buf2, format="{message}", enqueue=True)
for i in range(2000):
    logger.debug(f"m{i}")
fork_check("child reads count of lines", lambda: f"{buf2.getvalue().count(chr(10))} of 2000 lines")
logger.remove(hid2)
print(f"  parent after remove: {buf2.getvalue().count(chr(10))} of 2000 lines")

print("C. a test's own thread holds a plain lock in a loop")
lock = threading.Lock()
stop = False


def holder():
    while not stop:
        with lock:
            time.sleep(0.005)


t = threading.Thread(target=holder, daemon=True)
t.start()
time.sleep(0.05)
hung = 0
for i in range(10):
    fork_check(f"try {i}: child takes the lock", lambda: (lock.acquire(), "got lock")[1])
stop = True
t.join()

print("D. a thread that threading does not list (_thread.start_new_thread)")
_thread.start_new_thread(lambda: time.sleep(1), ())
time.sleep(0.05)
print(f"  {threads()}  enumerate={[x.name for x in threading.enumerate()]}")
fork_check("child", lambda: "ran")
