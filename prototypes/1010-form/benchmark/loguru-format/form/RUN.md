# Run

From this folder (`benchmark/loguru-format/form/`):

```sh
../../.venv/bin/python ../../../run.py .
```

loguru is installed in that venv, so no `PYTHONPATH` is needed. `run.py` puts its own folder, which holds `cases.py`, on `sys.path`.

The checkers, from the same folder:

```sh
ty check --python ../../.venv/bin/python test_add_option_format.py
nix-shell -p pyright --run 'pyright test_add_option_format.py'
```

`pyrightconfig.json` points pyright at the venv and at the folder of `cases.py`.
