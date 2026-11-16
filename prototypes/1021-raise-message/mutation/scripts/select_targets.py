"""Join static items with baseline raise events. Output runs/targets.json.

Per item: kept? (reached by a passing test, the exception was caught by the item),
first passing execution (nodeid + ordinal of that item's executions in the nodeid),
its site kind (own-raise / implicit / outside), and the raise statement span.
"""

import ast
import collections
import json
import sys

sys.path.insert(0, "/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/e9de2c2b-a74e-4d6e-b5d0-498e964f8ad4/scratchpad/mut/scripts")
from common import PROJ, RUNS  # noqa

items = [i for i in json.loads((RUNS / "items.json").read_text()) if i["proj"] in PROJ]
by_line = collections.defaultdict(list)
for k, it in enumerate(items):
    it["key"] = k
    by_line[(it["file"], it["with_line"])].append(it)
    if it["ctx_line"] != it["with_line"]:
        by_line[(it["file"], it["ctx_line"])].append(it)


def load(path):
    reports = collections.defaultdict(dict)
    events = []
    for line in path.read_text().splitlines():
        r = json.loads(line)
        if r["ev"] == "report":
            reports[r["nodeid"]][r["when"]] = r["outcome"]
        else:
            events.append(r)
    passed = {n for n, w in reports.items() if w.get("call") == "passed" and all(v == "passed" for v in w.values())}
    return reports, passed, events


def match_item(ev):
    cands = by_line.get((ev["with_file"], ev["with_line"]), [])
    if len(cands) > 1:
        tails = {e.rsplit(".", 1)[-1] for e in ev["expected"]}
        c2 = [c for c in cands if set(c["types"]) & tails] or cands
        cands = c2
    if len(cands) > 1:
        uniq = {c["key"] for c in cands}
        if len(uniq) > 1:
            return None  # ambiguous
    return cands[0] if cands else None


_ast_cache = {}


def raise_span(path, line):
    if path not in _ast_cache:
        _ast_cache[path] = ast.parse(open(path, "rb").read())
    best = None
    for n in ast.walk(_ast_cache[path]):
        if isinstance(n, ast.Raise) and n.lineno <= line <= n.end_lineno:
            if best is None or (n.end_lineno - n.lineno) < (best.end_lineno - best.lineno):
                best = n
    if best is None:
        return None
    return [best.lineno, best.col_offset, best.end_lineno, best.end_col_offset]


def item_events(events):
    """Return {(item_key, nodeid): [events in order]} and count of unmatched/ambiguous."""
    out = collections.defaultdict(list)
    unmatched = 0
    for ev in events:
        it = match_item(ev)
        if it is None:
            unmatched += 1
            continue
        out[(it["key"], ev["nodeid"])].append(ev)
    return out, unmatched


if __name__ == "__main__":
    targets = []
    stats = {}
    for p in PROJ:
        reports, passed, events = load(RUNS / "baseline" / f"{p}.jsonl")
        ie, unmatched = item_events(events)
        stats[p] = {"events": len(events), "unmatched_events": unmatched}
        per_item = collections.defaultdict(list)
        for (k, nid), evs in ie.items():
            per_item[k].append((nid, evs))
        for it in [i for i in items if i["proj"] == p]:
            t = dict(it)
            runs = per_item.get(it["key"], [])
            good = [(nid, evs) for nid, evs in runs if nid in passed and evs and all(e["outcome"] == "caught" for e in evs)]
            t["n_nodeids"] = len(runs)
            if not good:
                t["status"] = "not kept"
                t["why"] = "never reached" if not runs else "no passing test with caught exception"
                targets.append(t)
                continue
            nid, evs = good[0]
            ev = evs[0]
            sites = {(e["site"].get("file"), e["site"].get("line")) for _, es in good for e in es}
            t["distinct_sites_all_execs"] = len(sites)
            t["nodeid"] = nid
            t["n_execs_in_nodeid"] = len(evs)
            t["orig"] = ev
            kind = ev["site"]["kind"]
            if kind != "own-raise":
                t["status"] = "no own raise site"
                t["why"] = kind
            else:
                span = raise_span(ev["site"]["file"], ev["site"]["line"])
                if span is None:
                    t["status"] = "no own raise site"
                    t["why"] = "RAISE_VARARGS but no ast.Raise at line"
                else:
                    t["status"] = "target"
                    t["span"] = span
            targets.append(t)
    (RUNS / "targets.json").write_text(json.dumps(targets, indent=1))
    print(json.dumps(stats))
    c = collections.Counter((t["proj"], t["group"], t["status"], t.get("why", "")) for t in targets)
    for k, v in sorted(c.items()):
        print(v, k)
