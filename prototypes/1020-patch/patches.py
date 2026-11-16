"""Count patch forms per test function in a corpus. Matchers are named per row."""

import ast
import sys
from collections import Counter, defaultdict
from pathlib import Path

root = Path(sys.argv[1])
rows = Counter()
per_proj = defaultdict(Counter)
examples = defaultdict(list)


def dotted(node):
    parts = []
    while isinstance(node, ast.Attribute):
        parts.append(node.attr)
        node = node.value
    if isinstance(node, ast.Name):
        parts.append(node.id)
        return ".".join(reversed(parts))
    return None


def imported_modules(tree):
    names = set()
    for n in ast.walk(tree):
        if isinstance(n, ast.Import):
            for a in n.names:
                names.add((a.asname or a.name).split(".")[0])
        elif isinstance(n, ast.ImportFrom):
            for a in n.names:
                names.add(a.asname or a.name)
    return names


PATCH_CALL = {
    "patch",
    "patch.object",
    "patch.dict",
    "patch.multiple",
    "mock.patch",
    "mock.patch.object",
    "mock.patch.dict",
    "mock.patch.multiple",
    "unittest.mock.patch",
    "unittest.mock.patch.object",
    "unittest.mock.patch.dict",
}

for f in root.rglob("*.py"):
    rel = f.relative_to(root)
    proj = rel.parts[0]
    if not (f.name.startswith("test") or "tests" in rel.parts or "test" in rel.parts):
        continue
    if ".git" in rel.parts:
        continue
    try:
        tree = ast.parse(f.read_text(encoding="utf-8", errors="replace"))
    except SyntaxError:
        continue
    mods = imported_modules(tree)
    for fn in ast.walk(tree):
        if not isinstance(
            fn, (ast.FunctionDef, ast.AsyncFunctionDef)
        ) or not fn.name.startswith("test"):
            continue
        hits = set()
        params = {a.arg for a in fn.args.args + fn.args.kwonlyargs}
        if "monkeypatch" in params:
            hits.add("param monkeypatch (pytest)")
        if "mocker" in params:
            hits.add("param mocker (pytest-mock)")
        for d in fn.decorator_list:
            name = dotted(d.func if isinstance(d, ast.Call) else d) or ""
            if name in PATCH_CALL:
                hits.add("decorator mock.patch*")
            if name.endswith("freeze_time"):
                hits.add("decorator freeze_time")
        for n in ast.walk(fn):
            if isinstance(n, (ast.With, ast.AsyncWith)):
                for item in n.items:
                    e = item.context_expr
                    name = dotted(e.func) if isinstance(e, ast.Call) else None
                    if name in PATCH_CALL:
                        hits.add("with mock.patch*")
                    elif name and name.endswith("freeze_time"):
                        hits.add("with freeze_time")
            if isinstance(n, ast.Call):
                name = dotted(n.func) or ""
                if name in PATCH_CALL and not any(
                    n is w.context_expr
                    for w in ast.walk(fn)
                    if isinstance(w, ast.withitem)
                ):
                    hits.add("call mock.patch* (start/stop)")
                if (
                    name == "setattr"
                    and n.args
                    and isinstance(n.args[0], ast.Name)
                    and n.args[0].id in mods
                ):
                    hits.add("setattr(imported_module, ...)")
            if isinstance(n, (ast.Assign, ast.AugAssign, ast.Delete)):
                targets = (
                    n.targets if isinstance(n, (ast.Assign, ast.Delete)) else [n.target]
                )
                for t in targets:
                    if isinstance(t, ast.Attribute):
                        base = t
                        while isinstance(base, ast.Attribute):
                            base = base.value
                        if (
                            isinstance(base, ast.Name)
                            and base.id in mods
                            and base.id not in params
                        ):
                            hits.add("assign imported_module.attr = ...")
        rows["test functions"] += 1
        per_proj[proj]["test functions"] += 1
        if hits:
            rows["any patch form"] += 1
            per_proj[proj]["any patch form"] += 1
        for h in hits:
            rows[h] += 1
            per_proj[proj][h] += 1
            if len(examples[h]) < 3:
                examples[h].append(f"{rel}:{fn.lineno} {fn.name}")

for k, v in rows.most_common():
    print(f"{v:6d}  {k}")
print()
for p in sorted(per_proj):
    c = per_proj[p]
    print(
        p,
        c["test functions"],
        "tests,",
        c["any patch form"],
        "with a patch form:",
        {k: v for k, v in c.items() if k not in ("test functions", "any patch form")},
    )
print()
for k, ex in examples.items():
    print(k)
    for e in ex:
        print("   ", e)
