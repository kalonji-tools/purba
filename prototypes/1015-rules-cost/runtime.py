"""PROTOTYPE for #1015, never merge. An `ast` UPPER BOUND on the rules that
execution applies while a case runs, over the tests that main's selection saw
(tally.tsv). Execution refuses a branch only when it went one way, and a loop
only when it ran no round, so a count here is at most the true count.
Reads files with `ast` only; imports and runs nothing."""

import ast
import collections
import sys

tally, root = sys.argv[1], sys.argv[2]
RAISES = ("raises", "assertRaises", "assertRaisesRegex", "warns", "deprecated_call")


def is_raise_check(node: ast.With) -> bool:
    return any(
        isinstance(i.context_expr, ast.Call)
        and ast.unparse(i.context_expr.func).split(".")[-1] in RAISES
        for i in node.items
    )


def features(fn: ast.FunctionDef) -> set[str]:
    body = fn.body
    if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant):
        body = body[1:]
    # the trailing run of asserts is the expectations; the rest is the act
    end = len(body)
    while end > 0 and isinstance(body[end - 1], ast.Assert):
        end -= 1
    act, checks = body[:end], body[end:]
    found = set()
    if not checks:
        found.add("ends with no assert")
    for stmt in act:
        if isinstance(stmt, ast.With) and is_raise_check(stmt):
            found.add("raise check: with raises(...)")
        for node in ast.walk(stmt):
            if isinstance(node, ast.Try) and node.handlers:
                found.add("second-path: try/except in the act")
            elif isinstance(node, (ast.If, ast.Match)):
                found.add("second-path?: if/match in the act")
            elif isinstance(node, (ast.IfExp, ast.BoolOp)) or (
                isinstance(node, ast.comprehension) and node.ifs
            ):
                found.add("second-path?: or/and/x-if-y/comprehension-if")
            elif isinstance(node, (ast.For, ast.While)):
                found.add("loop in the act (refused only with no round)")
            elif isinstance(node, ast.Assert):
                found.add("assert-before-expectation")
            elif isinstance(node, ast.With) and is_raise_check(node):
                found.add("raise check: with raises(...)")
    return found


counts = collections.Counter()
examples = {}
seen = 0
for row in open(tally, encoding="utf-8"):
    project, path, function, verdict, _ = row.rstrip("\n").split("\t")
    if verdict == "parse-error":
        continue
    tree = ast.parse(open(f"{root}/{path}", encoding="utf-8").read())
    defs = [n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == function]
    if not defs:
        continue
    seen += 1
    for f in features(defs[-1]):
        counts[f] += 1
        examples.setdefault(f, f"{path}::{function}")
print(f"tests read: {seen}")
for f, n in counts.most_common():
    print(f"{n:5d}  {f}   e.g. {examples[f]}")
