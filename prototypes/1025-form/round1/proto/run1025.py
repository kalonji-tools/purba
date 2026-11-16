"""PROTOTYPE for #1025, throwaway: three candidate checks of each item, run for real.

python run1025.py <shape> test_file.py
  shape = all    rewrites `assert all(<check> for <t> in <reading>)`
  shape = for    rewrites a trailing `for <t> in <reading>: assert <check>`
  shape = each   runs the module as plain Python; `each()` from iterkit is a proxy

The rewrite turns the check into a call of `_each(line, src, reading, lambda t: ...)`,
which reads every item, and prints each item that breaks the check with its index.
A reading that is a one-shot iterator is refused before any item is read.
A reading whose iteration raises names the index it reached.
"""

import ast
import sys
import types
import traceback

SHAPES = ("all", "for", "each")


class Refused(Exception):
    pass


class Failed(Exception):
    pass


def is_one_shot(v):
    try:
        return iter(v) is v
    except TypeError:
        return False


def _each(line, src, reading, check, check_src):
    if is_one_shot(reading):
        raise Refused(
            f"line {line} reads {type(reading).__name__} {src!r}, an iterator that one reading empties;"
            f" make it a list in the act"
        )
    bad = []
    i = -1
    it = iter(reading)
    while True:
        try:
            item = next(it)
        except StopIteration:
            break
        except Exception as e:  # the reading's own iteration raised
            raise Failed(f"{src} raised {type(e).__name__}: {e} after item [{i}]") from None
        i += 1
        if not check(item):
            bad.append((i, item))
    if bad:
        lines = [f"assert {check_src}  for each item of {src}"]
        lines += [f"  [{k}]  {item!r}" for k, item in bad]
        lines.append(f"  {len(bad)} of {i + 1} items break it")
        raise Failed("\n".join(lines))


def check_shape_ok(node):
    """The form's check shapes: one comparison, not x, isinstance, startswith/endswith."""
    if isinstance(node, ast.Compare):
        return len(node.ops) == 1
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.Not):
        return True
    if isinstance(node, ast.Call):
        f = node.func
        if isinstance(f, ast.Name) and f.id == "isinstance":
            return True
        if isinstance(f, ast.Attribute) and f.attr in ("startswith", "endswith"):
            return True
    return False


class Rewrite(ast.NodeTransformer):
    def __init__(self, shape):
        self.shape = shape
        self.refusals = []

    def _call(self, line, target, it, check, kind="all"):
        lam = ast.Lambda(
            args=ast.arguments(posonlyargs=[], args=[ast.arg(arg="_t")], kwonlyargs=[], kw_defaults=[], defaults=[]),
            body=ast.Call(
                func=ast.Lambda(
                    args=ast.arguments(posonlyargs=[], args=[], kwonlyargs=[], kw_defaults=[], defaults=[]),
                    body=check,
                ),
                args=[],
                keywords=[],
            ),
        )
        # bind the loop target inside the lambda: (lambda _t: [(check) for <target> in [_t]][0])
        lam = ast.Lambda(
            args=ast.arguments(posonlyargs=[], args=[ast.arg(arg="_t")], kwonlyargs=[], kw_defaults=[], defaults=[]),
            body=ast.Subscript(
                value=ast.ListComp(
                    elt=check,
                    generators=[ast.comprehension(target=target, iter=ast.List(elts=[ast.Name("_t", ctx=ast.Load())], ctx=ast.Load()), ifs=[], is_async=0)],
                ),
                slice=ast.Constant(0), ctx=ast.Load(),
            ),
        )
        return ast.Expr(
            ast.Call(
                func=ast.Name("_each" if kind == "all" else "_some", ctx=ast.Load()),
                args=[ast.Constant(line), ast.Constant(ast.unparse(it)), it, lam, ast.Constant(ast.unparse(check))],
                keywords=[],
            )
        )

    def visit_FunctionDef(self, fn):
        if not fn.name.startswith("test"):
            return fn
        body = fn.body
        if self.shape == "all":
            new = []
            for st in body:
                t = st.test if isinstance(st, ast.Assert) else None
                if isinstance(t, ast.Call) and isinstance(t.func, ast.Name) and t.func.id in ("all", "any"):
                    g = t.args[0] if len(t.args) == 1 else None
                    if not isinstance(g, ast.GeneratorExp) or len(g.generators) != 1 or g.generators[0].ifs:
                        self.refusals.append(f"line {st.lineno}: all() takes one `for`, with no `if`")
                    elif not check_shape_ok(g.elt):
                        self.refusals.append(f"line {st.lineno}: the check of each item is not one comparison")
                    else:
                        gen = g.generators[0]
                        st = ast.copy_location(self._call(st.lineno, gen.target, gen.iter, g.elt, t.func.id), st)
                new.append(st)
            fn.body = new
        elif self.shape == "for":
            last = body[-1]
            if isinstance(last, ast.For) and len(last.body) == 1 and isinstance(last.body[0], ast.Assert):
                chk = last.body[0].test
                if check_shape_ok(chk):
                    body[-1] = ast.copy_location(self._call(last.lineno, last.target, last.iter, chk), last)
        return fn


def run(shape, path):
    src = open(path, encoding="utf-8").read()
    tree = ast.parse(src)
    rw = Rewrite(shape)
    if shape != "each":
        tree = rw.visit(tree)
        ast.fix_missing_locations(tree)
    mod = types.ModuleType("t")
    mod.__dict__["_each"] = _each
    mod.__dict__["_some"] = _some
    name = path.rsplit("/", 1)[-1]
    if rw.refusals:
        for r in rw.refusals:
            print(f"{name} project error: {r}")
        return 4
    exec(compile(tree, path, "exec"), mod.__dict__)
    code = 0
    for k, fn in list(mod.__dict__.items()):
        if not (k.startswith("test") and callable(fn)):
            continue
        try:
            fn()
            print(f"{name}::{k} passed")
        except Refused as e:
            print(f"{name}::{k} project error: {e}")
            code = max(code, 4)
        except (Failed, AssertionError) as e:
            print(f"{name}::{k} failed")
            msg = str(e) or traceback.format_exception_only(e)[-1].strip()
            print("\n".join("    " + ln for ln in msg.splitlines()))
            code = max(code, 1)
        except Exception as e:
            print(f"{name}::{k} raised: {type(e).__name__}: {e}")
            code = max(code, 4)
    return code



def _some(line, src, reading, check, check_src):
    if is_one_shot(reading):
        raise Refused(f"line {line} reads {type(reading).__name__} {src!r}, an iterator that one reading empties; make it a list in the act")
    items = list(reading)
    if not any(check(x) for x in items):
        lines = [f"assert {check_src}  for some item of {src}", "  no item holds it:"]
        lines += [f"  [{k}]  {x!r}" for k, x in enumerate(items)]
        raise Failed("\n".join(lines))


if __name__ == "__main__":
    sys.exit(run(sys.argv[1], sys.argv[2]))
