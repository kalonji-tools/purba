# PROTOTYPE 4: the Python form of an experiment (#967). Never merge.

Throwaway. Prototype 3's form, with one rule for the act, `before_each`, `handling`, and the test run as written; measured on the same loguru test as prototypes 1, 2 and 3.

- [`STRATEGY.md`](STRATEGY.md): the design, the answer to each question of prototype 3, and where each decision of the map is held.
- [`LEARNINGS.md`](LEARNINGS.md): what held, what failed, and what the next design must answer.
- [`FORM.md`](FORM.md): the rules sheet, as the porter and the raters got it.
- [`benchmark/`](benchmark/README.md): the loguru port, and three blind ratings beside pytest and oxitest.
- `cases.py`: what a tester imports. `run.py`: the runner.
- `examples/`: the form on small examples. Some cases fail or are refused on purpose; `OUTPUT.txt` and `CHECKERS.txt` hold one run of each.

Run it from this directory with Python 3.12 on Linux:

```text
python run.py examples                 # or a file, or any folder
ty check --extra-search-path . --extra-search-path examples cases.py run.py examples
nix-shell -p pyright --run pyright     # reads pyrightconfig.json; set its venvPath to a venv of your own
```

| file | what it shows |
|---|---|
| `test_1_slug.py` | a test with no case, a test with named cases and a check message, and an expected raise with `match` |
| `test_2_money.py` | an expected warning, an expected raise, and a warning that no test expects |
| `test_3_cart.py` | a stateful API in one act; a chain whose two inputs read one replay; a chain blocked by a step that reads differently |
| `test_4_compare.py` | a subclass of `str` equals a `str`; `True` is not `1`; checks with `in`, `<`, `!=`, and the expected value on the left |
| `test_5_except.py` | code inside an `except` with `handling`; a statement that did not raise; checks on `caught.value` |
| `test_6_refused.py` | what the parse refuses, every fault of a case at once |
| `test_7_typed.py` | what ty and pyright catch on a case line, and a wrong type on purpose with a coded ignore in the act |
| `test_8_stock.py` | a stock that offers a sample, and one that offers a checkpoint |
| `test_9_file.py` | `before_each`; a callback with an `if` that the test hands to the code; a loop over literal data; a helper the test calls itself, refused for its branch |
