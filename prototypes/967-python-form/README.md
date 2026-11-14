# PROTOTYPE: the Python form of an experiment (#967). Never merge.

Throwaway. Eight forms of the same experiments, so a reader can compare the code a tester writes.
One mini runner (`engine.py`) runs every form, so the screen is the same, and only the code differs.
The runner applies the answers of #963, #964, #965 and #966: one act, a fork for each case and each assay, a stock with a sample or a checkpoint, and a chain that replays its earlier steps.

Run it from this directory with any Python 3.12 on Linux:

```text
python run.py record_strict       # or stacked, klass, below, call, record, module, klasscall
python run.py stacked --literals  # the act may pass a literal
python run.py stacked --shared    # a replayed step shares the case's fixtures, as before #967 Q8
python run.py measure             # line widths, and ty on each form
nix-shell -p pyright --run 'python run.py measure'   # pyright beside ty
```

`OUTPUT.txt` holds one run of `python run.py stacked klass below call record module klasscall record_strict measure`, with pyright on the PATH.

| form | the act | a case | an assay | a step of a chain |
|---|---|---|---|---|
| `stacked/` | the decorated function | `@case("plain", input=dict(…), assay=dict(…))` above the act | `@assay(slug_of)` | `after(opened, "ann")`, by text |
| `klass/` | `def act(…)` in a class | `plain = case(text=…, product=…)`, flat keys | every other function in the class | `after(Opened.ann)`, an attribute |
| `below/` | the decorated function | `slug_of.case("plain", text=…).expect(product=…)` below the act | `@slug_of.assay` | `ann = opened.case(…)`, then `after(ann)` |
| `call/` | the decorated function | `plain = slug_of(text=…).expect(product=…)`: a typed call of the act | `@slug_of.assay` | `after(plain)`, a variable |
| `record/` | `def act(self)` in a class whose fields are the inputs | `plain = SlugOf(text=…).expect(product=…)`: a typed instance | every other method, reading `self.product` or a field | `after(plain)`, a variable |
| `module/` | `def act(…)` in a file of its own | `plain = case(text=…, product=…)`, flat keys | every other function the file defines | `from opened import ann`, then `after(ann)` |
| `klasscall/` | `def act(…)` in a class | `plain = case(act, text=…).expect(product=…)`: a typed call | every other function in the class | `after(Opened.ann)`, an attribute |
| `record_strict/` | `def act(self)`, abstract in the base | `plain = SlugOf(text=…).expect(product=…)`, and `expect` is declared once with a typed keyword for each assay | every other method | `ann.product` or `first.db`, typed |

| file | what it shows |
|---|---|
| `ex1_slug.py` | a pure function; the built-in assay `product`; `sep` as a default of the act |
| `ex2_money.py` | an expected warning, an expected raise, and an assay named like an input |
| `ex3_sign_up.py` | a stock `use(UsersDb)` and a fake `use(outbox)`; a case that chains to a case of its own experiment |
| `ex4_cart.py` | a chain of three steps; an act that returns `None` |
| `ex5_refused.py` | what the parse refuses |
| `ex6_typo.py` | a typo in the name of a step: who catches it |
| `ex7_inputs.py` | a misspelled input, and an input of the wrong type: who catches each |
| `ex8_refused_typed.py` | `record_strict/` only: the refusals of a declared `expect`, a field named like an assay, and a case name bound twice |
| `ex9_generated.py` | `record_strict/` only: cases that a loop builds, and a family of cases through a subclass |
| `ex10_stocks.py` | `record_strict/` only: a stock with a misspelled offer, a stock that offers nothing, and two readers of a stock with no checkpoint |
| `lab.py` | the fixtures every form shares; `UsersDb` declares a sample, a checkpoint and a fingerprint as its bases |

`module/` holds a folder for each example, one file for each experiment, and `ORDER` lists the files.

Weighed and not built:

- **Cases in the act's docstring, as `>>>` lines.** It is a table in text, and no type checker reads it.
- **No assay functions: `expect(product.cents, 1250)` through a recording proxy.** A type checker joins the two types, so a wrong expected type passes, and each case could read different values.
- **`with experiment(added) as e:` and cases inside.** It is `below/` with one more indent.
- **A generator that yields its cases.** It is logic in a test, and the records refuse a `yield`.
- **Typed expected values, `expect(items == 2)`, through `__eq__` on an assay.** It hides a comparison in an operator. No form here types an expected value.

## The form agreed on 2026-11-14: `record_strict/`

- **An experiment is a class whose fields are its inputs.** `act` is abstract in the base, so a type checker refuses an experiment without one, and an `act` whose type is not the product's.
- **A case is an instance, named by its variable.** A type checker sees a misspelled input and an input of the wrong type.
- **A step of a chain is `step.product` or `step.<field>`.** A type checker sees a chain that feeds the wrong type. `add()` returns `None`, so a checkout chains from the cart, an input: `two_books.cart`.
- **Each experiment declares `expect` once, with a typed keyword for each assay.** An assay that reads an input is required. An assay that reads the product defaults to `UNREAD`, because a case that raises has no product. A type checker sees a misspelled assay, a value of the wrong type, and a missing value for an assay that reads an input.
- **purba's parse refuses what a type checker does not see:** an `expect` out of step with the assays, a field and an assay with one name, an assay whose type no literal can write, and a case name that the file binds twice, which loses a case without a word.
- **`self.` stays.** pyright warns on any other name for it.
- **A literal that the act needs is a field with a default.** Every case shares it, any case may vary it, and the screen prints it with the inputs. A nested call is refused: each call is a step of a chain. A function of the test file as the act is refused, unless a reason is given; the form of the reason is open.
- **Cases are written one by one.** A loop or a list that builds cases is refused, and so is a subclass of an experiment.
- **The fixture of a stock declares each offer as a base:** `Samples`, `Checkpoints`, `Fingerprints`. A type checker sees a method of the wrong type. purba refuses a stock that leaves an offer's method unwritten, such as a misspelled one, a stock that offers neither a sample nor a checkpoint, and two assays that read a stock with no checkpoint. The bases take their parameters by position, so a stock names them freely.
- **A replayed step of a chain builds its own inputs.** A case gets only what its chain names, so `taken` reads no mail that the replay of `first` sent. A replay that agrees prints nothing; one that differs blocks the case.
- **An assay's reading is compared in the assay's process, by type first, then by value.** Only a verdict crosses back, or the two texts of a failure. `True` is not `1`, `1.0` is not `1`, and `nan` equals `nan`.

On `ex1`-`ex4`, ty and pyright report nothing. On `ex5` and `ex8`, each line they report is a line written to be refused.
