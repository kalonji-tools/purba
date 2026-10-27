"""PROTOTYPE (#689): join what the text says (classify.py) with what happened (leakwatch.py).

usage: report.py STATIC.json DYNAMIC.json [--each]
A function leaks when any of its items (parametrized cases) leaks.
"""

import json, sys, collections

static = json.load(open(sys.argv[1]))["tests"]
dynamic = json.load(open(sys.argv[2]))
each = "--each" in sys.argv

leaks = collections.defaultdict(set)
ran = set()
for item in dynamic["items"]:
    ran.add(item["func"])
    leaks[item["func"]] |= set(item["leaks"])

funcs = [f for f in static if f in ran]
print(f"functions: {len(funcs)} judged and run · items run: {len(dynamic['items'])}")
print(
    f"snapshot cost: {1000 * dynamic['snapshot_seconds'] / max(dynamic['snapshots'], 1):.2f} ms"
    " per snapshot, 2 per test"
)
print()
print(
    f"{'level':14s} {'says isolated':>14s} {'of those leak':>14s}"
    f" {'says not':>9s} {'of those clean':>15s}"
)
for level in ("L0_signature", "L1_body", "L2_calls"):
    yes = [f for f in funcs if static[f][level]]
    no = [f for f in funcs if not static[f][level]]
    print(
        f"{level:14s} {len(yes):14d} {sum(bool(leaks[f]) for f in yes):14d}"
        f" {len(no):9d} {sum(not leaks[f] for f in no):15d}"
    )
print()
kinds = collections.Counter(l.split(" [")[0] for f in funcs for l in leaks[f])
print("what leaked (functions):", kinds.most_common(20))
if each:
    print()
    for f in funcs:
        s = static[f]
        print(
            f"{f:48s} L0={s['L0_signature']!s:5} L1={s['L1_body']!s:5}"
            f" L2={s['L2_calls']!s:5} leaks={sorted(leaks[f])}"
        )
else:
    print()
    for f in funcs:
        if leaks[f]:
            print(f"  LEAKS  {f:58s} L1={static[f]['L1_body']!s:5} {sorted(leaks[f])[:4]}")
