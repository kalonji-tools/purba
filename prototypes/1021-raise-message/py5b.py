"""#1021, numbers for "a raise always names its message".

A. Of the expected raises that read caught.value with no match=, what does the check read:
   the message (str(), .args, .msg, .message, .match()) or something else?
B. For each expected raise, how many `raise <Type>` sites of that exact type name sit in the
   project's own non-test code? More sites = more raises that a type alone cannot tell apart.
   Lower bound: subclasses, implicit raises (a bad call raising TypeError) and other packages'
   raises are not counted.

Unit = one `with` item in a `def test*` of a file named test*.py.
"""

import ast
import collections
import pathlib
import statistics
import sys

RAISES = {"raises", "assertRaises", "assertRaisesRegex"}
MSG_ATTRS = {"args", "msg", "message", "strerror"}


def tail(n):
    if isinstance(n, ast.Attribute):
        return n.attr
    if isinstance(n, ast.Name):
        return n.id
    if isinstance(n, ast.Call):
        return tail(n.func)
    return ""


def raise_sites(root: pathlib.Path) -> collections.Counter:
    sites = collections.Counter()
    for p in root.rglob("*.py"):
        parts = {s.lower() for s in p.relative_to(root).parts}
        if any("test" in s for s in parts) or "docs" in parts:
            continue
        try:
            tree = ast.parse(p.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        for n in ast.walk(tree):
            if isinstance(n, ast.Raise) and n.exc is not None:
                sites[tail(n.exc)] += 1
    return sites


def value_read_kind(fn, var):
    kinds = set()
    for n in ast.walk(fn):
        # var.match(...) on pytest's ExceptionInfo
        if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute):
            f = n.func
            if f.attr == "match" and isinstance(f.value, ast.Name) and f.value.id == var:
                kinds.add("message")
        if isinstance(n, ast.Call) and tail(n.func) in {"str", "repr"} and n.args:
            a = n.args[0]
            if (
                isinstance(a, ast.Attribute)
                and a.attr in {"value", "exception"}
                and isinstance(a.value, ast.Name)
                and a.value.id == var
            ):
                kinds.add("message")
        if isinstance(n, ast.Attribute) and isinstance(n.value, ast.Attribute):
            inner = n.value
            if (
                inner.attr in {"value", "exception"}
                and isinstance(inner.value, ast.Name)
                and inner.value.id == var
            ):
                kinds.add("message" if n.attr in MSG_ATTRS else f"attr .{n.attr}")
    return kinds or {"value itself (is / isinstance / ==)"}


corpus = pathlib.Path(sys.argv[1])
A = collections.Counter()
B = collections.defaultdict(list)
examples = collections.defaultdict(list)
for proj in sorted(p for p in corpus.iterdir() if p.is_dir()):
    sites = raise_sites(proj)
    for path in sorted(proj.rglob("test*.py")):
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        for fn in ast.walk(tree):
            if not (isinstance(fn, ast.FunctionDef) and fn.name.startswith("test")):
                continue
            for node in ast.walk(fn):
                if not isinstance(node, ast.With):
                    continue
                for it in node.items:
                    e = it.context_expr
                    if not (isinstance(e, ast.Call) and tail(e.func) in RAISES) or not e.args:
                        continue
                    kw = {k.arg for k in e.keywords}
                    named = "match" in kw or "msg" in kw or tail(e.func).endswith("Regex")
                    var = it.optional_vars.id if isinstance(it.optional_vars, ast.Name) else None
                    etype = e.args[0]
                    types = [tail(x) for x in etype.elts] if isinstance(etype, ast.Tuple) else [tail(etype)]
                    n = sum(sites[t] for t in types)
                    if named:
                        group = "names a message"
                    elif var:
                        group = "reads caught"
                        for k in value_read_kind(fn, var):
                            A[k] += 1
                            if len(examples[k]) < 2:
                                examples[k].append(f"{path.relative_to(corpus)}::{fn.name}")
                    else:
                        group = "type alone"
                    B[group].append((proj.name, "/".join(types), n))

print("A. checks that read caught.value with no match= (a test may count in two kinds)")
for k, v in A.most_common():
    print(f"  {v:5}  {k}    e.g. {', '.join(examples[k])}")
print()
print("B. raise sites of the expected type in the project's own code, per expected raise")
for g, rows in B.items():
    ns = [n for _, _, n in rows]
    zero = sum(1 for x in ns if x == 0)
    one = sum(1 for x in ns if x == 1)
    many = sum(1 for x in ns if x >= 2)
    ten = sum(1 for x in ns if x >= 10)
    print(
        f"  {g:16} items {len(ns):4}  sites=0 {zero:4}  =1 {one:4}  >=2 {many:4}  >=10 {ten:4}"
        f"  median {statistics.median(ns):5}"
    )
print()
print("B'. type alone, by project and type (items, sites in the project)")
c = collections.Counter((p, t, n) for p, t, n in B["type alone"])
for (p, t, n), k in sorted(c.items(), key=lambda x: -x[1])[:25]:
    print(f"  {k:4}  {p:9} {t:28} sites {n}")
