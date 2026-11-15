# Notes on the port

## What maps to what

| Original | Port |
|---|---|
| fixture `app` (conftest.py) | stock `BlogDb` in `blog_fixtures.py`: `Samples[Blog]` and `Fingerprints[Blog]`. `sample()` makes the temporary database and the app. `discard()` closes and removes the file. |
| fixture `client` (conftest.py) | field `Blog.client`, built in `BlogDb.sample()` |
| fixture `auth` and class `AuthActions` (conftest.py) | field `Blog.auth`, built in `BlogDb.sample()`; `AuthActions` is ported as it is |
| fixture `runner` (conftest.py) | not ported: no test in `test_blog.py` uses it |
| `_data_sql` and `data.sql` | `_data_sql` in `blog_fixtures.py`; `data.sql` is copied into this folder |
| `test_index` | `test_index`, one case under its own name |
| `test_login_required[/create]`, `[/1/update]`, `[/1/delete]` | `test_login_required`, cases `login_required_create`, `login_required_update`, `login_required_delete` |
| `test_author_required` | `test_author_required`, one case |
| `test_exists_required[/2/update]`, `[/2/delete]` | `test_exists_required`, cases `exists_required_update`, `exists_required_delete` |
| `test_create` | `test_create`, one case |
| `test_update` | `test_update`, one case |
| `test_create_update_validate[/create]`, `[/1/update]` | `test_create_update_validate`, cases `create_update_validate_create`, `create_update_validate_update` |
| `test_delete` | `test_delete`, one case |

Each test takes `blog: Blog = use(BlogDb)`. The port has no stages: no original test hands a product to another.
The original has 12 test items. The port has 12 cases, and all 12 pass, with 22 checks.

## Changes that the form forced

- A fixture of the form is `use(factory)`, and the runner calls `factory()` with no arguments. A fixture cannot take another fixture. So `app`, `client` and `auth` are one fixture, and its value `Blog` holds all three. A test reads `blog.app`, `blog.client` and `blog.auth`.
- The database file lives outside the process, so `app` is a stock. A stock must offer `Fingerprints` or `Checkpoints`. `BlogDb.fingerprint()` returns the bytes of the database file. The original has no such code.
- The pytest `yield` fixture becomes `sample()` and `discard()`. The file descriptor from `mkstemp` is kept on the stock, and `discard()` closes it, as the original's teardown does.
- `@pytest.mark.parametrize` becomes one named case for each value.
- The act ends at the first check. In `test_index`, the original checks the anonymous page before the login. The port makes both requests first, and then checks both pages. The requests run in the same order.
- A check reads; it does not act. In `test_author_required`, `test_exists_required`, `test_create` and `test_update`, the original makes a request inside an `assert`. The port makes each request in the act, keeps the response in a name, and checks it at the end.
- In `test_create`, `test_update` and `test_delete`, the original checks the database inside `with app.app_context():`. The port reads the value inside that `with`, and checks it after the `with`.

## Changes that the type checkers forced

- The port has type annotations. The original has none.
- `ty` finds the project root above this folder, so it did not find `flaskr`. A `ty.toml` in this folder adds `../flaskr-src` and the runner's folder to the search paths. `pyrightconfig.json` does the same for pyright, and also names the venv.
- With these two files, `ty` and pyright report no diagnostics. No `# type: ignore` is in the port.

## Faults of the runner

- None found in this port. The run reports 12 passed, 0 failed, 0 refused and 0 blocked.

## Where the port checks less, more or differently

- More: before and after each check, the runner compares the fingerprint of the database file. A check that changed the database would be refused. The original has no such guard.
- More: the runner reads each check after the first alone, in a fork. The original reads each check once, in order.
- Differently: the original stops a test at its first failed assert. The port runs every check, even after one fails.
- Differently: the database checks of `test_create`, `test_update` and `test_delete` compare a value read under the app context. The original compares it under that context. The values are the same.
- Differently: in `test_index`, a failed anonymous check no longer stops the login and the second request.
- Same: each case has its own temporary database, its own app and its own client, as each pytest test has.
- `AuthActions.logout` is ported, but no test calls it, as in the original.
