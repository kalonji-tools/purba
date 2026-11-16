# How to run this port

Run each command from this folder (`benchmark/flaskr-blog/form/`).

## The tests

```sh
PYTHONPATH=../flaskr-src ../../.venv/bin/python ../../../run.py .
```

The output of the last run is in `OUTPUT.txt`. The run picks a new seed each time; add `--seed N` after `run.py` to repeat a seed.

## The type checkers

```sh
ty check
nix-shell -p pyright --run 'pyright'
```

`ty.toml` and `pyrightconfig.json` point both checkers at the venv in `../../.venv`, at the code under test in `../flaskr-src`, and at `cases.py` in `../../..`. Their output is in `CHECKERS.txt`.

## The run against a broken login

```sh
mkdir -p _mutant && cp -r ../flaskr-src _mutant/flaskr-src
# in _mutant/flaskr-src/flaskr/auth.py, in login(), just before `if error is None:` add:
#         error = "Login always fails (mutant)."
PYTHONPATH=_mutant/flaskr-src ../../.venv/bin/python ../../../run.py . > OUTPUT_mutant-login.txt 2>&1
rm -r _mutant
```
