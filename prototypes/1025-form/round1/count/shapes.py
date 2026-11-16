"""Count shapes of an iterable check over the pinned corpus (PROTOTYPE, #1025)."""
import ast, sys, pathlib, collections
root = pathlib.Path(sys.argv[1])
c = collections.Counter(); ex = collections.defaultdict(list)
def tests(tree):
    for n in ast.walk(tree):
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name.startswith("test"):
            yield n
def is_cmp(e): return isinstance(e, ast.Compare) and len(e.ops) == 1
for f in sorted(root.rglob("test*.py")):
    try: tree = ast.parse(f.read_text(encoding="utf-8", errors="replace"))
    except SyntaxError: continue
    for t in tests(tree):
        c["tests"] += 1
        body = t.body
        # trailing loop: last stmt is a for whose body holds only asserts
        last = body[-1]
        if isinstance(last, ast.For):
            inner = last.body
            if all(isinstance(s, ast.Assert) for s in inner):
                c["trailing for, asserts only"] += 1
                if len(inner) == 1:
                    c["  of them one assert"] += 1
                    if is_cmp(inner[0].test): c["  of them one comparison"] += 1
                    ex["trailing for one assert"].append(f"{f.relative_to(root)}:{last.lineno} {ast.unparse(last)[:90]!r}")
        for n in ast.walk(t):
            if isinstance(n, ast.For) and any(isinstance(s, ast.Assert) for s in ast.walk(n)):
                c["for holding an assert (anywhere)"] += 1
            if isinstance(n, ast.Assert):
                e = n.test
                if isinstance(e, ast.Call) and isinstance(e.func, ast.Name) and e.func.id in ("all", "any"):
                    c[f"assert {e.func.id}(...)"] += 1
                    g = e.args[0] if e.args else None
                    if isinstance(g, (ast.GeneratorExp, ast.ListComp)):
                        gen = g.generators
                        k = "one gen, no if" if len(gen) == 1 and not gen[0].ifs else "if or nested"
                        c[f"  {e.func.id} {k}"] += 1
                        if is_cmp(g.elt): c[f"  {e.func.id} body one comparison"] += 1
                        ex[f"{e.func.id}"].append(f"{f.relative_to(root)}:{n.lineno} {ast.unparse(e)[:90]!r}")
                    else:
                        c[f"  {e.func.id} of a non-generator"] += 1
                for m in ast.walk(e):
                    if isinstance(m, ast.Call) and isinstance(m.func, ast.Name) and m.func.id in ("list", "tuple", "set", "sorted") and m.args and isinstance(m.args[0], ast.Call):
                        c["assert reads list/tuple/set/sorted(<call>)"] += 1; break
                for m in ast.walk(e):
                    if isinstance(m, ast.Call) and isinstance(m.func, ast.Name) and m.func.id == "next":
                        c["assert reads next(...)"] += 1; break
for k, v in c.items(): print(f"{v:6}  {k}")
for k, v in ex.items():
    print(f"\n# {k}: {len(v)}"); print("\n".join(v[:12]))
