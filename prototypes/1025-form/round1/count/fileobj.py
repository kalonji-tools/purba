"""PROTOTYPE #1025: tests whose assert reads a name the test bound to StringIO/BytesIO/open/iter/map/zip/filter/a call of a generator-ish builtin."""
import ast, sys, pathlib, collections
root = pathlib.Path(sys.argv[1]); c = collections.Counter(); ex = collections.defaultdict(list)
KINDS = {"StringIO": "file-like", "BytesIO": "file-like", "open": "file-like", "TemporaryFile": "file-like",
         "iter": "iterator", "map": "iterator", "zip": "iterator", "filter": "iterator", "reversed": "iterator", "enumerate": "iterator"}
def callee(v):
    if isinstance(v, ast.Call):
        f = v.func
        return f.id if isinstance(f, ast.Name) else f.attr if isinstance(f, ast.Attribute) else None
for f in sorted(root.rglob("test*.py")):
    try: tree = ast.parse(f.read_text(encoding="utf-8", errors="replace"))
    except SyntaxError: continue
    for t in ast.walk(tree):
        if not (isinstance(t, ast.FunctionDef) and t.name.startswith("test")): continue
        c["tests"] += 1
        bound = {}
        for n in ast.walk(t):
            if isinstance(n, ast.Assign) and len(n.targets) == 1 and isinstance(n.targets[0], ast.Name):
                k = KINDS.get(callee(n.value))
                if k: bound[n.targets[0].id] = k
            if isinstance(n, ast.withitem) and isinstance(n.optional_vars, ast.Name):
                k = KINDS.get(callee(n.context_expr))
                if k: bound[n.optional_vars.id] = k
        hit = set()
        for n in ast.walk(t):
            if isinstance(n, ast.Assert):
                for m in ast.walk(n.test):
                    if isinstance(m, ast.Name) and m.id in bound:
                        hit.add(bound[m.id]); ex[bound[m.id]].append(f"{f.relative_to(root)}:{n.lineno} {ast.unparse(n.test)[:80]}")
        for k in hit: c[f"tests whose assert reads a {k} name"] += 1
for k, v in c.items(): print(f"{v:6}  {k}")
for k, v in ex.items(): print(f"\n# {k}: {len(v)} asserts"); print("\n".join(v[:8]))
