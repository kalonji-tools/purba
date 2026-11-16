"""Enumerate expected-raise `with` items using py5b.py's unit and grouping logic.

Unit (as py5b): one `with` item whose context expr is a call to raises / assertRaises /
assertRaisesRegex with a positional arg, inside a `def test*` of a file named test*.py.
Groups (py5b): match=/msg=/*Regex -> "names a message"; else `as var` -> "reads caught";
else "type alone". Refinement required by the task: an `as var` that is never loaded in
the test function is moved to "type alone" (py5b's "reads caught" counts any `as`).
Writes mut/runs/items.json.
"""

import ast
import json
import pathlib
import sys

RAISES = {"raises", "assertRaises", "assertRaisesRegex"}
SRC = pathlib.Path(sys.argv[1])  # mut/src
OUT = pathlib.Path(sys.argv[2])


def tail(n):
    if isinstance(n, ast.Attribute):
        return n.attr
    if isinstance(n, ast.Name):
        return n.id
    if isinstance(n, ast.Call):
        return tail(n.func)
    return ""


def loads(fn, var):
    return any(isinstance(n, ast.Name) and n.id == var and isinstance(n.ctx, ast.Load) for n in ast.walk(fn))


items = []
for proj in sorted(p for p in SRC.iterdir() if p.is_dir()):
    for path in sorted(proj.rglob("test*.py")):
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        seen = set()
        for fn in ast.walk(tree):
            if not (isinstance(fn, ast.FunctionDef) and fn.name.startswith("test")):
                continue
            for node in ast.walk(fn):
                if not isinstance(node, ast.With):
                    continue
                for idx, it in enumerate(node.items):
                    e = it.context_expr
                    if not (isinstance(e, ast.Call) and tail(e.func) in RAISES) or not e.args:
                        continue
                    key = (node.lineno, idx)
                    if key in seen:  # nested test defs: py5b would count twice; keep one
                        continue
                    seen.add(key)
                    kw = {k.arg for k in e.keywords}
                    named = "match" in kw or "msg" in kw or tail(e.func).endswith("Regex")
                    var = it.optional_vars.id if isinstance(it.optional_vars, ast.Name) else None
                    etype = e.args[0]
                    types = [tail(x) for x in etype.elts] if isinstance(etype, ast.Tuple) else [tail(etype)]
                    if named:
                        group = "names a message"
                    elif var and loads(fn, var):
                        group = "reads caught"
                    else:
                        group = "type alone"
                    py5b_group = "names a message" if named else ("reads caught" if var else "type alone")
                    items.append(
                        {
                            "proj": proj.name,
                            "file": str(path.resolve()),
                            "rel": str(path.relative_to(proj)),
                            "fn": fn.name,
                            "with_line": node.lineno,
                            "ctx_line": e.lineno,
                            "idx": idx,
                            "n_items": len(node.items),
                            "types": types,
                            "api": tail(e.func),
                            "group": group,
                            "py5b_group": py5b_group,
                            "var": var,
                            "match_src": ast.unparse(e),
                        }
                    )
OUT.write_text(json.dumps(items, indent=1))
import collections

c = collections.Counter((i["proj"], i["group"]) for i in items)
for k, v in sorted(c.items()):
    print(k, v)
print("moved reads caught -> type alone (unread as-name):", sum(1 for i in items if i["group"] != i["py5b_group"]))
