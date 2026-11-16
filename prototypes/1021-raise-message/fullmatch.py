"""#1021 Q2: of the match= patterns in the corpus, how many match the WHOLE message the
baseline run caught, and how many only a part of it (re.search but not re.fullmatch)?

Joins mut/runs/items.json (match_src) with mut/runs/baseline/*.jsonl (exc_str at with_line).
Unit = one raise event caught at a `with` that names a message by a string literal.
"""

import ast
import collections
import json
import pathlib
import re
import sys

runs = pathlib.Path(sys.argv[1])
items = json.load(open(runs / "items.json"))
pat = {}
for it in items:
    if it["group"] != "names a message" or not it.get("match_src"):
        continue
    try:
        call = ast.parse(it["match_src"].strip(), mode="eval").body
    except SyntaxError:
        continue
    kw = {k.arg: k.value for k in getattr(call, "keywords", [])}
    m = kw.get("match", kw.get("msg"))
    if m is None and it["match_src"].split("(")[0].endswith("Regex") and len(call.args) > 1:
        m = call.args[1]
    if isinstance(m, ast.Constant) and isinstance(m.value, str):
        pat[(it["proj"], it["rel"], it["with_line"])] = m.value

c = collections.Counter()
seen = set()
ex = collections.defaultdict(list)
for f in sorted((runs / "baseline").glob("*.jsonl")):
    proj = f.stem
    for line in f.open():
        e = json.loads(line)
        if e.get("ev") != "raise" or "exc_str" not in e:
            continue
        try:
            rel = str(pathlib.Path(e["with_file"]).relative_to(runs.parent / "src" / proj))
        except ValueError:
            c["skipped: with_file outside the project (unittest context)"] += 1
            continue
        key = (proj, rel, e["with_line"])
        if key not in pat:
            continue
        p, s = pat[key], e["exc_str"]
        kind = (
            "whole message (fullmatch)"
            if re.fullmatch(p, s, re.S)
            else "part of the message (search only)"
            if re.search(p, s)
            else "no match?"
        )
        c[kind] += 1
        if key not in seen:
            seen.add(key)
            c[f"distinct with items: {kind}"] += 1
            if len(ex[kind]) < 3:
                ex[kind].append(f"{proj}:{rel}:{e['with_line']}  match={p!r}  msg={s[:70]!r}")
for k in sorted(c):
    print(f"{c[k]:5}  {k}")
for k, v in ex.items():
    print(k)
    for x in v:
        print("   ", x)
