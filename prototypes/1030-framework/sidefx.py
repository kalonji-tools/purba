"""Compare interpreter state before and after importing a module named in argv."""
import atexit
import importlib
import signal
import sys
import threading
import time
import warnings


def state():
    return {
        "meta_path": [type(f).__name__ for f in sys.meta_path],
        "path_hooks": len(sys.path_hooks),
        "warnings.filters": len(warnings.filters),
        "showwarning": warnings.showwarning.__module__,
        "excepthook": sys.excepthook.__module__,
        "displayhook": sys.displayhook.__module__,
        "SIGINT": repr(signal.getsignal(signal.SIGINT)),
        "SIGALRM": repr(signal.getsignal(signal.SIGALRM)),
        "threads": threading.active_count(),
        "atexit": atexit._ncallbacks(),
        "trace": sys.gettrace(),
        "profile": sys.getprofile(),
        "recursionlimit": sys.getrecursionlimit(),
        "modules": len(sys.modules),
    }


name = sys.argv[1]
before = state()
t = time.perf_counter()
importlib.import_module(name)
ms = (time.perf_counter() - t) * 1000
after = state()
print(f"--- import {name}: {ms:.0f} ms")
for k in before:
    mark = "CHANGED" if before[k] != after[k] else "same"
    print(f"{k:18} {mark:8} {before[k]!r} -> {after[k]!r}" if mark == "CHANGED" else f"{k:18} same")
