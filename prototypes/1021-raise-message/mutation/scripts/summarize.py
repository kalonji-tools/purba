"""Tables from targets.json + results.jsonl. Writes runs/summary.txt and runs/survivors.json."""

import collections
import json
import re
import sys

sys.path.insert(0, "/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/e9de2c2b-a74e-4d6e-b5d0-498e964f8ad4/scratchpad/mut/scripts")
from common import RUNS, SRC  # noqa

targets = {t["key"]: t for t in json.loads((RUNS / "targets.json").read_text())}
results = {}
for line in (RUNS / "results.jsonl").read_text().splitlines():
    r = json.loads(line)
    results[r["key"]] = r  # last run wins

GROUPS = ["type alone", "names a message", "reads caught"]
out = []
P = lambda *a: out.append(" ".join(str(x) for x in a))  # noqa


def short(path):
    return str(path).replace(str(SRC) + "/", "")


rows = collections.defaultdict(collections.Counter)
for t in targets.values():
    c = rows[(t["group"], t["proj"])]
    c["static"] += 1
    if t["status"] == "not kept":
        continue
    c["kept"] += 1
    if t["status"] == "no own raise site":
        c["no own raise"] += 1
        continue
    r = results.get(t["key"])
    if r is None:
        c["not run"] += 1
        continue
    c["run"] += 1
    c[r["result"]] += 1

cols = ["static", "kept", "no own raise", "run", "survived", "killed", "error", "not run"]
P(f"{'group':16} {'project':9} " + " ".join(f"{c:>12}" for c in cols))
for g in GROUPS:
    tot = collections.Counter()
    for (gg, p), c in sorted(rows.items()):
        if gg != g:
            continue
        tot += c
        P(f"{g:16} {p:9} " + " ".join(f"{c[k]:>12}" for k in cols))
    P(f"{g:16} {'TOTAL':9} " + " ".join(f"{tot[k]:>12}" for k in cols))
    P("")

surv = []
for k, r in results.items():
    if r["result"] != "survived":
        continue
    t = targets[k]
    o = t["orig"]
    d = r.get("first_diff")
    m = d["mutant"] if d else None
    rec = {
        "group": t["group"],
        "proj": t["proj"],
        "nodeid": t["nodeid"],
        "with": f"{t['rel']}:{t['with_line']}",
        "with_src": t["match_src"],
        "orig_type": o["exc_type"],
        "orig_msg": o["exc_str"],
        "orig_site": f"{short(o['site']['file'])}:{o['site']['line']}",
        "n_execs_in_nodeid": t["n_execs_in_nodeid"],
        "diff_index": d["index"] if d else None,
        "new_type": m and m["exc_type"],
        "new_msg": m and m["exc_str"],
        "new_site": m and f"{short(m['site'].get('file'))}:{m['site'].get('line')} ({m['site'].get('kind')})",
        "same_type": bool(m) and m["exc_type"] == o["exc_type"],
    }
    if m is not None and t["group"] == "type alone":
        # would `match=re.escape(original str)` have rejected the new exception?
        rec["match_orig_would_kill"] = re.search(re.escape(o["exc_str"]), m["exc_str"] or "") is None
    if d is None:
        rec["note"] = "mutant events identical to baseline for this item"
    surv.append(rec)

(RUNS / "survivors.json").write_text(json.dumps(surv, indent=1))
P("survivors by group:", collections.Counter(s["group"] for s in surv))
ta = [s for s in surv if s["group"] == "type alone"]
P("type-alone survivors:", len(ta),
  " match=re.escape(orig) would kill:", sum(1 for s in ta if s.get("match_orig_would_kill") is True),
  " would not:", sum(1 for s in ta if s.get("match_orig_would_kill") is False),
  " undetermined:", sum(1 for s in ta if "match_orig_would_kill" not in s))
P("type-alone survivors with an empty original message:", sum(1 for s in ta if s["orig_msg"] == ""))
P("survivors where the new exception has the same type:", sum(1 for s in surv if s["same_type"]), "/", len(surv))
P("survivors whose new exception came from: ", collections.Counter((s["new_site"] or "").rsplit("(", 1)[-1] for s in surv))
(RUNS / "summary.txt").write_text("\n".join(out) + "\n")
print("\n".join(out))
