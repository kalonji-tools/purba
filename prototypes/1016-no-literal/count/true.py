# Counts each assert in the test files of a corpus that compares a value to True
# by == or is, with the operator, the file and the line.
import ast
import pathlib
import sys

corpus = pathlib.Path(sys.argv[1])
hits = []
for p in sorted(corpus.rglob("*.py")):
    if not (
        p.name.startswith("test") or "/tests/" in str(p) or p.name.endswith("_test.py")
    ):
        continue
    try:
        t = ast.parse(p.read_text(encoding="utf-8", errors="replace"))
    except SyntaxError:
        continue
    for n in ast.walk(t):
        if (
            isinstance(n, ast.Assert)
            and isinstance(n.test, ast.Compare)
            and len(n.test.ops) == 1
        ):
            r = n.test.comparators[0]
            l = n.test.left
            for side in (l, r):
                if (
                    isinstance(side, ast.Constant)
                    and side.value is True
                    and isinstance(n.test.ops[0], (ast.Eq, ast.Is))
                ):
                    hits.append(
                        f"{type(n.test.ops[0]).__name__} {p.relative_to(corpus)}:{n.lineno}: {ast.unparse(n)}"
                    )
print(len(hits))
print("\n".join(hits))
