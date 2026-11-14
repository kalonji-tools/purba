# PROTOTYPE 2: the Python form of an experiment (#967). Never merge.

Throwaway. A new form, designed from what prototype 1 taught, and measured on the same loguru test.

- [`STRATEGY.md`](STRATEGY.md): the design, and the learning of prototype 1 that each rule answers.
- [`FORM.md`](FORM.md): the rules sheet, as the porter and the raters got it.
- [`benchmark/`](benchmark/README.md): the loguru port, and three blind ratings against pytest, oxitest and prototype 1.
- `experiment.py`: what a tester imports. `run.py`: the runner.
- `examples/`: the form on small examples. Some cases fail or are refused on purpose; `OUTPUT.txt` and `CHECKERS.txt` hold one run of each.

Run it from this directory with Python 3.12 on Linux:

```text
python run.py examples                 # or a file, or any folder
ty check --extra-search-path . --extra-search-path examples experiment.py run.py examples
nix-shell -p pyright --run pyright     # reads pyrightconfig.json; set its venvPath to a venv of your own
```

| file | what it shows |
|---|---|
| `ex1_slug.py` | a pure function, the built-in assay `product`, and a raise with `match` |
| `ex2_money.py` | an expected warning, an expected raise, and a warning that no case expects |
| `ex3_cart.py` | a stateful API in one act; a chain whose two inputs read one replay; a chain blocked by a step that reads differently |
| `ex4_compare.py` | a subclass of `str` equals a `str`; `True` is not `1` |
| `ex5_handling.py` | code that runs inside an `except`, with `handling(…)` |
| `ex6_refused.py` | what the parse refuses, case by case |
| `ex7_typed.py` | what ty and pyright catch: a misspelled input, an input of the wrong type, a product of the wrong type |
