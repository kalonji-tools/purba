"""PROTOTYPE for #1015, never merge. For each `computes NAME` refusal that main's
selection printed (tally.tsv), say where NAME comes from in the test: a parameter
(an input), a local bound in the test's body, or a name of the module.
Reads files with `ast` only; imports and runs nothing."""

import ast
import collections
import re
import sys

tally, root = sys.argv[1], sys.argv[2]
counts = collections.Counter()
examples = collections.defaultdict(list)
for row in open(tally, encoding="utf-8"):
    project, path, function, verdict, reasons = row.rstrip("\n").split("\t")
    if verdict != "refused":
        continue
    names = [m for m in re.findall(r"computes (\S+), and an expected", reasons)]
    names = [n for n in names if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", n)]
    if not names:
        continue
    tree = ast.parse(open(f"{root}/{path}", encoding="utf-8").read())
    defs = [n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == function]
    if not defs:
        continue
    fn = defs[-1]
    params = {a.arg for a in fn.args.args + fn.args.kwonlyargs}
    decorated = any("parametrize" in ast.unparse(d) for d in fn.decorator_list)
    local = {
        t.id
        for n in ast.walk(fn)
        for t in ast.walk(n)
        if isinstance(t, ast.Name) and isinstance(t.ctx, ast.Store)
    }
    for name in names:
        if name in params:
            kind = "input, from parametrize" if decorated else "input, from a fixture"
        elif name in local:
            kind = "local of the test"
        else:
            kind = "name of the module"
        counts[kind] += 1
        examples[kind].append(f"{path}::{function} {name}")
for kind, n in counts.most_common():
    print(f"{n:5d}  {kind}   e.g. {examples[kind][0]}")
