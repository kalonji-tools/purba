# Notes on the port of flaskr `test_blog.py`

The original has 8 tests. With its parametrized items, it has 12 items, and the port has 12 cases. All 12 cases pass, and the port has 22 checks, one for each assert that the original runs. The runner refused nothing on the first run. Every change below moves code. No check was dropped, and no check was weakened.

The fixtures sit in `test_blog.py` itself, and the port has no separate fixtures module. The tutorial's chain `app` → `client` → `auth` is kept, one factory for each fixture. The `runner` fixture is not ported, because no test of `test_blog.py` uses it.

## What maps to what

| Original test (items) | Port test | Cases | Checks per case |
|---|---|---|---|
| `test_index` (1) | `test_index` | `test_index` (runs under its own name) | 6 |
| `test_login_required[/create, /1/update, /1/delete]` (3) | `test_login_required` | `login_required_create`, `login_required_update`, `login_required_delete` | 1 |
| `test_author_required` (1) | `test_author_required` | `test_author_required` | 3 |
| `test_exists_required[/2/update, /2/delete]` (2) | `test_exists_required` | `exists_required_update`, `exists_required_delete` | 1 |
| `test_create` (1) | `test_create` | `test_create` | 2 |
| `test_update` (1) | `test_update` | `test_update` | 2 |
| `test_create_update_validate[/create, /1/update]` (2) | `test_create_update_validate` | `create_update_validate_create`, `create_update_validate_update` | 1 |
| `test_delete` (1) | `test_delete` | `test_delete` | 2 |
| **8 tests, 12 items** | **8 tests** | **12 cases** | **22 checks** |

| Original conftest | Port |
|---|---|
| fixture `app` (temporary file, `create_app`, `init_db`, `data.sql`; teardown closes and removes the file) | stock `BlogDb` (`Samples[Path]` + `Fingerprints[Path]`) for the file, and factory `app(db=use(BlogDb))` for the app |
| fixture `client(app)` | factory `client(app=use(app))` |
| class `AuthActions` | class `AuthActions`, the same |
| fixture `auth(client)` | factory `auth(client=use(client))` |
| fixture `runner(app)` | not ported: no test in `test_blog.py` uses it |

## Changes that the form forced

1. **`test_index`: the login and the second `GET /` moved to before all checks.** Rule: "The act ends at the first check, and only checks follow it." The original checks the anonymous page, then logs in and gets the page again. The port keeps the two responses as `anonymous` and `logged_in`, and checks both at the end. The checks are the same, in the same order.
2. **`test_author_required`, `test_exists_required`: each request moved out of its assert into the act.** Rule: "A reading reads; it does not act." The original writes `assert client.post("/1/update").status_code == 403`. The port binds `update = client.post("/1/update")` in the act, and checks `update.status_code`. A probe confirmed the rule: a check `assert client.post("/1/delete").status_code == 302` was refused with "reading it changed the stock BlogDb".
3. **`test_create`, `test_update`: the check of the `GET` form's status moved from before the `POST` to the end.** Rule: "The act ends at the first check." The port binds `form = client.get(...)` and checks `form.status_code == 200` with the other checks.
4. **`test_create`, `test_update`, `test_delete`: all checks sit at the end of the `with app.app_context():` block.** Rule: "Checks at its end read under it." The database query is part of the act, so every check must follow it. In `test_delete`, the `Location` check therefore moved into the `with` block, before `post is None`, to keep the original order.
5. **Each parametrized item became a named case line.** Rule: "A case is `name = case(test, input=…, …)` at module level, named by its variable." `@pytest.mark.parametrize("path", …)` became `path: str` as an input, plus one `case(...)` line for each value.
6. **The pytest fixtures became `use(...)` defaults.** Rule: "Share a setup through a fixture: the default of an input, `use(factory)`", and "A fixture may use a fixture."
7. **The temporary database file became a stock, `BlogDb`.** Rule: "A stock is an input whose state lives outside the process, such as a database file", and "For each check, the fixture also subclasses `Fingerprints[T]` … or `Checkpoints`." `sample()` makes the file, and `discard()` removes it, in place of the original's code after `yield`. The form refuses `yield`, so the original's teardown could not stay in the `app` factory. `fingerprint()` returns the file's bytes. One run left no temporary file behind in `/tmp`.

Choices that the form did not force:

- `sample()` closes the file descriptor from `mkstemp` at once. The original keeps it open until teardown. No check reads it, so the change has no effect on any check.
- The conftest reads `data.sql` with `with open(...)`, and the port reads it with `Path.read_bytes()`. The conftest's private name `_data_sql` became `DATA_SQL`.
- Type annotations were added to every test, fixture and method. They let the runner and the checkers verify each input.

## Changes that the type checkers forced

None. The annotations were written before the first check run. ty reports no faults. pyright in strict mode reports one error: `Type of "create_app" is partially unknown`, at the import of `create_app`. The cause is in the code under test, which has no annotation on `test_config`. A fix in the test code would need a `cast` or a `# pyright: ignore`, so the error stays, and `CHECKERS.txt` shows it. Two probes confirmed that both checkers resolve `cases.py` and `flaskr`: a misspelled case input (`paht=`) and a missing name in `flaskr.db` each gave errors in both checkers.

## Faults of the runner

None hit. One detail of the report differs from FORM.md. FORM.md says the run ends with `N cases: … passed · … failed · … refused · … blocked · … skipped`. The runner printed `12 cases: 12 passed · 0 failed · 0 refused · 0 blocked   (22 checks held)`, with no `skipped` field, because this port has no skipped case. The runner possibly prints the field only when the count is above 0.

## Where the port checks less, more or differently

- **More: every check runs, even after another fails.** pytest stops a test at its first failed assert. In the mutant run, the port reported all three checks of `test_author_required`, where pytest would report only the first one.
- **More: each check is fingerprinted against the database file.** A check that changes the database is refused (see change 2).
- **More: `==` compares by type.** `status_code` is an `int`, and the expected values are `int` literals, so no result changes.
- **Same:** each check reads the same value as in the original. The moved `GET` checks read the same response objects, and a response does not change after its request.
- **Mutant run (`OUTPUT_mutant-login.txt`): the login was changed to always fail, and 9 of the 12 cases failed.** The 3 `login_required_*` cases passed, as they should, because they do not log in. In `test_index`, 5 of 6 checks still held, because flaskr shows the posts to anonymous users too. Only the check of the edit link (`href="/1/update"`) saw the failed login. The original has the same blind spot.
