# Run the port

All commands run from this folder:

```sh
cd eval2/oxitest
```

Tests (strict = "abort" comes from `pyproject.toml`; the run refuses to start on any strict violation):

```sh
../.venv/bin/oxitest -vv --color=never
```

Type checkers:

```sh
ty check --python ../.venv/bin/python tests
nix-shell -p pyright --run pyright      # reads pyrightconfig.json in this folder
```

Regenerate the evidence files:

```sh
../.venv/bin/oxitest -vv --color=never > OUTPUT.txt 2>&1; echo "exit=$?" >> OUTPUT.txt
{ echo '$ ty check --python ../.venv/bin/python tests'; ty --version
  ty check --python ../.venv/bin/python tests 2>&1; echo "exit=$?"
  echo; echo '$ nix-shell -p pyright --run pyright'
  nix-shell -p pyright --run 'pyright --version; pyright' 2>&1; echo "exit=$?"; } > CHECKERS.txt
```

The console never names individual tests, so the second half of `OUTPUT.txt`
lists them from oxitest's CTRF JSON report:

```sh
../.venv/bin/oxitest --color=never --json results.json
jq -r '.results.tests[] | "\(.status)  \(.name)"' results.json
```
