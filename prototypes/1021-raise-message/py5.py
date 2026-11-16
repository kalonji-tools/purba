"""#1021: how each expected raise or warning in the pinned corpus names more than its type.

Counts `with` ITEMS (not tests) whose call ends in raises/warns/assertRaises*/assertWarns*,
in every `def test*` (module level and in a class). Also counts the one-statement `try`.
"""

import ast
import collections
import pathlib
import sys

RAISES = {"raises", "assertRaises", "assertRaisesRegex", "warns", "assertWarns", "assertWarnsRegex"}
BROAD = {"Exception", "BaseException", "ValueError", "OSError", "IOError", "EnvironmentError", "TypeError", "RuntimeError"}


def tail(n):
    if isinstance(n, ast.Attribute):
        return n.attr
    if isinstance(n, ast.Name):
        return n.id
    return ""


c = collections.Counter()
for path in sorted(pathlib.Path(sys.argv[1]).rglob("test*.py")):
    try:
        tree = ast.parse(path.read_text(encoding="utf-8", errors="replace"))
    except SyntaxError:
        continue
    for fn in ast.walk(tree):
        if not (isinstance(fn, ast.FunctionDef) and fn.name.startswith("test")):
            continue
        for node in ast.walk(fn):
            # call form: raises(E, fn, ...) as a statement
            if isinstance(node, ast.Expr) and isinstance(node.value, ast.Call):
                if tail(node.value.func) in RAISES and len(node.value.args) >= 2:
                    c["call form raises(E, fn, ...)"] += 1
            if isinstance(node, ast.Try) and len(node.body) == 1 and len(node.handlers) == 1:
                h = node.handlers[0]
                if h.type is not None and not node.orelse and not node.finalbody:
                    c["try: one statement / except T:"] += 1
                    if all(isinstance(s, ast.Pass) for s in h.body):
                        c["  ... whose except is only pass"] += 1
            if not isinstance(node, ast.With):
                continue
            for it in node.items:
                e = it.context_expr
                if not (isinstance(e, ast.Call) and tail(e.func) in RAISES):
                    continue
                kind = "warns" if "arn" in tail(e.func) else "raises"
                c[f"{kind}: all"] += 1
                kw = {k.arg: k.value for k in e.keywords}
                msg = kw.get("match", kw.get("msg"))
                regex = tail(e.func).endswith("Regex")
                if msg is not None or regex:
                    c[f"{kind}: names a message (match=/msg=/Regex)"] += 1
                    m = msg if msg is not None else (e.args[1] if len(e.args) > 1 else None)
                    if isinstance(m, ast.Constant) and isinstance(m.value, str):
                        c[f"{kind}:   ... a literal"] += 1
                        if m.value == "":
                            c[f"{kind}:   ... the empty string"] += 1
                    else:
                        c[f"{kind}:   ... not a literal"] += 1
                    continue
                v = it.optional_vars
                if isinstance(v, ast.Name):
                    reads = [
                        n.attr
                        for n in ast.walk(fn)
                        if isinstance(n, ast.Attribute)
                        and isinstance(n.value, ast.Name)
                        and n.value.id == v.id
                    ]
                    if "value" in reads or "exception" in reads:
                        c[f"{kind}: no message arg, reads .value/.exception"] += 1
                    elif reads:
                        c[f"{kind}: no message arg, reads another attr ({sorted(set(reads))[0]})"] += 1
                    else:
                        c[f"{kind}: binds as, reads nothing"] += 1
                else:
                    c[f"{kind}: type alone"] += 1
                    if e.args and tail(e.args[0]) in BROAD:
                        c[f"{kind}:   ... of a broad type"] += 1
for k in sorted(c):
    print(f"{c[k]:6}  {k}")
