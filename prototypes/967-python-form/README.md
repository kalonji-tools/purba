# PROTOTYPE: the Python form of an experiment (#967). Never merge.

Throwaway. Seven forms of the same experiments, so a reader can compare the code a tester writes.
One mini runner (`engine.py`) runs every form, so the screen is the same, and only the code differs.
The runner applies the answers of #963, #964, #965 and #966: one act, a fork for each case and each assay, a stock with a sample or a checkpoint, and a chain that replays its earlier steps.

Run it from this directory with any Python 3.12 on Linux:

```text
python run.py stacked             # or klass, below, call, record, module, klasscall
python run.py stacked --literals  # the act may pass a literal
python run.py measure             # line widths, and ty on each form
```

`OUTPUT.txt` holds one run of `python run.py stacked klass below call record module klasscall measure`.

| form | the act | a case | an assay | a step of a chain |
|---|---|---|---|---|
| `stacked/` | the decorated function | `@case("plain", input=dict(…), assay=dict(…))` above the act | `@assay(slug_of)` | `after(opened, "ann")`, by text |
| `klass/` | `def act(…)` in a class | `plain = case(text=…, product=…)`, flat keys | every other function in the class | `after(Opened.ann)`, an attribute |
| `below/` | the decorated function | `slug_of.case("plain", text=…).expect(product=…)` below the act | `@slug_of.assay` | `ann = opened.case(…)`, then `after(ann)` |
| `call/` | the decorated function | `plain = slug_of(text=…).expect(product=…)`: a typed call of the act | `@slug_of.assay` | `after(plain)`, a variable |
| `record/` | `def act(self)` in a class whose fields are the inputs | `plain = SlugOf(text=…).expect(product=…)`: a typed instance | every other method, reading `self.product` or a field | `after(plain)`, a variable |
| `module/` | `def act(…)` in a file of its own | `plain = case(text=…, product=…)`, flat keys | every other function the file defines | `from opened import ann`, then `after(ann)` |
| `klasscall/` | `def act(…)` in a class | `plain = case(act, text=…).expect(product=…)`: a typed call | every other function in the class | `after(Opened.ann)`, an attribute |

| file | what it shows |
|---|---|
| `ex1_slug.py` | a pure function; the built-in assay `product`; `sep` as a default of the act |
| `ex2_money.py` | an expected warning, an expected raise, and an assay named like an input |
| `ex3_sign_up.py` | a stock `use(UsersDb)` and a fake `use(outbox)`; a case that chains to a case of its own experiment |
| `ex4_cart.py` | a chain of three steps; an act that returns `None` |
| `ex5_refused.py` | what the parse refuses |
| `ex6_typo.py` | a typo in the name of a step: who catches it |
| `ex7_inputs.py` | a misspelled input, and an input of the wrong type: who catches each |
| `lab.py` | the fixtures every form shares; `UsersDb` offers a sample, a checkpoint and a fingerprint |

`module/` holds a folder for each example, one file for each experiment, and `ORDER` lists the files.

Weighed and not built:

- **Cases in the act's docstring, as `>>>` lines.** It is a table in text, and no type checker reads it.
- **No assay functions: `expect(product.cents, 1250)` through a recording proxy.** A type checker joins the two types, so a wrong expected type passes, and each case could read different values.
- **`with experiment(added) as e:` and cases inside.** It is `below/` with one more indent.
- **A generator that yields its cases.** It is logic in a test, and the records refuse a `yield`.
- **Typed expected values, `expect(items == 2)`, through `__eq__` on an assay.** It hides a comparison in an operator. No form here types an expected value.
