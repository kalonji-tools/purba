"""PROTOTYPE (#689): the ground truth. Diff shared state around each test, setup to teardown.

A test LEAKS when something it leaves behind would reach the next test in the same
interpreter. Load with `-p leakwatch`; set LEAKWATCH_OUT and LEAKWATCH_ROOTS (paths
whose modules' globals are watched, joined by os.pathsep).
"""

import json, logging, os, signal, socket, sys, time, types, warnings, random, locale
import pytest

ROOTS = [os.path.realpath(p) for p in os.environ["LEAKWATCH_ROOTS"].split(os.pathsep)]
OUT = os.environ["LEAKWATCH_OUT"]
DEPTH = int(os.environ.get("LEAKWATCH_DEPTH", "3"))
SCALARS = (int, float, complex, bool, str, bytes, type(None))
RESULTS = []
SNAP_TIME = [0.0, 0]


def fp(value, depth, seen):
    """A fingerprint of CONTENT, to DEPTH.

    No id of a nested object, since some are rebuilt on each read.
    """
    if isinstance(value, SCALARS):
        return value if not isinstance(value, (str, bytes)) or len(value) < 200 else hash(value)
    if id(value) in seen:
        return ("cycle",)
    if depth == 0:
        return ("deep", type(value).__name__)
    seen = seen | {id(value)}
    try:
        if isinstance(value, types.ModuleType):
            return ("module", value.__name__)
        if isinstance(value, dict):
            return (
                "dict",
                tuple(sorted((repr(k)[:80], fp(v, depth - 1, seen)) for k, v in value.items())),
            )
        if isinstance(value, (list, tuple)):
            return (type(value).__name__, tuple(fp(v, depth - 1, seen) for v in value))
        if isinstance(value, (set, frozenset)):
            return ("set", tuple(sorted(repr(v)[:80] for v in value)))
        if isinstance(value, type):
            own = {k: v for k, v in vars(value).items() if not k.startswith("__")}
            return (
                "class",
                value.__qualname__,
                tuple(sorted((k, fp(v, depth - 1, seen)) for k, v in own.items())),
            )
        if callable(value) and not isinstance(getattr(value, "__dict__", None), dict):
            return ("callable", getattr(value, "__qualname__", type(value).__name__))
        attrs = getattr(value, "__dict__", None)
        if isinstance(attrs, dict):
            return ("obj", type(value).__qualname__, fp(dict(attrs), depth - 1, seen))
        return ("opaque", type(value).__qualname__)
    except Exception:
        return ("error", type(value).__name__)


def watched_modules():
    mods = {}
    for name, mod in list(sys.modules.items()):
        path = getattr(mod, "__file__", None)
        if path and any(os.path.realpath(path).startswith(r) for r in ROOTS):
            mods[name] = mod
    return mods


def snapshot():
    start = time.perf_counter()
    root = logging.getLogger()
    snap = {
        "os.environ": dict(os.environ),
        "cwd": os.getcwd(),
        "sys.path": list(sys.path),
        "sys.modules (removed)": set(sys.modules),
        "sys.stdout/stderr/stdin": (id(sys.stdout), id(sys.stderr), id(sys.stdin)),
        "warnings.filters": list(warnings.filters),
        "logging root": (
            root.level,
            tuple(id(h) for h in root.handlers),
            logging.root.manager.disable,
        ),
        "signal handlers": tuple(signal.getsignal(s) for s in (signal.SIGINT, signal.SIGTERM)),
        "recursion limit": sys.getrecursionlimit(),
        "locale": locale.setlocale(locale.LC_ALL),
        "socket timeout": socket.getdefaulttimeout(),
        "random state": hash(random.getstate()),
        "umask": os.umask(os.umask(0o22)),
    }
    globs = {}
    for name, mod in watched_modules().items():
        globs[name] = {
            k: (None if isinstance(v, SCALARS) else id(v), fp(v, DEPTH, set()))
            for k, v in list(vars(mod).items())
            if not k.startswith("__")
        }
    snap["globals"] = globs
    SNAP_TIME[0] += time.perf_counter() - start
    SNAP_TIME[1] += 1
    return snap


def diff(before, after):
    found = []
    for key in before:
        if key in ("globals", "sys.modules (removed)"):
            continue
        if before[key] != after[key]:
            if key == "os.environ":
                changed = sorted(
                    k
                    for k in set(before[key]) | set(after[key])
                    if before[key].get(k) != after[key].get(k)
                )
                found.append(f"os.environ {changed[:4]}")
            else:
                found.append(key)
    removed = before["sys.modules (removed)"] - after["sys.modules (removed)"]
    if removed:
        found.append(f"sys.modules removed {sorted(removed)[:3]}")
    for mod, names in before["globals"].items():
        now = after["globals"].get(mod)
        if now is None:
            continue
        for name in sorted(set(names) | set(now)):
            if names.get(name) != now.get(name):
                found.append(f"{mod}.{name}")
    return found


@pytest.hookimpl(hookwrapper=True)
def pytest_runtest_protocol(item, nextitem):
    before = snapshot()
    yield
    after = snapshot()
    RESULTS.append(
        {
            "nodeid": item.nodeid,
            "func": f"{item.path.name}::{getattr(item, 'originalname', item.name)}",
            "leaks": diff(before, after),
            "new_modules": len(after["sys.modules (removed)"] - before["sys.modules (removed)"]),
        }
    )


def pytest_sessionfinish(session):
    with open(OUT, "w") as fh:
        json.dump(
            {"snapshot_seconds": SNAP_TIME[0], "snapshots": SNAP_TIME[1], "items": RESULTS},
            fh,
            indent=1,
        )
