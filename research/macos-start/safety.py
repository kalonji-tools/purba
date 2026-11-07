"""RESEARCH (#852): which work in a parent breaks a fork without exec.

usage: safety.py CASE
The parent runs the case's setup, counts its own OS threads, and forks.
The child runs the case's action and exits 0. The parent prints one JSON line:
the threads before the fork, and how the child ended (exit, signal, or hang).
run.py runs each case in a fresh interpreter, so no case sees another's setup.
"""

import ctypes, json, os, signal, sys, threading, time

HANG_AFTER = 20.0


def os_threads():
    """The OS threads of this process, not only Python's."""
    if sys.platform == "darwin":
        # proc_pidinfo(PROC_PIDTASKINFO) fills struct proc_taskinfo, <sys/proc_info.h>.
        class TaskInfo(ctypes.Structure):
            wide = "virt res user sys threads_user threads_sys"
            _fields_ = [(n, ctypes.c_uint64) for n in wide.split()] + [
                (n, ctypes.c_int32)
                for n in "pol flt pin cow ms mr scm scu csw threadnum nrun pri".split()
            ]

        info = TaskInfo()
        lib = ctypes.CDLL("/usr/lib/libproc.dylib")
        got = lib.proc_pidinfo(os.getpid(), 4, 0, ctypes.byref(info), ctypes.sizeof(info))
        return info.threadnum if got == ctypes.sizeof(info) else -1
    return len(os.listdir("/proc/self/task"))


def proxies():
    import urllib.request

    return urllib.request.getproxies()


def cf_preferences():
    """A CoreFoundation call that reaches cfprefsd, as a framework user does."""
    cf = ctypes.CDLL("/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation")
    cf.CFStringCreateWithCString.restype = ctypes.c_void_p
    cf.CFStringCreateWithCString.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_uint32]
    cf.CFPreferencesCopyAppValue.restype = ctypes.c_void_p
    cf.CFPreferencesCopyAppValue.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
    key = cf.CFStringCreateWithCString(None, b"AppleLocale", 0x08000100)
    app = ctypes.c_void_p.in_dll(cf, "kCFPreferencesAnyApplication")
    return cf.CFPreferencesCopyAppValue(key, app)


def dns():
    import socket

    return socket.getaddrinfo("example.com", 443)


def tls():
    import ssl

    return ssl.create_default_context()


def mac_ver():
    import platform

    return platform.mac_ver()


def matmul():
    import numpy

    a = numpy.ones((400, 400))
    return float((a @ a).sum())


def nothing():
    return None


def loop(fn):
    """Keep fn running on a second thread while the main thread forks."""

    def body():
        while True:
            fn()

    threading.Thread(target=body, daemon=True).start()
    time.sleep(0.5)


def idle_thread():
    threading.Thread(target=threading.Event().wait, daemon=True).start()


def import_only(name):
    return lambda: __import__(name)


# case: (setup in the parent, action in the child)
CASES = {
    "bare": (nothing, nothing),
    "idle-thread": (idle_thread, nothing),
    "proxies-child-only": (nothing, proxies),
    "proxies-import-parent": (import_only("urllib.request"), proxies),
    "proxies-both": (proxies, proxies),
    "proxies-parent-only": (proxies, nothing),
    "proxies-thread-both": (lambda: loop(proxies), proxies),
    "cf-prefs-both": (cf_preferences, cf_preferences),
    "cf-prefs-parent-only": (cf_preferences, nothing),
    "dns-both": (dns, dns),
    "tls-both": (tls, tls),
    "mac-ver-both": (mac_ver, mac_ver),
    "numpy-import-parent": (import_only("numpy"), matmul),
    "numpy-both": (matmul, matmul),
}


def child_end(pid):
    deadline = time.monotonic() + HANG_AFTER
    while time.monotonic() < deadline:
        got, status = os.waitpid(pid, os.WNOHANG)
        if got:
            if os.WIFSIGNALED(status):
                return f"signal {signal.Signals(os.WTERMSIG(status)).name}"
            return f"exit {os.WEXITSTATUS(status)}"
        time.sleep(0.01)
    os.kill(pid, signal.SIGKILL)
    os.waitpid(pid, 0)
    return f"hang (killed after {HANG_AFTER:.0f} s)"


def main():
    case = sys.argv[1]
    setup, action = CASES[case]
    setup()
    threads = os_threads()
    sys.stdout.flush()
    sys.stderr.flush()
    pid = os.fork()
    if pid == 0:
        try:
            action()
            os._exit(0)
        except BaseException as error:
            print(f"child raised {type(error).__name__}: {error}", file=sys.stderr)
            os._exit(1)
    print(json.dumps({"case": case, "threads_at_fork": threads, "child": child_end(pid)}))


if __name__ == "__main__":
    main()
