"""Count assert shapes in a pinned corpus (#1016 support). Reads with `ast` only.
File selection and the test-function definition are copied from
p1015/prototypes/1015-rules-cost/practices.py: files named test_*.py or *_test.py
outside dot-directories; a test is a `def test*` at module level or directly in a
module-level class. kind = "fn" (module level) or "meth" (in a class)."""

import ast
import collections
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
outdir = pathlib.Path(sys.argv[2])

C = collections.defaultdict(collections.Counter)  # suite -> key -> n
EX = collections.defaultdict(list)  # category -> [(suite, file:line, src)]


def tail(node):
    try:
        s = ast.unparse(node)
    except Exception:
        return ""
    return s.split("(")[0].split(".")[-1]


def bump(suite, kind, key, n=1):
    C[suite][f"{key}|{kind}"] += n


def example(cat, suite, path, node, lines):
    src = lines[node.lineno - 1].strip()
    EX[cat].append((suite, f"{path}:{node.lineno}", src))


# ---------- name scopes ----------
def parametrize_names(fn):
    out = set()
    for d in fn.decorator_list:
        if isinstance(d, ast.Call) and tail(d.func) == "parametrize" and d.args:
            a = d.args[0]
            if isinstance(a, ast.Constant) and isinstance(a.value, str):
                out |= {s.strip() for s in a.value.split(",") if s.strip()}
            elif isinstance(a, (ast.List, ast.Tuple)):
                out |= {
                    e.value
                    for e in a.elts
                    if isinstance(e, ast.Constant) and isinstance(e.value, str)
                }
    return out


def fn_params(fn):
    a = fn.args
    names = {x.arg for x in a.posonlyargs + a.args + a.kwonlyargs}
    if a.vararg:
        names.add(a.vararg.arg)
    if a.kwarg:
        names.add(a.kwarg.arg)
    return names | parametrize_names(fn)


def fn_locals(fn):
    out = set()
    for n in ast.walk(fn):
        if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Store):
            out.add(n.id)
        elif isinstance(n, (ast.Import, ast.ImportFrom)):
            out |= {(al.asname or al.name).split(".")[0] for al in n.names}
        elif isinstance(n, ast.ExceptHandler) and n.name:
            out.add(n.name)
    return out


def module_names(tree):
    out = set()

    def visit(stmts):
        for s in stmts:
            if isinstance(s, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                out.add(s.name)
            elif isinstance(s, (ast.Import, ast.ImportFrom)):
                out.update((al.asname or al.name).split(".")[0] for al in s.names)
            else:
                for n in ast.walk(s):
                    if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Store):
                        out.add(n.id)
                for field in ("body", "orelse", "finalbody"):
                    visit(getattr(s, field, []) or [])
                for h in getattr(s, "handlers", []) or []:
                    visit(h.body)

    visit(tree.body)
    return out


def imports_hypothesis(tree):
    def visit(stmts):
        for s in stmts:
            if isinstance(s, ast.Import) and any(
                al.name.split(".")[0] == "hypothesis" for al in s.names
            ):
                return True
            if (
                isinstance(s, ast.ImportFrom)
                and (s.module or "").split(".")[0] == "hypothesis"
            ):
                return True
            if isinstance(s, (ast.If, ast.Try)):
                subs = list(s.body) + list(s.orelse) + list(getattr(s, "finalbody", []))
                for h in getattr(s, "handlers", []):
                    subs += h.body
                if visit(subs):
                    return True
        return False

    return visit(tree.body)


# ---------- shape matchers ----------
WRAPPERS = {
    "list",
    "tuple",
    "set",
    "frozenset",
    "dict",
    "sorted",
    "str",
    "repr",
    "bytes",
    "int",
    "float",
    "bool",
    "len",
    "type",
    "iter",
    "next",
    "reversed",
}


def roundtrip_depth(name, side, skip_wrappers=False):
    """Max number of enclosing Calls (between `side` root and an occurrence of
    Name `name`) whose *arguments* contain that occurrence. 0 = not an argument."""
    best = 0

    def walk(node, depth):
        nonlocal best
        if isinstance(node, ast.Name) and node.id == name:
            best = max(best, depth)
            return
        if isinstance(node, ast.Call):
            if isinstance(node.func, ast.Attribute):
                walk(
                    node.func.value, depth + 1
                )  # receiver of a method call counts as an input
            else:
                walk(node.func, depth)  # a plain callee is not an input
            gain = (
                0
                if (
                    skip_wrappers
                    and isinstance(node.func, ast.Name)
                    and node.func.id in WRAPPERS
                )
                else 1
            )
            for a in node.args:
                walk(a, depth + gain)
            for k in node.keywords:
                walk(k.value, depth + gain)
            return
        for ch in ast.iter_child_nodes(node):
            walk(ch, depth)

    walk(side, 0)
    return best


def has_call(node):
    return any(isinstance(n, ast.Call) for n in ast.walk(node))


def is_call_or_chain(node):
    # a Call, or an attribute/subscript chain; no literal anywhere inside
    return isinstance(node, (ast.Call, ast.Attribute, ast.Subscript))


def no_literal(node):
    return not any(isinstance(n, (ast.Constant, ast.JoinedStr)) for n in ast.walk(node))


def classify_pair(left, right, scopes):
    """Return (cat, info) for a 2-sided equality: cat in {'rt2','rt1','rel',None}."""
    params, locs, mods = scopes
    for exp, other in ((right, left), (left, right)):
        if isinstance(exp, ast.Name) and not isinstance(other, ast.Name):
            d = roundtrip_depth(exp.id, other)
            if d >= 1:
                n = exp.id
                where = (
                    "param"
                    if n in params
                    else "local"
                    if n in locs
                    else "module"
                    if n in mods
                    else "other"
                )
                if d >= 2:
                    strict = roundtrip_depth(exp.id, other, skip_wrappers=True) >= 2
                    return ("rt2", where + ("|strict" if strict else "|wrapper"))
                return ("rt1", where)
    if (
        is_call_or_chain(left)
        and is_call_or_chain(right)
        and no_literal(left)
        and no_literal(right)
    ):
        n = has_call(left) + has_call(right)
        sub = {2: "call_both_sides", 1: "call_one_side", 0: "no_call"}[n]
        return ("rel", sub)
    return (None, None)


def strip_common(l, r):
    """Peel identical wrappers off both sides: f(a) vs f(b) -> a, b; a.m() vs b.m() -> a, b; a.x vs b.x -> a, b."""
    while True:
        if (
            isinstance(l, ast.Call)
            and isinstance(r, ast.Call)
            and not l.keywords
            and not r.keywords
        ):
            if (
                isinstance(l.func, ast.Name)
                and isinstance(r.func, ast.Name)
                and l.func.id == r.func.id
                and len(l.args) == len(r.args) == 1
            ):
                l, r = l.args[0], r.args[0]
                continue
            if (
                isinstance(l.func, ast.Attribute)
                and isinstance(r.func, ast.Attribute)
                and l.func.attr == r.func.attr
                and not l.args
                and not r.args
            ):
                l, r = l.func.value, r.func.value
                continue
        if (
            isinstance(l, ast.Attribute)
            and isinstance(r, ast.Attribute)
            and l.attr == r.attr
        ):
            l, r = l.value, r.value
            continue
        return l, r


def flows(y, x, assigns, hops=0):
    """Call depth by which local y is computed from name x through Assign statements (0 = no flow)."""
    if hops > 4:
        return 0
    best = 0
    for expr in assigns.get(y, []):
        d = roundtrip_depth(x, expr)
        if d:
            best = max(best, d)
            continue
        for n in {
            m.id
            for m in ast.walk(expr)
            if isinstance(m, ast.Name) and isinstance(m.ctx, ast.Load)
        }:
            if n in assigns and n not in (y, x):
                inner = flows(n, x, assigns, hops + 1)
                if inner:
                    best = max(best, inner + roundtrip_depth(n, expr))
    return best


def cross_rt(left, right, assigns, scopes):
    l, r = strip_common(left, right)
    if not (isinstance(l, ast.Name) and isinstance(r, ast.Name)) or l.id == r.id:
        return None
    for y, x in ((l.id, r.id), (r.id, l.id)):
        d = flows(y, x, assigns)
        if d:
            params, locs, mods = scopes
            where = (
                "param"
                if x in params
                else "local"
                if x in locs
                else "module"
                if x in mods
                else "other"
            )
            return ("deep" if d >= 2 else "one", where)
    return None


def allany(node):
    """If node is all(...)/any(...), possibly negated or compared ==/is True, return arg kind."""
    shape = "plain"
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.Not):
        node, shape = node.operand, "not"
    elif (
        isinstance(node, ast.Compare)
        and len(node.ops) == 1
        and isinstance(node.ops[0], (ast.Eq, ast.Is))
        and isinstance(node.comparators[0], ast.Constant)
        and node.comparators[0].value in (True, False)
        and node.comparators[0].value is not None
        and isinstance(node.comparators[0].value, bool)
    ):
        node, shape = node.left, "cmp_bool"
    if (
        isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id in ("all", "any")
        and len(node.args) == 1
    ):
        a = node.args[0]
        if isinstance(a, ast.GeneratorExp):
            arg = "genexp"
        elif isinstance(a, (ast.ListComp, ast.SetComp, ast.DictComp)):
            arg = "comp"
        elif isinstance(a, ast.Name):
            arg = "name"
        elif isinstance(a, ast.Call):
            arg = "call"
        else:
            arg = "other"
        return node.func.id, shape, arg
    return None


EQ_ASSERTS = {"assertEqual", "assertEquals", "assertIs", "failUnlessEqual"}
TRUE_ASSERTS = {"assertTrue", "assertFalse", "assert_", "failUnless", "failIf"}


# ---------- per-test walk ----------
def walk_test(fn, suite, kind, path, lines, scopes):
    counted = collections.Counter()
    tail_stmts = set()

    def mark_tail(stmts):
        if not stmts:
            return
        last = stmts[-1]
        tail_stmts.add(id(last))
        if isinstance(last, (ast.With, ast.AsyncWith, ast.If)):
            mark_tail(last.body)
            if isinstance(last, ast.If):
                mark_tail(last.orelse)
        elif isinstance(last, ast.Try):
            mark_tail(last.finalbody or last.orelse or last.body)

    mark_tail(fn.body)
    last_top = fn.body[-1]
    assigns = collections.defaultdict(list)
    for n in ast.walk(fn):
        if (
            isinstance(n, ast.Assign)
            and len(n.targets) == 1
            and isinstance(n.targets[0], ast.Name)
        ):
            assigns[n.targets[0].id].append(n.value)
        elif (
            isinstance(n, ast.AnnAssign)
            and isinstance(n.target, ast.Name)
            and n.value is not None
        ):
            assigns[n.target.id].append(n.value)

    def on_cross(prefix, left, right, node):
        xr = cross_rt(left, right, assigns, scopes)
        if xr:
            counted[f"{prefix}x_rt_{xr[0]}"] += 1
            counted[f"{prefix}x_rt_{xr[0]}_{xr[1]}"] += 1
            example(f"{prefix}x_{xr[0]}", suite, path, node, lines)

    def visit(node, loop, subtest):
        # loop = outermost enclosing For/While node, or None
        if (
            isinstance(
                node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef)
            )
            and node is not fn
        ):
            for n in ast.walk(node):
                if isinstance(n, ast.Assert):
                    counted["assert_in_nested_def"] += 1
            return
        if isinstance(node, ast.Assert):
            on_assert(node, loop)
        if (
            isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and isinstance(node.func.value, ast.Name)
            and node.func.value.id == "self"
            and node.func.attr.startswith(("assert", "fail"))
            and node.func.attr not in ("failureException",)
        ):
            on_self_assert(node, loop, subtest)
        new_loop = loop
        if isinstance(node, (ast.For, ast.AsyncFor, ast.While)) and loop is None:
            new_loop = node
        new_sub = subtest
        if isinstance(node, (ast.With, ast.AsyncWith)) and any(
            isinstance(it.context_expr, ast.Call)
            and tail(it.context_expr.func) == "subTest"
            for it in node.items
        ):
            new_sub = True
        for ch in ast.iter_child_nodes(node):
            # the loop's iter/target are not its body
            if isinstance(node, (ast.For, ast.AsyncFor)) and ch in (
                node.iter,
                node.target,
            ):
                visit(ch, loop, subtest)
            elif isinstance(node, ast.While) and ch is node.test:
                visit(ch, loop, subtest)
            else:
                visit(ch, new_loop, new_sub)

    def on_assert(node, loop):
        counted["assert"] += 1
        t = node.test
        if (
            isinstance(t, ast.Compare)
            and len(t.ops) == 1
            and isinstance(t.ops[0], (ast.Eq, ast.Is))
        ):
            cat, info = classify_pair(t.left, t.comparators[0], scopes)
            if cat not in ("rt1", "rt2"):
                on_cross("A1", t.left, t.comparators[0], node)
                if cat == "rel" and cross_rt(t.left, t.comparators[0], assigns, scopes):
                    counted["A1x_also_counted_in_A2"] += 1
            if cat in ("rt1", "rt2"):
                counted[f"A1_{cat}"] += 1
                for part in info.split("|"):
                    counted[f"A1_{cat}_{part}"] += 1
                example(
                    f"1_{cat}" + ("_strict" if "strict" in info else ""),
                    suite,
                    path,
                    node,
                    lines,
                )
            elif cat == "rel":
                counted["A2_rel_eq"] += 1
                counted[f"A2_rel_eq_{info}"] += 1
                example("2_" + info, suite, path, node, lines)
        elif isinstance(t, ast.Compare) and len(t.ops) == 1:
            l, r = t.left, t.comparators[0]
            if (
                is_call_or_chain(l)
                and is_call_or_chain(r)
                and no_literal(l)
                and no_literal(r)
            ):
                counted["A2_rel_other_op"] += 1
                example("2_otherop", suite, path, node, lines)
        aa = allany(t)
        if aa:
            f, shape, arg = aa
            counted["A3_allany"] += 1
            counted[f"A3_{f}"] += 1
            counted[f"A3_shape_{shape}"] += 1
            counted[f"A3_arg_{arg}"] += 1
            example("3", suite, path, node, lines)
        if loop is not None:
            counted["A4_in_loop"] += 1
            if loop is last_top:
                counted["A4_loop_is_last_top_stmt"] += 1
            elif id(loop) in tail_stmts:
                counted["A4_loop_in_tail_via_with_if_try"] += 1
            else:
                counted["A4_loop_not_last"] += 1
            counted[f"A4_looptype_{type(loop).__name__}"] += 1
            example("4", suite, path, node, lines)

    def on_self_assert(node, loop, subtest):
        name = node.func.attr
        counted["S_self_assert"] += 1
        if name in EQ_ASSERTS and len(node.args) >= 2:
            cat, info = classify_pair(node.args[0], node.args[1], scopes)
            if cat not in ("rt1", "rt2"):
                on_cross("S1", node.args[0], node.args[1], node)
            if cat in ("rt1", "rt2"):
                counted[f"S1_{cat}"] += 1
                for part in info.split("|"):
                    counted[f"S1_{cat}_{part}"] += 1
                example(f"S1_{cat}", suite, path, node, lines)
            elif cat == "rel":
                counted["S2_rel_eq"] += 1
                counted[f"S2_rel_eq_{info}"] += 1
                example("S2", suite, path, node, lines)
        if name in TRUE_ASSERTS and node.args:
            aa = allany(node.args[0])
            if aa:
                f, shape, arg = aa
                counted["S3_allany"] += 1
                counted[f"S3_arg_{arg}"] += 1
                example("S3", suite, path, node, lines)
        if loop is not None:
            counted["S7_in_for_or_while"] += 1
            if isinstance(loop, (ast.For, ast.AsyncFor)):
                counted["S7_in_for"] += 1
            example("7_loop", suite, path, node, lines)
        if subtest:
            counted["S7_in_subTest"] += 1
            example("7_subtest", suite, path, node, lines)
        if loop is not None and subtest:
            counted["S7_in_loop_and_subTest"] += 1

    visit(fn, None, False)
    for k, v in counted.items():
        bump(suite, kind, k, v)
    # per-test flags
    flags = {k for k in counted if counted[k]}
    for f in (
        "A1_rt2",
        "A1_rt1",
        "A1x_rt_deep",
        "A1x_rt_one",
        "S1x_rt_deep",
        "S1x_rt_one",
        "A2_rel_eq",
        "A3_allany",
        "A4_in_loop",
        "S1_rt2",
        "S1_rt1",
        "S2_rel_eq",
        "S3_allany",
        "S7_in_for_or_while",
        "S7_in_subTest",
    ):
        if f in flags:
            bump(suite, kind, f"T_{f}")
    if "A4_in_loop" in flags:
        if counted["A4_loop_is_last_top_stmt"]:
            bump(suite, kind, "T_A4_loop_last_top")
    if any(
        tail(d.func if isinstance(d, ast.Call) else d).endswith("given")
        for d in fn.decorator_list
    ):
        bump(suite, kind, "T_given")
        d = next(
            d
            for d in fn.decorator_list
            if tail(d.func if isinstance(d, ast.Call) else d).endswith("given")
        )
        example("6_given", suite, path, d, lines)
    if not counted["assert"] and not counted["S_self_assert"]:
        bump(suite, kind, "T_no_assert_or_self_assert")


for project in sorted(p for p in root.iterdir() if p.is_dir()):
    suite = project.name
    for path in sorted(project.rglob("*.py")):
        if not (path.name.startswith("test_") or path.name.endswith("_test.py")) or any(
            part.startswith(".") for part in path.parts
        ):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        tree = ast.parse(text)
        lines = text.splitlines()
        rel = path.relative_to(root)
        hyp = imports_hypothesis(tree)
        C[suite]["files"] += 1
        if hyp:
            C[suite]["files_import_hypothesis"] += 1
            EX["6_import"].append(
                (suite, f"{rel}", "imports hypothesis at module level")
            )
        mods = module_names(tree)
        fns = []
        for node in tree.body:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                fns.append((node, "fn"))
            elif isinstance(node, ast.ClassDef):
                fns += [
                    (m, "meth")
                    for m in node.body
                    if isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef))
                ]
        for fn, kind in fns:
            if not fn.name.startswith("test"):
                continue
            bump(suite, kind, "T_tests")
            if hyp:
                bump(suite, kind, "T_tests_in_hypothesis_file")
            scopes = (fn_params(fn), fn_locals(fn), mods)
            walk_test(fn, suite, kind, rel, lines, scopes)

# ---------- output ----------
keys = sorted({k for c in C.values() for k in c})
suites = sorted(C)
with open(outdir / "per_suite.tsv", "w") as fh:
    fh.write("key\t" + "\t".join(suites) + "\tTOTAL\n")
    for k in keys:
        row = [C[s][k] for s in suites]
        fh.write(k + "\t" + "\t".join(map(str, row)) + f"\t{sum(row)}\n")
with open(outdir / "examples.json", "w") as fh:
    json.dump(EX, fh, indent=1)
print("wrote", outdir / "per_suite.tsv")
