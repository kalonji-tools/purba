"""Run one way's test with the real copy, then with the planted bug.

usage: python run.py <ways-dir> <module>
Prints one line for each policy the module names.
"""

import importlib
import platform
import sys

ways_dir, name = sys.argv[1], sys.argv[2]
sys.path[:0] = [ways_dir, __file__.rsplit("/", 1)[0]]

import common  # noqa: E402
import sketch  # noqa: E402
import subject  # noqa: E402

real = subject.copy
mod = importlib.import_module(name)
keys = ", ".join(repr(k) for k in sketch.inputs(mod.test_copy).values())


def once(copy, policy):
    common.KEPT.clear()
    common.SEEN.clear()
    subject.copy = copy
    try:
        sketch.call(mod.test_copy, **policy)
        outcome = "passed"
    except sketch.Refused as error:
        return f"refused ({error})", "-", "-"
    except AssertionError:
        outcome = "failed"
    src, dst = common.SEEN[0]
    kept = all(id(db) in common.KEPT for db in (src, dst))
    return outcome, "same" if src is dst else "two", "yes" if kept else "NO"


for label, policy in mod.POLICIES.items():
    good, same, kept = once(real, policy)
    bug, _, _ = once(subject.copy_nothing, policy)
    print(
        f"{platform.python_version():8} {name:18} {label:18} "
        f"copy={good:8} bug={bug:8} objects={same:4} builder={kept:3}"
    )
print(f"{'':8} {name:18} keys: {keys}")
