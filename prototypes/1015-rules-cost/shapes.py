"""PROTOTYPE for #1015, never merge. Classify each `line N is not one comparison`
assert, and each private name, that main's selection printed (tally.tsv).
Reads files with `ast` only; imports and runs nothing."""

import ast
import collections
import re
import sys

tally, root = sys.argv[1], sys.argv[2]
shape = collections.Counter()
private = collections.Counter()
ex = {}
for row in open(tally, encoding="utf-8"):
    project, path, function, verdict, reasons = row.rstrip("\n").split("\t")
    if verdict != "refused":
        continue
    for name in re.findall(r"reads the private name (\S+),", reasons):
        kind = (
            "dunder"
            if name.startswith("__") and name.endswith("__")
            else ("_name of the library" if name.startswith("_") else "attr ._x")
        )
        private[name] += 1
    lines = [int(n) for n in re.findall(r"line (\d+) is not one comparison", reasons)]
    if not lines:
        continue
    tree = ast.parse(open(f"{root}/{path}", encoding="utf-8").read())
    asserts = {n.lineno: n for n in ast.walk(tree) if isinstance(n, ast.Assert)}
    for line in lines:
        t = asserts[line].test
        if isinstance(t, ast.Compare) and len(t.ops) > 1:
            k = "chained a < b < c"
        elif isinstance(t, ast.Call):
            k = f"call {ast.unparse(t.func).split('.')[-1]}(...)"
            if k not in (
                "call isinstance(...)",
                "call callable(...)",
                "call hasattr(...)",
                "call issubclass(...)",
                "call all(...)",
                "call any(...)",
            ):
                k = "call other(...)"
        elif isinstance(t, ast.UnaryOp) and isinstance(t.op, ast.Not):
            k = "not x"
        elif isinstance(t, ast.BoolOp):
            k = "x and/or y"
        elif isinstance(t, (ast.Name, ast.Attribute, ast.Subscript)):
            k = "bare truthy x"
        elif isinstance(t, ast.Tuple):
            k = "tuple assert"
        else:
            k = type(t).__name__
        shape[k] += 1
        ex.setdefault(k, f"{path}:{line}  {ast.unparse(asserts[line])[:70]}")
print("not one comparison:")
for k, n in shape.most_common():
    print(f"{n:5d}  {k:22s} e.g. {ex[k]}")
print("\nprivate names, top 12 of", sum(private.values()), "reads,", len(private), "names:")
for k, n in private.most_common(12):
    print(f"{n:5d}  {k}")
