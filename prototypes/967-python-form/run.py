"""PROTOTYPE (#967). Run the examples of one form, or measure all three. Never merge.

    python run.py stacked            # or klass, or below
    python run.py stacked --literals # the act may pass a literal
    python run.py measure            # line widths and the type checker on each form
"""

import importlib.util
import os
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import traceback

HERE = os.path.dirname(os.path.abspath(__file__))
FORMS = ("stacked", "klass", "below")
EXAMPLES = ("ex1_slug", "ex2_money", "ex3_sign_up", "ex4_cart", "ex5_refused", "ex6_typo")


def run(form: str) -> None:
    import engine

    print(f"══ form: {form}{'  (--literals)' if engine.OPTS['literals'] else ''} " + "═" * 40)
    for ex in EXAMPLES:
        path = os.path.join(HERE, form, ex + ".py")
        engine.REGISTRY.clear()
        print(f"\n── {form}/{ex}.py")
        spec = importlib.util.spec_from_file_location(f"{form}_{ex}", path)
        assert spec and spec.loader, "a spec"
        mod = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = mod
        try:
            spec.loader.exec_module(mod)
        except Exception as exc:  # noqa: BLE001
            tb = traceback.extract_tb(exc.__traceback__)[-1]
            print(f"   ✗ THE MODULE CANNOT LOAD, so none of its experiments runs")
            print(f"       {type(exc).__name__}: {exc}  (line {tb.lineno})")
            continue
        engine.parse_all(stocks={"db"})
        failed: set[tuple[str, str]] = set()
        for e in engine.REGISTRY:
            print("\n".join(engine.report(e, failed)))


def measure() -> None:
    print("══ measure " + "═" * 50)
    print(f"{'form':<9}{'lines':>7}{'chars':>8}{'widest':>8}{'>100':>6}   (ex1-ex4, blank lines and docstrings left out)")
    for form in FORMS:
        lines = []
        for ex in EXAMPLES[:4]:
            src = open(os.path.join(HERE, form, ex + ".py"), encoding="utf-8").read().split("\n")
            body = [ln for ln in src if ln.strip() and not ln.startswith(('"""', "from ", "import "))]
            lines += [ln for ln in body if not (ln.startswith(("`", " ")) and ln.strip().startswith("`"))]
        lines = [ln for ln in lines if '"""' not in ln]
        print(f"{form:<9}{len(lines):>7}{sum(map(len, lines)):>8}{max(map(len, lines)):>8}{sum(len(ln) > 100 for ln in lines):>6}")
    print("\n── the type checker (ty) on ex6_typo.py of each form")
    for form in FORMS:
        out = subprocess.run(["ty", "check", "--python", sys.executable, "--extra-search-path", HERE,
                              "--output-format", "concise", os.path.join(form, "ex6_typo.py")],
                             cwd=HERE, capture_output=True, text=True)
        hits = [ln for ln in out.stdout.splitlines() if "ex6_typo.py" in ln]
        print(f"   {form:<8} " + ("\n            ".join(h.split("ex6_typo.py:")[-1] for h in hits) if hits else "no diagnostic"))
    print("\n── the type checker (ty) on ex1-ex5 of each form: diagnostics per file")
    for form in FORMS:
        out = subprocess.run(["ty", "check", "--python", sys.executable, "--extra-search-path", HERE,
                              "--output-format", "concise", *[os.path.join(form, e + ".py") for e in EXAMPLES[:5]]],
                             cwd=HERE, capture_output=True, text=True)
        hits = [ln for ln in out.stdout.splitlines() if ".py:" in ln]
        print(f"   {form:<8} {len(hits)} diagnostics")
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
