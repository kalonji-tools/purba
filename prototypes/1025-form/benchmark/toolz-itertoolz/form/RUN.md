# Running the port

Run every command from this folder (`benchmark/toolz-itertoolz/form/`). `python` is CPython 3.12
(the eval venv: `.../eval/.venv/bin/python`, with pytest 9.2.0; the runner needs no package).

```sh
# the form's runner; the code under test goes on PYTHONPATH, cases.py is found next to run.py
PYTHONPATH=../toolz-src python ../../../run.py test_itertoolz.py

# repeat one run's values of random (none of these tests uses random)
PYTHONPATH=../toolz-src python ../../../run.py --seed 744333 test_itertoolz.py

# the type checkers; each reads its config in this folder (ty.toml, pyrightconfig.json),
# which puts ../../.. (cases.py) and ../toolz-src (toolz) on the search path
ty check
nix-shell -p pyright --run 'pyright'
```

`OUTPUT.txt` is the first command's output; `CHECKERS.txt` holds the output of `ty check` and `pyright`.
