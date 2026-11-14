"""Build blind copies of the #967 prototype: one folder per rater, forms under shuffled neutral letters."""

import os
import random
import re
import shutil
import sys

SRC = sys.argv[1]
DST = sys.argv[2]
FORMS = ["stacked", "klass", "below", "call", "record", "module", "klasscall", "record_strict"]
EXAMPLES = ["ex1_slug", "ex2_money", "ex3_sign_up", "ex4_cart", "ex5_refused", "ex6_typo", "ex7_inputs"]
LETTERS = "ABCDEFGH"


def scrub(text: str, mapping: dict[str, str]) -> str:
    text = re.sub(r'"""PROTOTYPE \(#967\)\. Form [A-Z_]+: the form agreed on 2026-11-14\. Never merge\.', '"""Form.', text)
    text = re.sub(r'"""PROTOTYPE \(#967\)\. Form [A-Z_]+:([^\n]*?) Never merge\.', r'"""Form:\1', text)
    text = text.replace(" Never merge; not how purba would build it.", " Not how purba would build it.")
    text = re.sub(r'"""PROTOTYPE \(#967\)\. ([^\n]*?) Never merge\.', r'"""\1', text)
    text = re.sub(r" ?\(#96\d[^)]*\)", "", text)
    text = text.replace("from the answers of #963-#966", "from earlier design answers").replace("Each form (stacked, klass, below) turns", "Each form turns")
    text = re.sub(r"  # ty[^\n]*", "", text)                       # checker hints only some forms carried
    text = text.replace(" Lines marked `# ty` are seen by a type checker too.", "")
    text = re.sub(r"  # the field is obscured, says pyright", "", text)
    for name, letter in sorted(mapping.items(), key=lambda kv: -len(kv[0])):
        text = re.sub(rf"\bform_{name}\b", f"form_{letter}", text)
    return text


def build(rater: int) -> dict[str, str]:
    rng = random.Random(9670 + rater)
    letters = list(LETTERS)
    rng.shuffle(letters)
    mapping = dict(zip(FORMS, letters))
    out = os.path.join(DST, f"rater{rater}")
    shutil.rmtree(out, ignore_errors=True)
    os.makedirs(out)
    for f in ("app.py", "lab.py", "engine.py"):
        open(os.path.join(out, f), "w").write(scrub(open(os.path.join(SRC, f)).read(), mapping))
    run = open(os.path.join(SRC, "run.py")).read()
    run = run.replace('FORMS = ("stacked", "klass", "below", "call", "record", "module", "klasscall", "record_strict")',
                      "FORMS = (" + ", ".join(f'"{l}"' for l in LETTERS) + ")")
    run = re.sub(r'EXAMPLES = \([^)]*\)', "EXAMPLES = (" + ", ".join(f'"{e}"' for e in EXAMPLES) + ")", run)
    run = run.replace('(*EXAMPLES[:5], EXAMPLES[7])', 'EXAMPLES[:5]').replace("and ex8: diagnostics (ex5 and ex8 hold refusals on purpose)", ": diagnostics (ex5 holds refusals on purpose)")
    run = run.replace("or klass, below, call, record, module", "or any letter A-H").replace("measure all six", "measure all eight")
    run = run.replace("python run.py stacked", "python run.py A")
    open(os.path.join(out, "run.py"), "w").write(scrub(run, mapping))
    for name, letter in mapping.items():
        mod = open(os.path.join(SRC, f"form_{name}.py")).read()
        open(os.path.join(out, f"form_{letter}.py"), "w").write(scrub(mod, mapping))
        for ex in EXAMPLES:
            s = os.path.join(SRC, name, ex)
            d = os.path.join(out, letter, ex)
            if os.path.isdir(s):
                os.makedirs(d)
                for f in os.listdir(s):
                    if f.endswith(".py") or f == "ORDER":
                        open(os.path.join(d, f), "w").write(scrub(open(os.path.join(s, f)).read(), mapping))
            else:
                os.makedirs(os.path.join(out, letter), exist_ok=True)
                open(d + ".py", "w").write(scrub(open(s + ".py").read(), mapping))
    open(os.path.join(out, "README.md"), "w").write(README)
    return mapping


README = """# Eight forms of one test design, A to H

Each folder `A/` … `H/` writes the same seven example files in one Python form of a test.
`form_X.py` is the small API that form `X` offers to a tester. `engine.py` is one shared mini runner, so every
form prints the same screen; only the tester's code differs. `app.py` is the code under test. `lab.py` holds the
fixtures every form shares (`UsersDb` is a sqlite "stock": state that lives outside the process).

The vocabulary: an **experiment** has **inputs**, one **act** (a single call) whose return value is the **product**,
and **assays** that each read the product or an input and hand back a reading. A **case** gives the inputs and the
expected reading of each assay. A **chain** step is a case whose input comes from an earlier case (it is replayed).

| file | what it shows |
|---|---|
| `ex1_slug.py` | a pure function; the built-in assay `product`; `sep` as a shared default |
| `ex2_money.py` | an expected warning, an expected raise, and an assay named like an input |
| `ex3_sign_up.py` | a stock `use(UsersDb)` and a fake `use(outbox)`; a case that chains to an earlier case of the same experiment |
| `ex4_cart.py` | a chain of three steps; an act that returns `None` |
| `ex5_refused.py` | what the runner refuses before anything runs |
| `ex6_typo.py` | a typo in the name of a chain step: who catches it |
| `ex7_inputs.py` | a misspelled input, and an input of the wrong type: who catches each |

Several examples fail or are refused on purpose (an accent bug, an unexpected warning, a wrong total, typos).

Run from this folder (Linux, Python 3.12):

```text
python run.py A            # one form; or several: python run.py A B C
python run.py measure      # line widths, and ty on each form
nix-shell -p pyright --run 'python run.py measure'   # adds pyright beside ty
```
"""

if __name__ == "__main__":
    for r in (1, 2, 3):
        m = build(r)
        print(f"rater{r}: " + " ".join(f"{k}={v}" for k, v in m.items()))
