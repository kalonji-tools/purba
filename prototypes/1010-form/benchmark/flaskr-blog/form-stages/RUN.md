# Run

From this folder:

```sh
PYTHONPATH=../flaskr-src ../../.venv/bin/python ../../../run.py test_blog.py
```

Its output is in `OUTPUT.txt`.

The mutant run, with the copy of flaskr whose login fails:

```sh
PYTHONPATH=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/7035d15e-9c89-420f-a728-c3bb3c9b0a70/scratchpad/p1010/stage-porter/mutant-login ../../.venv/bin/python ../../../run.py test_blog.py
```

Its output is in `OUTPUT_mutant-login.txt`.

The checkers, from this folder (`ty.toml` and `pyrightconfig.json` point at the venv, `flaskr-src` and the folder of `cases.py`):

```sh
ty check --python ../../.venv/bin/python test_blog.py
nix-shell -p pyright --run 'pyright test_blog.py'
```

Their output is in `CHECKERS.txt`.
