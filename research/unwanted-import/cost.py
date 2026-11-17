"""What each means costs in time, on one way, one run after another.

    python cost.py <python to probe> <site-packages that holds pytest> <work dir>

For each means it runs the child on two test modules, one that imports pytest
and one that imports `json` only, nine times each, and prints the median wall
time of the whole child process, start-up included. The difference between a
means and `none` is what the means adds. It also prints how many modules
`modulefinder` walks, and how long it takes, for the module that imports
`json` only.
"""

import json
import os
import statistics
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from matrix import FINDER, filtered_site  # noqa: E402

MEANS = [
    "none",
    "modules-diff",
    "audit-open",
    "monitoring-code",
    "modules-none",
    "meta-path-finder",
    "builtins-import",
    "audit-import",
    "path-without-target",
]


def timed(args):
    start = time.perf_counter()
    subprocess.run(args, capture_output=True, check=False)
    return time.perf_counter() - start


def main():
    python, site, work = sys.argv[1:4]
    filtered = filtered_site(site, work)
    tests = {"imports pytest": "import pytest\n", "imports json": "import json\n"}
    for label, text in tests.items():
        way = os.path.join(work, "cost-" + label.replace(" ", "-"))
        os.makedirs(os.path.join(way, "lib"), exist_ok=True)
        with open(os.path.join(way, "test_way.py"), "w") as f:
            f.write(text)
        print(f"== {label}")
        child = [os.path.join(HERE, "child.py"), way]
        rows = {m: [python, *child, m, site, filtered] for m in MEANS}
        rows["importtime"] = [python, "-X", "importtime", *child, "none", site, filtered]
        rows["strace-open"] = ["strace", "-f", "-qq", "-e", "trace=openat", "-o", os.devnull,
                               python, *child, "none", site, filtered]
        for name, args in rows.items():
            seconds = statistics.median(timed(args) for _ in range(9))
            print(f"{name:22} {seconds * 1000:7.1f} ms")
        done = subprocess.run(
            [python, "-c", FINDER, os.path.join(way, "lib"), os.path.join(way, "test_way.py")],
            capture_output=True,
            text=True,
        )
        found = json.loads(done.stdout.strip().splitlines()[-1])
        print(f"modulefinder           {found['seconds'] * 1000:7.1f} ms,"
              f" {found['modules']} modules walked")


main()
