# PROTOTYPE: the Python form of an experiment (#967). Never merge.

Throwaway. Three forms of the same experiments, so a reader can compare the code a tester writes.
One mini runner (`engine.py`) runs every form, so the screen is the same, and only the code differs.
The runner applies the answers of #963, #964, #965 and #966: one act, a fork for each case and each assay, a stock with a sample or a checkpoint, and a chain that replays its earlier steps.

Run it from this directory with any Python 3.12 on Linux:

```text
python run.py stacked             # or klass, or below
python run.py stacked --literals  # the act may pass a literal
python run.py measure             # line widths, and ty on each form
```

`OUTPUT.txt` holds one run of `python run.py stacked klass below measure`.

| form | the act | a case | an assay | a step of a chain |
|---|---|---|---|---|
| `stacked/` | the decorated function | `@case("plain", input=dict(…), assay=dict(…))` above the act | `@assay(slug_of)` | `after(opened, "ann")`, by text |
| `klass/` | `def act(…)` in a class | `plain = case(text=…, product=…)`, flat keys | every other function in the class | `after(Opened.ann)`, an attribute |
| `below/` | the decorated function | `slug_of.case("plain", text=…).expect(product=…)` below the act | `@slug_of.assay` | `ann = opened.case(…)`, then `after(ann)` |

| file | what it shows |
|---|---|
| `ex1_slug.py` | a pure function; the built-in assay `product`; `sep` as a default of the act |
| `ex2_money.py` | an expected warning, an expected raise, and an assay named like an input |
| `ex3_sign_up.py` | a stock `use(UsersDb)` and a fake `use(outbox)`; a case that chains to a case of its own experiment |
| `ex4_cart.py` | a chain of three steps; an act that returns `None` |
| `ex5_refused.py` | what the parse refuses |
| `ex6_typo.py` | a typo in the name of a step: who catches it |
| `lab.py` | the fixtures every form shares; `UsersDb` offers a sample, a checkpoint and a fingerprint |
