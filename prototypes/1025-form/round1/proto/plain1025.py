"""PROTOTYPE #1025: a plain check refuses a name that holds a one-shot iterator; a raise in the act names its frames."""
import ast, sys, types, traceback, inspect
from run1025 import is_one_shot

def _no_iter(line, names):
    for n, v in names.items():
        if is_one_shot(v):
            raise RuntimeError(f"REFUSED line {line} reads {n}, a {type(v).__name__} that one reading empties; make it a list in the act")

class R(ast.NodeTransformer):
    def visit_FunctionDef(self, fn):
        out = []
        for st in fn.body:
            if isinstance(st, ast.Assert):
                names = sorted({n.id for n in ast.walk(st.test) if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Load)})
                d = ast.Dict(keys=[ast.Constant(n) for n in names], values=[ast.Name(n, ctx=ast.Load()) for n in names])
                guard = ast.parse(f"_no_iter({st.lineno}, None)").body[0]
                guard.value.args[1] = d
                out.append(ast.copy_location(guard, st))
            out.append(st)
        fn.body = out
        return fn

def main(path):
    tree = R().visit(ast.parse(open(path).read())); ast.fix_missing_locations(tree)
    g = {"_no_iter": _no_iter, "__builtins__": __builtins__}
    name = path.rsplit("/", 1)[-1]
    exec(compile(tree, path, "exec"), g)
    for k, fn in list(g.items()):
        if not (k.startswith("test") and callable(fn)): continue
        try:
            fn(); print(f"{name}::{k} passed")
        except RuntimeError as e:
            print(f"{name}::{k} project error: {str(e).removeprefix('REFUSED ')}")
        except AssertionError:
            print(f"{name}::{k} failed")
        except Exception as e:
            tb = traceback.extract_tb(e.__traceback__)[1:]
            test_line = next(f for f in tb if f.name == k).lineno
            last = tb[-1]
            frame_code = e.__traceback__
            while frame_code.tb_next: frame_code = frame_code.tb_next
            gen = bool(frame_code.tb_frame.f_code.co_flags & inspect.CO_GENERATOR)
            where = f"inside the generator {last.name}() at line {last.lineno}" if gen else f"in {last.name}() at line {last.lineno}"
            print(f"{name}::{k} raised: {type(e).__name__}: {e}\n    line {test_line} of the test; raised {where}")
main(sys.argv[1])
