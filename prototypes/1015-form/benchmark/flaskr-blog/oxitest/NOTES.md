# Notes on the port

The port runs all 12 items of the original and all 12 pass under `strict = "abort"`.
The three fixtures that `test_blog.py` uses are `@oxi.fixture(lifetime="function")` declarations in `tests/__fixtures__.py`.
Every test asks for them as `Fixture[T]` parameters with the same names as in the original.
oxitest forced three changes: a message on each assert, frozen-dataclass cases with word IDs, and a type-checker cost.
`ty` and pyright both reject every `Fixture[T]` annotation.

## Map

| Original (pytest) | Port (oxitest) | Cases |
|---|---|---|
| `conftest.py` `_data_sql` | `tests/__fixtures__.py` `_data_sql`, read from `tests/data.sql` (a copy) | — |
| fixture `app` (`tempfile.mkstemp`, yield, close + unlink) | fixture `app(tmp: TempDir) -> Flask`, `lifetime="function"`, no yield | — |
| fixture `client(app)` | fixture `client(app: Fixture[Flask]) -> FlaskClient`, `lifetime="function"` | — |
| fixture `runner(app)` | not ported | — |
| class `AuthActions` | `tests/auth_actions.py` `AuthActions`, a plain class | — |
| fixture `auth(client)` | fixture `auth(client: Fixture[FlaskClient]) -> AuthActions`, `lifetime="function"` | — |
| `test_index` | `test_index` | 1 |
| `test_login_required[/create, /1/update, /1/delete]` | `test_login_required[create, update, delete]` | 3 |
| `test_author_required` | `test_author_required` | 1 |
| `test_exists_required[/2/update, /2/delete]` | `test_exists_required[update, delete]` | 2 |
| `test_create` | `test_create` | 1 |
| `test_update` | `test_update` | 1 |
| `test_create_update_validate[/create, /1/update]` | `test_create_update_validate[create, update]` | 2 |
| `test_delete` | `test_delete` | 1 |
| | | **12** |

All three parametrized tests share one case type, `Route(path: str)`, a frozen dataclass in `test_blog.py`.
The tests take `path` in expanded mode, as the original does.

## Changes that oxitest forced

- **A message on every assert.** Strict mode refuses a bare `assert`. Each of the 18 asserts now carries a short message. The compared values are unchanged.
- **Cases are frozen dataclasses with word IDs.** Strict mode refuses dict cases. A case ID is a Python keyword argument, so `/create` cannot be an ID. The IDs name the route instead: `create`, `update`, `delete`. In `test_exists_required` the IDs `update` and `delete` stand for `/2/update` and `/2/delete`.
- **Fixtures are annotated and injected by annotation.** Every fixture declares a return type, and every injected parameter is `Fixture[T]`. A fixture asks for another fixture the same way.

## Changes made by choice

- **`AuthActions` lives in its own module, `tests/auth_actions.py`.** The test module imports it to annotate `auth: Fixture[AuthActions]`. This keeps the test module from importing `__fixtures__.py`.
- **`runner` is not ported.** No blog test uses it, and the task ports only the fixtures that `test_blog.py` uses.

## Changes made to follow oxitest's documentation

- **The database file lives in the built-in `TempDir`.** The original makes a file with `tempfile.mkstemp`, then closes and unlinks it after the yield. The port asks for `tmp: TempDir` inside the `app` fixture and puts `flaskr.sqlite` in it. oxitest removes the directory at the `app` fixture's boundary, which is the end of the test. `app` therefore returns its value and has no teardown code of its own.
- **`data.sql` is read with `Path.read_text`** instead of `open(...).read().decode()`. The text is the same.

## Type checkers

`CHECKERS.txt` holds the real output: `ty` reports 21 diagnostics, and pyright reports 26 errors.
Every one of them comes from a `Fixture[T]` annotation. No other error is reported.

- `ty` rejects each `Fixture[T]` annotation: `Non-generic class _FixtureType cannot be specialized in a type expression` (21 annotations).
- pyright accepts the annotation, but types the parameter as the class `Fixture`, not as `T`. Each use of the value is then an error. `client.get` is an example: `Cannot access attribute "get" for class "Fixture"` (25 errors). `AuthActions(client)` in `__fixtures__.py` is `reportArgumentType` (1 error).
- oxitest's documentation says that `T` flows through to the type checker. With ty 0.0.99 and pyright 1.1.411 it does not.
- The other documented route, the `fx: Fixtures` proxy, types every value as `Any`. The documentation says so. It would silence both checkers, but no checker would then see the fixture types. The port keeps `Fixture[T]` and leaves the errors visible. It adds no suppression comment.

## oxitest problems hit

- The `Fixture[T]` type-checker failure above.
- The strict-mode documentation lists an "unused fixture" check. A probe added a typed `runner` fixture to `__fixtures__.py` that no test asks for. With `strict = "abort"`, and again with `--strict=enforce`, the run reported no violation and passed all 12 items.
- The documentation's code examples are `--8<--` include markers. The included files are not in `docs/user/`, so the examples were not readable. The port follows the prose and the installed package's docstrings.
- Nothing in the original was impossible to express.

## Where the port checks differently

- **Checks the same.** Each original assert is present, with the same operands and operator, in the same order.
- **Isolation is the same.** Each test gets its own app, client, `AuthActions` and database file. A probe on a copy of the port confirmed that the run can fail: changing `count == 2` to `count == 3` failed `test_create` (exit 1). Removing that assert's message made the run abort with exit 3 before any test ran.
- **Teardown differs.** The original closes the descriptor and unlinks the file. The port relies on `TempDir` to delete the directory that holds the file.
- **Test IDs differ.** `test_login_required[/create]` is now `test_login_required[create]`, and so on, as listed in the map.
