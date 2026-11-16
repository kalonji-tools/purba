# How to run the port

Run each command from this folder (`benchmark/loguru-format/form/`).

```sh
# the form's runner (writes OUTPUT.txt)
../../.venv/bin/python ../../../run.py test_add_option_format.py > OUTPUT.txt 2>&1

# the type checkers (CHECKERS.txt holds both outputs)
ty check test_add_option_format.py
nix-shell -p pyright --run 'pyright test_add_option_format.py'
```

- `ty.toml` points ty at the venv (`../../.venv`) and at `cases.py` (`../../..`).
- `pyrightconfig.json` points pyright at the same two places (`venvPath` `../..`, `venv` `.venv`, `extraPaths` `["../../.."]`).
- The versions used: ty 0.0.99, pyright 1.1.411, CPython 3.12.15, loguru 0.8.0.
