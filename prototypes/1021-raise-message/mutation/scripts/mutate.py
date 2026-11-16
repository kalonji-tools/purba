"""For each target item: replace its raise statement with `pass`, run only its test, restore.

One mutant per item (the first passing execution's raise site). Projects run in parallel,
mutants within a project run one at a time (they edit the shared mut/src/<p> copy).
Output: runs/mutants/<p>/<key>.jsonl (trace) and runs/results.jsonl.
"""

import ast
import json
import sys
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, "/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/e9de2c2b-a74e-4d6e-b5d0-498e964f8ad4/scratchpad/mut/scripts")
from common import PROJ, RUNS, run_pytest  # noqa
from select_targets import item_events, load  # noqa

targets = json.loads((RUNS / "targets.json").read_text())
projs = sys.argv[1:] or list(PROJ)


def mutate_bytes(src: bytes, span):
    l1, c1, l2, c2 = span  # ast offsets: 1-based lines, UTF-8 byte columns
    lines = src.splitlines(keepends=True)
    head = b"".join(lines[: l1 - 1]) + lines[l1 - 1][:c1]
    tail = lines[l2 - 1][c2:]
    rest = b"".join(lines[l2:])
    return head + b"pass" + tail + rest


def sig(e):
    s = e.get("site", {})
    return (e.get("exc_type"), e.get("exc_str"), s.get("file"), s.get("line"))


def run_one(t):
    p = t["proj"]
    path = t["orig"]["site"]["file"]
    orig = open(path, "rb").read()
    mutant = mutate_bytes(orig, t["span"])
    res = {"key": t["key"], "proj": p, "group": t["group"], "nodeid": t["nodeid"], "site": [path, t["span"]]}
    try:
        ast.parse(mutant)
    except SyntaxError as e:
        res["result"] = "error"
        res["detail"] = f"mutant does not parse: {e}"
        return res
    d = RUNS / "mutants" / p
    d.mkdir(parents=True, exist_ok=True)
    out = d / f"{t['key']}.jsonl"
    if out.exists():
        out.unlink()
    try:
        open(path, "wb").write(mutant)
        rc, log = run_pytest(p, [t["nodeid"]], out, timeout=180)
    finally:
        open(path, "wb").write(orig)
    assert open(path, "rb").read() == orig
    (d / f"{t['key']}.log").write_text(str(log)[-4000:])
    res["rc"] = rc
    if rc == "timeout":
        res["result"] = "error"
        res["detail"] = "timeout"
        return res
    reports, passed, events = load(out) if out.exists() else ({}, set(), [])
    w = reports.get(t["nodeid"], {})
    if t["nodeid"] in passed:
        res["result"] = "survived"
    elif w.get("call") == "failed":
        res["result"] = "killed"
    else:
        res["result"] = "error"
        res["detail"] = f"reports={w} rc={rc}"
    # what reached the item in the mutant run vs baseline
    ie, _ = item_events(events)
    mev = ie.get((t["key"], t["nodeid"]), [])
    res["mutant_events"] = mev
    base = t["_base_events"]
    diff = None
    for i in range(max(len(base), len(mev))):
        b = base[i] if i < len(base) else None
        m = mev[i] if i < len(mev) else None
        if b is None or m is None or sig(b) != sig(m):
            diff = {"index": i, "base": b, "mutant": m}
            break
    res["first_diff"] = diff
    return res


def do_project(p):
    _, _, events = load(RUNS / "baseline" / f"{p}.jsonl")
    ie, _ = item_events(events)
    rs = []
    for t in targets:
        if t["proj"] != p or t["status"] != "target":
            continue
        t["_base_events"] = ie[(t["key"], t["nodeid"])]
        rs.append(run_one(t))
    return rs


with ThreadPoolExecutor(len(projs)) as ex, open(RUNS / "results.jsonl", "a") as fh:
    for rs in ex.map(do_project, projs):
        for r in rs:
            fh.write(json.dumps(r) + "\n")
        if rs:
            print(rs[0]["proj"], len(rs), {k: sum(1 for r in rs if r["result"] == k) for k in ("survived", "killed", "error")})
