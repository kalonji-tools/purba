# Run the port

From this folder:

```sh
PYTHONPATH=../flaskr-src ../../.venv/bin/python ../../../run.py .
```

`../../../run.py` is the form's runner, and `../flaskr-src` holds the code under test.

The type checkers, from this folder:

```sh
ty check --python ../../.venv/bin/python test_blog.py blog_fixtures.py
nix-shell -p pyright --run 'pyright test_blog.py blog_fixtures.py'
```

`ty.toml` and `pyrightconfig.json` add `../flaskr-src` and the runner's folder to the search paths.
