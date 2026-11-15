# Run the port

All commands run from this folder:

```sh
cd benchmark/flaskr-blog/oxitest
```

The code under test stays in `../flaskr-src`, so `PYTHONPATH` points there.

Tests (strict = "abort" comes from `pyproject.toml`; the run refuses to start on any strict violation):

```sh
PYTHONPATH=../flaskr-src ../../.venv/bin/oxitest -vv --color=never
```

Type checkers:

```sh
PYTHONPATH=../flaskr-src ty check --python ../../.venv/bin/python tests
nix-shell -p pyright --run pyright      # reads pyrightconfig.json in this folder
```

Regenerate the evidence files:

```sh
{ echo '$ PYTHONPATH=../flaskr-src ../../.venv/bin/oxitest -vv --color=never'
  PYTHONPATH=../flaskr-src ../../.venv/bin/oxitest -vv --color=never 2>&1; echo "exit=$?"
  echo; echo '$ PYTHONPATH=../flaskr-src ../../.venv/bin/oxitest --color=never --json results.json'
  PYTHONPATH=../flaskr-src ../../.venv/bin/oxitest --color=never --json results.json >/dev/null 2>&1; echo "exit=$?"
  jq -r '.results.tests[] | "\(.status)  \(.name)"' results.json; } > OUTPUT.txt
{ echo '$ PYTHONPATH=../flaskr-src ty check --python ../../.venv/bin/python tests'; ty --version
  PYTHONPATH=../flaskr-src ty check --python ../../.venv/bin/python tests 2>&1; echo "exit=$?"
  echo; echo '$ nix-shell -p pyright --run pyright'
  nix-shell -p pyright --run 'pyright --version; pyright' 2>&1; echo "exit=$?"; } > CHECKERS.txt
```

The console never names individual tests, so the second half of `OUTPUT.txt`
lists them from oxitest's CTRF JSON report (`results.json`).
