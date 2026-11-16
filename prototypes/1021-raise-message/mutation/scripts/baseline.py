"""Run each project's suite unmodified with the raisetrace plugin. Output: runs/baseline/<p>.jsonl + .log"""

import sys
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, "/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/e9de2c2b-a74e-4d6e-b5d0-498e964f8ad4/scratchpad/mut/scripts")
from common import PROJ, RUNS, run_pytest  # noqa

d = RUNS / "baseline"
d.mkdir(parents=True, exist_ok=True)
projs = sys.argv[1:] or list(PROJ)


def one(p):
    out = d / f"{p}.jsonl"
    if out.exists():
        out.unlink()
    rc, log = run_pytest(p, PROJ[p][1], out, timeout=1800)
    (d / f"{p}.log").write_text(log)
    return p, rc, log.strip().splitlines()[-1] if log.strip() else ""


with ThreadPoolExecutor(len(projs)) as ex:
    for p, rc, last in ex.map(one, projs):
        print(p, rc, last)
