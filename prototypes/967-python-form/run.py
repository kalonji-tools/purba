"""PROTOTYPE (#967). Run the examples of one form, or measure all six. Never merge.

    python run.py stacked            # or klass, below, call, record, module
    python run.py stacked --literals # the act may pass a literal
    python run.py measure            # line widths, and ty on each form
"""

import importlib
import json
import importlib.util
import os
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import traceback

HERE = os.path.dirname(os.path.abspath(__file__))
FORMS = ("stacked", "klass", "below", "call", "record", "module", "klasscall", "record_strict")
EXAMPLES = ("ex1_slug", "ex2_money", "ex3_sign_up", "ex4_cart", "ex5_refused", "ex6_typo", "ex7_inputs", "ex8_refused_typed", "ex9_generated")


def files_of(form: str, ex: str) -> list[str]:
    path = os.path.join(HERE, form, ex)
    if os.path.isdir(path):  # the module form: one file for each experiment, in the order of ORDER
        order = open(os.path.join(path, "ORDER"), encoding="utf-8").read().split()
        return [os.path.join(path, n + ".py") for n in order]
    return [path + ".py"] if os.path.exists(path + ".py") else []


def load(form: str, path: str) -> object:
    folder = os.path.dirname(path)
    if folder != os.path.join(HERE, form):  # a folder of experiment files that import each other
        return importlib.import_module(os.path.basename(path)[:-3])
    spec = importlib.util.spec_from_file_location(f"{form}_{os.path.basename(path)[:-3]}", path)
    assert spec and spec.loader, "a spec"
    mod = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = mod
    spec.loader.exec_module(mod)
    return mod


def run(form: str) -> None:
    import engine

    fm = importlib.import_module(f"form_{form}")
    print(f"══ form: {form}{'  (--literals)' if engine.OPTS['literals'] else ''} " + "═" * 40)
    for ex in EXAMPLES:
        paths = files_of(form, ex)
        if not paths:
            continue
        folder = os.path.dirname(paths[0])
        local = [os.path.basename(p)[:-3] for p in paths]
        for n in local:
            sys.modules.pop(n, None)
        sys.path.insert(0, folder)
        engine.REGISTRY.clear()
        print(f"\n── {os.path.relpath(os.path.join(HERE, form, ex), HERE)}{'/' if len(paths) > 1 or folder.endswith(ex) else '.py'}")
        loaded = []
        for p in paths:
            try:
                mod = load(form, p)
            except Exception as exc:  # noqa: BLE001
                tb = traceback.extract_tb(exc.__traceback__)[-1]
                where = os.path.relpath(p, HERE)
                print(f"   ✗ {where} CANNOT LOAD, so none of its experiments runs")
                print(f"       {type(exc).__name__}: {exc}  (line {tb.lineno})")
                engine.REGISTRY[:] = [e for e in engine.REGISTRY if os.path.abspath(e.file) != os.path.abspath(p)]
                continue
            if mod not in loaded:
                loaded.append(mod)
        for mod in loaded:
            if hasattr(fm, "collect"):
                fm.collect(mod)
        sys.path.remove(folder)
        engine.parse_all(stocks={"db"})
        failed: set[tuple[str, str]] = set()
        for e in engine.REGISTRY:
            print("\n".join(engine.report(e, failed)))


def ty(paths: list[str], extra: list[str]) -> list[str]:
    cmd = ["ty", "check", "--python", sys.executable, "--output-format", "concise"]
    for x in [HERE, *extra]:
        cmd += ["--extra-search-path", x]
    out = subprocess.run([*cmd, *paths], cwd=HERE, capture_output=True, text=True)
    hits = [ln for ln in out.stdout.splitlines() if ".py:" in ln]
    if shutil.which("pyright"):  # run under `nix-shell -p pyright` to add it
        cfg = os.path.join(os.environ["PROTO967_TMP"], "pyrightconfig.json")
        with open(cfg, "w", encoding="utf-8") as f:
            json.dump({"extraPaths": [HERE, *extra]}, f)
        out = subprocess.run(["pyright", "-p", cfg, *paths], cwd=HERE, capture_output=True, text=True)
        hits += ["pyright " + ln.strip().replace(os.path.dirname(ln.strip().split(":")[0]) + "/", "")
                 for ln in out.stdout.splitlines() if ".py:" in ln and " - " in ln]
    return hits


def measure() -> None:
    print("══ measure " + "═" * 50)
    print(f"{'form':<9}{'files':>6}{'lines':>7}{'chars':>8}{'widest':>8}{'>100':>6}   (ex1-ex4; blank lines, imports and docstrings left out)")
    for form in FORMS:
        lines, nfiles = [], 0
        for ex in EXAMPLES[:4]:
            for p in files_of(form, ex):
                nfiles += 1
                src = open(p, encoding="utf-8").read()
                if src.startswith('"""'):
                    src = src.split('"""', 2)[2]
                lines += [ln for ln in src.split("\n") if ln.strip() and not ln.startswith(("from ", "import "))]
        print(f"{form:<9}{nfiles:>6}{len(lines):>7}{sum(map(len, lines)):>8}{max(map(len, lines)):>8}{sum(len(ln) > 100 for ln in lines):>6}")
    for ex, what in (("ex6_typo", "a typo in the name of a step"), ("ex7_inputs", "a misspelled input, and an input of the wrong type")):
        print(f"\n── the type checkers on {ex}: {what}")
        for form in FORMS:
            paths = files_of(form, ex)
            hits = ty(paths, [os.path.dirname(paths[0])])
            short = [h if h.startswith("pyright ") else "ty      " + os.path.basename(h) for h in hits]
            print(f"   {form:<8} " + ("\n            ".join(short) if short else "no diagnostic"))
    print("\n── the type checkers on ex1-ex5 and ex8: diagnostics (ex5 and ex8 hold refusals on purpose)")
    for form in FORMS:
        paths = [p for ex in (*EXAMPLES[:5], EXAMPLES[7]) for p in files_of(form, ex)]
        hits = []
        for ex in (*EXAMPLES[:5], EXAMPLES[7]):
            ps = files_of(form, ex)
            if ps:
                hits += ty(ps, [os.path.dirname(ps[0])])
        print(f"   {form:<8} {len(hits)} diagnostics in {len(paths)} files")
        for h in hits:
            print(f"            {h.split(form + '/')[-1]}")


def main() -> None:
    sys.path.insert(0, HERE)
    args = sys.argv[1:] or list(FORMS)
    tmp = tempfile.mkdtemp(prefix="PROTOTYPE-wipe-me-967-")
    try:
        db = os.path.join(tmp, "users.sqlite")
        sqlite3.connect(db).execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT UNIQUE)").connection.commit()
        os.environ["PROTO967_DB"] = db
        os.environ["PROTO967_TMP"] = tmp
        import engine

        engine.OPTS["literals"] = "--literals" in args
        for a in args:
            if a == "measure":
                measure()
            elif a in FORMS:
                run(a)
                print()
    finally:
        shutil.rmtree(tmp)


if __name__ == "__main__":
    main()
