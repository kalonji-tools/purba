"""Runs one way under one in-process means, and prints what happened as JSON.

    python child.py <way dir> <means> <site-packages> [<filtered site>] [--preload]

It loads `test_way.py` as purba loads a test module: it compiles the text,
runs it in a new module that it put in `sys.modules`, then calls each
`test_*` function (crates/assertion/src/lib.rs, `load` and `run`).

The JSON says which hits the means recorded, what evidence shows that code
of pytest's package ran or that a module named `pytest` or `_pytest` was
bound, and which errors the load and the calls raised.
"""

import builtins
import gc
import json
import os
import sys
import types

BLOCK = ("pytest", "_pytest")


def top(name):
    return name.split(".")[0]


def main():
    way, means, site = sys.argv[1], sys.argv[2], sys.argv[3]
    filtered = sys.argv[4] if len(sys.argv) > 4 and not sys.argv[4].startswith("--") else None
    preload = "--preload" in sys.argv
    os.environ["PROBE_SITE"] = site
    dirs = tuple(os.path.join(site, name) + os.sep for name in BLOCK)
    sys.path.insert(0, os.path.join(way, "lib"))
    hits = []
    end = {}

    def at_exit():
        # Runs after each callback that the test registered, since atexit runs
        # its callbacks last in, first out.
        late = [n for n in sys.modules if top(n) in BLOCK and n not in end.get("names", ())]
        print(json.dumps({"exit_hits": hits[end.get("hits", 0):], "exit_names": late}))

    import atexit

    atexit.register(at_exit)

    if preload:
        import pytest  # noqa: F401  # the environment loaded it before the test

    before_names = set(sys.modules)
    before_modules = {id(m) for m in gc.get_objects() if isinstance(m, types.ModuleType)}

    if means == "modules-none":
        for name in BLOCK:
            sys.modules[name] = None
    elif means == "meta-path-finder":

        class Refuse:
            @staticmethod
            def find_spec(name, path=None, target=None):
                if top(name) in BLOCK:
                    hits.append(name)
                    raise ImportError(f"REFUSED {name}")
                return None

        sys.meta_path.insert(0, Refuse)
    elif means == "builtins-import":
        original = builtins.__import__

        def refusing(name, globals=None, locals=None, fromlist=(), level=0):
            absolute = name
            if level > 0:
                # A relative import names its package through the importer's
                # globals.
                # https://docs.python.org/3.12/reference/import.html#package-relative-imports
                package = (globals or {}).get("__package__") or ""
                absolute = package.rsplit(".", level - 1)[0] + ("." + name if name else "")
            if top(absolute) in BLOCK:
                hits.append(absolute)
                raise ImportError(f"REFUSED {absolute}")
            return original(name, globals, locals, fromlist, level)

        builtins.__import__ = refusing
    elif means == "audit-import":

        def hook(event, args):
            if event == "import" and top(str(args[0])) in BLOCK:
                hits.append(str(args[0]))
                raise ImportError(f"REFUSED {args[0]}")

        sys.addaudithook(hook)
    elif means == "audit-open":

        def hook(event, args):
            if event == "open" and isinstance(args[0], str) and args[0].startswith(dirs):
                hits.append(os.path.relpath(args[0], site))

        sys.addaudithook(hook)
    elif means == "monitoring-code":
        tool = 4
        sys.monitoring.use_tool_id(tool, "probe")
        disable = sys.monitoring.DISABLE

        def start(code, offset):
            if code.co_filename.startswith(dirs):
                hits.append(os.path.relpath(code.co_filename, site) + ":" + code.co_name)
            return disable

        sys.monitoring.register_callback(tool, sys.monitoring.events.PY_START, start)
        sys.monitoring.set_events(tool, sys.monitoring.events.PY_START)
    elif means == "path-without-target":
        sys.path[:] = [filtered if os.path.realpath(p) == os.path.realpath(site) else p
                       for p in sys.path]
        import importlib

        importlib.invalidate_caches()
    elif means not in ("none", "modules-diff"):
        raise SystemExit(f"unknown means {means}")

    # A mark that an observer in another process reads: what comes after it
    # is the test's, and what comes before it is the environment's.
    sys.stderr.write("PROBE-LOAD-START\n")
    sys.stderr.flush()
    try:
        open("/nonexistent/probe-load-start")
    except OSError:
        pass
    errors = []
    test = os.path.join(way, "test_way.py")
    with open(test) as f:
        text = f.read()
    module = types.ModuleType("test_way")
    module.__file__ = test
    sys.modules["test_way"] = module
    try:
        exec(compile(text, test, "exec"), module.__dict__)
    except BaseException as err:  # noqa: BLE001  # a probe records every error
        errors.append(f"load: {type(err).__name__}: {str(err)[:120]}")
    if means == "modules-diff":
        hits += [f"after load: {k}" for k in set(sys.modules) - before_names if top(k) in BLOCK]
        seen = set(sys.modules)
    for name, value in list(module.__dict__.items()):
        if name.startswith("test_") and callable(value):
            try:
                value()
            except BaseException as err:  # noqa: BLE001
                errors.append(f"{name}: {type(err).__name__}: {str(err)[:120]}")
    if means == "modules-diff":
        hits += [f"after call: {k}" for k in set(sys.modules) - seen if top(k) in BLOCK]
    if means == "monitoring-code":
        sys.monitoring.set_events(4, 0)

    evidence = []
    for name in sorted(set(sys.modules) - before_names):
        mod = sys.modules[name]
        if top(name) in BLOCK and mod is not None:
            where = getattr(mod, "__file__", "") or ""
            inside = where.startswith(dirs)
            outside = f" (file {os.path.relpath(where, way)})"
            evidence.append(f"name:{name}" + ("" if inside else outside))
    for mod in gc.get_objects():
        if isinstance(mod, types.ModuleType) and id(mod) not in before_modules:
            where = getattr(mod, "__file__", None) or ""
            if where.startswith(dirs) and mod.__name__.split(".")[0] not in BLOCK:
                evidence.append(f"file:{os.path.relpath(where, site)} as {mod.__name__}")
            elif where.startswith(dirs) and sys.modules.get(mod.__name__) is not mod:
                evidence.append(f"file:{os.path.relpath(where, site)} (not in sys.modules)")
    for name, value in list(module.__dict__.items()):
        if isinstance(value, types.ModuleType) and top(value.__name__) in BLOCK:
            evidence.append(f"bound:{name}={value.__name__}")
    end.update(hits=len(hits), names=set(sys.modules))
    print(json.dumps({"hits": hits, "evidence": evidence, "errors": errors}))


main()
