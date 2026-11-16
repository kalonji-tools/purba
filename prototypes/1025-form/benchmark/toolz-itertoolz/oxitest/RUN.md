# Run the port

All commands run from this folder (`toolz-itertoolz/oxitest/`). The evaluation
venv lives in another session's scratchpad, so it is named once by an absolute
path:

```sh
VENV=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/eb0d7984-d4f3-4812-9745-d481a5457e94/scratchpad/eval/.venv
```

The code under test is not installed. It is the source tree `../toolz-src`,
put on the path for each tool:

| Tool    | How it finds `../toolz-src`                         |
|---------|-----------------------------------------------------|
| oxitest | `PYTHONPATH=../toolz-src` on the command line        |
| ty      | `[tool.ty.environment] extra-paths` in `pyproject.toml` |
| pyright | `extraPaths` in `pyrightconfig.json`                 |

Tests (`strict = "abort"` comes from `pyproject.toml`; the run refuses to start
on any strict violation):

```sh
PYTHONPATH=../toolz-src "$VENV/bin/oxitest" -vv --color=never
```

Type checkers:

```sh
ty check --python "$VENV/bin/python" tests
nix-shell -p pyright --run "pyright --pythonpath $VENV/bin/python"
```

Regenerate the evidence files:

```sh
{ echo '$ PYTHONPATH=../toolz-src "$VENV/bin/oxitest" -vv --color=never'
  PYTHONPATH=../toolz-src "$VENV/bin/oxitest" -vv --color=never 2>&1; echo "exit=$?"
  echo; echo '$ PYTHONPATH=../toolz-src "$VENV/bin/oxitest" --color=never --json results.json  (per-test list)'
  PYTHONPATH=../toolz-src "$VENV/bin/oxitest" --color=never --json results.json > /dev/null 2>&1; echo "exit=$?"
  jq -r '.results.tests[] | "\(.status)  \(.name)"' results.json; } > OUTPUT.txt

{ echo '$ ty check --python "$VENV/bin/python" tests'; ty --version
  ty check --python "$VENV/bin/python" tests 2>&1; echo "exit=$?"
  echo; echo '$ nix-shell -p pyright --run "pyright --pythonpath $VENV/bin/python"'
  nix-shell -p pyright --run "pyright --version; pyright --pythonpath $VENV/bin/python" 2>&1; echo "exit=$?"; } > CHECKERS.txt
```

The console never names individual tests, even at `-vv`, so the second half of
`OUTPUT.txt` lists them from oxitest's JSON report.
