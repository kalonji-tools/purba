"""Run test_copy, then test_starts_clean, in one session.

usage: python run_generic.py <ways-dir> <module>
Each line: one policy, one copy (the real one or the planted bug), and the
adapter's rollback on or off.
"""

import importlib
import platform
import sys

ways_dir, name = sys.argv[1], sys.argv[2]
sys.path[:0] = [ways_dir, __file__.rsplit("/", 1)[0]]

import adapter  # noqa: E402
import common  # noqa: E402
import sketch  # noqa: E402
import subject  # noqa: E402

real = subject.copy
mod = importlib.import_module(name)


def outcome(test, policy):
    try:
        sketch.call(test, **policy)
        return "passed"
    except sketch.Refused as error:
        return f"refused ({error})"
    except AssertionError:
        return "failed"


for label, policy in mod.POLICIES.items():
    for copy_name, copy in (("copy", real), ("bug", subject.copy_nothing)):
        for rollback in (True, False):
            sketch.SESSION.clear()
            common.KEPT.clear()
            common.SEEN.clear()
            adapter.rollback = rollback
            subject.copy = copy
            first = outcome(mod.test_copy, policy)
            second = outcome(mod.test_starts_clean, policy)
            facts = ""
            if len(common.SEEN) == 2:
                (src, dst), (src2, dst2) = common.SEEN
                facts = (
                    f" objects={'same' if src is dst else 'two'}"
                    f" shared={'yes' if src is src2 and dst is dst2 else 'no'}"
                    f" builds={len(common.KEPT)}"
                )
            print(
                f"{platform.python_version():8} {name:24} {label:7} {copy_name:4} "
                f"rollback={'on ' if rollback else 'off'} "
                f"test_copy={first:9} test_starts_clean={second:9}{facts}"
            )
