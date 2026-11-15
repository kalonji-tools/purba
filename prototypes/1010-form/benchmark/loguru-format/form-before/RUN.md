# Run

From this folder (`benchmark/loguru-format/form-before/`):

```sh
../../.venv/bin/python ../../../run.py test_add_option_format.py
```

The code under test, loguru, is installed in the shared venv, so no `PYTHONPATH` is needed.
`run.py` puts its own folder on `sys.path`, so `from cases import …` resolves.

Type checkers, from this folder:

```sh
ty check --python ../../.venv/bin/python test_add_option_format.py
nix-shell -p pyright --run 'pyright test_add_option_format.py'
```

`pyrightconfig.json` points pyright at the venv and at the folder of `cases.py`.
