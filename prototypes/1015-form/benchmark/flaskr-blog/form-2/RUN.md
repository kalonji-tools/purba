# Run the port

Run every command from this folder (`benchmark/flaskr-blog/form-2/`).

```sh
# the port, with the form's runner (saved as OUTPUT.txt)
PYTHONPATH=../flaskr-src ../../.venv/bin/python ../../../run.py .

# the type checkers (saved as CHECKERS.txt); both read their config from this folder
ty check
nix-shell -p pyright --run 'pyright'

# the port against a broken login (saved as OUTPUT_mutant-login.txt)
mkdir -p _mutant && cp -r ../flaskr-src _mutant/flaskr-src
# edit _mutant/flaskr-src/flaskr/auth.py: in login(), make the first branch `if True:` so
# `error` is always "Incorrect username."
PYTHONPATH=_mutant/flaskr-src ../../.venv/bin/python ../../../run.py .
rm -rf _mutant
```

`pyrightconfig.json` and `ty.toml` point at the venv `../../.venv`, at `../flaskr-src` (the code
under test) and at `../../..` (the form folder, for `cases.py`).
