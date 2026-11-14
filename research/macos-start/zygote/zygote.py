"""RESEARCH (#694): a stand-in for purba's zygote, in Python.

usage: zygote.py [--threads=join|exit] MODULE.py [MODULE.py ...]

The parent (purba) forks a zygote for each module. The zygote loads the module
once and forks a child for each test. The child runs the test, then hands back
one length-framed JSON outcome on a pipe and calls os._exit. A child that ends
without a frame is described by its wait status.

If a thread other than the main one is alive after the load, the zygote forks
no test: each test then runs in a process forked from the parent, which loads
the module again (Q1 answer a, as #978 does for the act).

Interrupt: the parent and the zygote keep a flag on SIGINT and wait for the
child; the child gets Python's default handler back. A second SIGINT in the
parent exits at once, by the signal.
"""

import json, os, signal, struct, sys, threading, time, traceback

THREADS = "join"
INTERRUPTS = 0


def on_sigint(signum, frame):
    global INTERRUPTS
    INTERRUPTS += 1
    if INTERRUPTS >= 2:
        signal.signal(signal.SIGINT, signal.SIG_DFL)
        os.kill(os.getpid(), signal.SIGINT)


def load(path):
    namespace = {"__name__": os.path.basename(path)[:-3], "__file__": path}
    with open(path) as f:
        code = compile(f.read(), path, "exec")
    exec(code, namespace)
    return namespace


def tests_of(namespace):
    return [n for n, v in namespace.items() if n.startswith("test_") and callable(v)]


def read_frame(fd):
    head = b""
    while len(head) < 4:
        try:
            got = os.read(fd, 4 - len(head))
        except InterruptedError:
            continue
        if not got:
            return None
        head += got
    (length,) = struct.unpack("<I", head)
    body = b""
    while len(body) < length:
        got = os.read(fd, length - len(body))
        if not got:
            return None
        body += got
    return json.loads(body)


def describe(status):
    if os.WIFSIGNALED(status):
        sig = os.WTERMSIG(status)
        return f"its process died by {signal.Signals(sig).name}"
    return f"its process exited {os.WEXITSTATUS(status)} before its outcome"


def child_runs(namespace, name, wfd):
    """In the child: run one test, hand back its outcome, exit."""
    signal.signal(signal.SIGINT, signal.default_int_handler)
    errors = []
    threading.excepthook = lambda args: errors.append(
        f"{args.exc_type.__name__}: {args.exc_value}"
    )
    try:
        namespace[name]()
        outcome = {"kind": "passed"}
    except KeyboardInterrupt:
        outcome = {"kind": "interrupted"}
    except AssertionError as e:
        outcome = {"kind": "failed", "text": str(e)}
    except BaseException as e:  # noqa: BLE001
        outcome = {"kind": "raised", "text": f"{type(e).__name__}: {e}"}
    if THREADS == "join":
        for t in threading.enumerate():
            if t is not threading.main_thread() and not t.daemon:
                t.join()
    outcome["threads"] = errors
    body = json.dumps(outcome).encode()
    os.write(wfd, struct.pack("<I", len(body)) + body)
    os._exit(0)


def fork_test(namespace_or_path, name, module):
    """Fork one test; the namespace is loaded already, or a path to load first."""
    rfd, wfd = os.pipe()
    pid = os.fork()
    if pid == 0:
        os.close(rfd)
        ns = namespace_or_path
        if isinstance(ns, str):
            ns = load(ns)
        child_runs(ns, name, wfd)
    os.close(wfd)
    frame = read_frame(rfd)
    os.close(rfd)
    while True:
        try:
            _, status = os.waitpid(pid, 0)
            break
        except InterruptedError:
            continue
    line = f"{module}::{name}"
    if frame is None:
        print(f"{line} raised: {describe(status)}", flush=True)
        return
    kind = frame["kind"]
    text = f": {frame['text']}" if "text" in frame else ""
    print(f"{line} {kind}{text}", flush=True)
    for error in frame["threads"]:
        print(f"{line} project error: thread raised {error}", flush=True)


def zygote(path):
    """In the zygote: load once, fork each test."""
    module = os.path.basename(path)
    try:
        namespace = load(path)
    except BaseException as e:  # noqa: BLE001
        print(f"{module} project error: {type(e).__name__}: {e}", flush=True)
        return "loadfail", []
    names = tests_of(namespace)
    if threading.active_count() > 1:
        return "thread", names
    for name in names:
        if INTERRUPTS:
            print(f"{module}::{name} unreached", flush=True)
            continue
        fork_test(namespace, name, module)
    return "ok", []


def run_module(path):
    rfd, wfd = os.pipe()
    pid = os.fork()
    if pid == 0:
        os.close(rfd)
        verdict, names = zygote(path)
        os.write(wfd, json.dumps([verdict, names, INTERRUPTS]).encode())
        os._exit(0)
    os.close(wfd)
    data = b""
    while chunk := os.read(rfd, 65536):
        data += chunk
    os.close(rfd)
    while True:
        try:
            _, status = os.waitpid(pid, 0)
            break
        except InterruptedError:
            continue
    if not data:
        print(f"{os.path.basename(path)} internal error: zygote {describe(status)}")
        return
    verdict, names, zygote_interrupts = json.loads(data)
    if verdict == "thread":
        module = os.path.basename(path)
        print(f"{module}: a thread is alive after the load, each test loads alone", flush=True)
        for name in names:
            if INTERRUPTS:
                print(f"{module}::{name} unreached", flush=True)
                continue
            fork_test(path, name, module)


def main():
    global THREADS
    args = sys.argv[1:]
    if args and args[0].startswith("--threads="):
        THREADS = args.pop(0).split("=", 1)[1]
    signal.signal(signal.SIGINT, on_sigint)
    start = time.perf_counter()
    for path in args:
        if INTERRUPTS:
            print(f"{os.path.basename(path)} unreached", flush=True)
            continue
        run_module(path)
    print(f"-- {time.perf_counter() - start:.2f} s", flush=True)
    if INTERRUPTS:
        signal.signal(signal.SIGINT, signal.SIG_DFL)
        os.kill(os.getpid(), signal.SIGINT)


if __name__ == "__main__":
    main()
