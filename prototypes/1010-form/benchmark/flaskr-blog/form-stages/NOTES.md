# Notes on the port

The port is one file, `test_blog.py`, and a copy of the original `data.sql`.
Two stages carry the shared arrange: `seeded` makes the app on a seeded database, and `logged_in` logs the user `test` in on a client of that app.
The temporary database file is a stock fixture, `TempDb`.
All 14 cases pass: the 12 cases of the original, and the 2 stages.

## Map

| Original | Port | Kind in the port |
|---|---|---|
| `app` fixture: temp file part (`mkstemp`, `os.close`, `os.unlink`) | `TempDb` | stock fixture (`Samples` + `Fingerprints`) |
| `app` fixture: `create_app`, `init_db`, load `data.sql` | `test_seed_app`, case `seeded` | stage; product `Flask` |
| `client` fixture | `app.test_client()` inside `test_login`, `test_login_required`, and `test_index` (its anonymous client) | plain call in the act |
| `auth` fixture, `AuthActions.login` | `AuthActions.login`, called in `test_login` | helper class; its call is the act of a stage |
| `auth.login()` at the start of a test | `test_login`, case `logged_in` | stage; product `FlaskClient` |
| `AuthActions.logout` | not ported | no test of `test_blog.py` calls it |
| `runner` fixture | not ported | no test of `test_blog.py` uses it |
| `test_index` | `test_index` | takes `use(logged_in)` |
| `test_login_required[/create]` | `anonymous_create` | case; takes `use(seeded)` |
| `test_login_required[/1/update]` | `anonymous_update` | case; takes `use(seeded)` |
| `test_login_required[/1/delete]` | `anonymous_delete` | case; takes `use(seeded)` |
| `test_author_required` | `test_author_required` | takes `use(logged_in)` |
| `test_exists_required[/2/update]` | `missing_update` | case; takes `use(logged_in)` |
| `test_exists_required[/2/delete]` | `missing_delete` | case; takes `use(logged_in)` |
| `test_create` | `test_create` | takes `use(logged_in)` |
| `test_update` | `test_update` | takes `use(logged_in)` |
| `test_create_update_validate[/create]` | `validate_create` | case; takes `use(logged_in)` |
| `test_create_update_validate[/1/update]` | `validate_update` | case; takes `use(logged_in)` |
| `test_delete` | `test_delete` | takes `use(logged_in)` |

## Which arranges became stages, and which did not

- **The seeded app is a stage.** Its act calls the code under test (`create_app`, `init_db`). Its checks read the two users and the one post that `data.sql` loads.
- **The login is a stage.** Its act posts to `/auth/login`. Its checks read the redirect to `/` and the user id `1` in the session. Nine cases take it.
- **The client is not a stage.** `app.test_client()` sends no request, so it has no state of its own to check. It is also not a fixture: `use(factory)` calls `factory()` with no input, so a fixture cannot take the app that the `seeded` stage makes. Each act that needs a client calls `app.test_client()`.
- **`AuthActions` is not a stage.** Its constructor only keeps the client. Only its `login` call acts, and that call is the act of `logged_in`.
- **The temporary database file is a stock, not a stage.** It holds no state of the code under test until the `seeded` stage writes to it.
- **The author change in `test_author_required` stays in that test's act.** Only one test uses it, and the original makes it part of that test. The original does not check it.

## Changes that the form or the type checkers forced

- **Checks move to the end of each test.** The form ends the act at the first check.
  - `test_index`: the original checks the logged-out page, logs in, then checks the logged-in page. The port reads both pages in the act and checks all six at the end.
  - `test_author_required`: the original sends the two POST requests inside its asserts. The port sends them in the act, keeps each response in a name, and checks the status codes at the end.
  - `test_create` and `test_update`: the original checks the GET status before the POST. The port keeps the GET response in `page` and checks its status at the end, inside the `with app.app_context()` block, next to the database check.
- **The database file is a stock.** Its state lives outside the process. `TempDb` gives each case a file of its own and removes it after the case. Its fingerprint is the file's bytes, so a check whose reading changes the database is refused.
- **Because the chain reads a stock, the runner replays the chain for each later case.** Each report line of a later case starts with `↺ … ran by replay: the chain reads a stock (TempDb)`. So the seed act runs in all 14 cases, and the login act runs in 10. Each replay compares the stage's checks again, and a stage that reads differently blocks the case. Each stage's checks are reported once, in the stage's own case.
- **A probe in the scratch folder replaced the stock with a plain fixture.** The chain then also ran by replay, for another reason: `the act left a file, socket or pipe open (fd 6 → /tmp/…)`. The seed stage's checks sit inside `with app.app_context()`, so the SQLite connection is open at the first check. That probe left its 14 database files in `/tmp`, because a plain fixture has no `discard`.
- **The client and the auth actions are built in the act.** A fixture takes no input (see above).
- **Parametrized cases become named cases.** The parameter ids become the names `anonymous_*`, `missing_*` and `validate_*`.
- **Type annotations are added** for the type checkers. `ty` and pyright report no error, with no suppression comment.
- **The original's `yield` teardown becomes `TempDb.discard`.** The form has no `yield`. `TempDb.sample` closes the `mkstemp` handle at once; the original keeps it open until teardown.

## Where the port checks less, more or differently

- **More: the stages check what the original assumes.** `seeded` checks the user rows `[(1, 'test'), (2, 'other')]` and the post row `[(1, 1, 'test title', 'test\nbody')]`. `logged_in` checks `Location == '/'` and session user id `1`. The original has no check of its fixtures or of the login.
- **The user rows are read `ORDER BY id`.** The first run read them in the order of the username index, `[(2, 'other'), (1, 'test')]`, and the stage failed. That run blocked the 13 later cases.
- **The login check reads `response.headers.get("Location")`.** So a failed login reads `None`, and does not raise `KeyError`. This was a choice, not forced.
- **Differently: `test_index` reads the logged-out page with a second client.** The original reads it on the same client before the login. The port's stage has already logged in its client, so the port asks a new client of the same app. The login does not change the database, so both clients see the same posts.
- **Differently: `test_author_required` logs in before the author change.** The original changes the author, then logs in. `get_post` reads the author at each request, so the order does not change the result.
- **Differently: every check runs.** pytest stops a test at its first failed assert. The form runs every check, and reads each later check alone too.
- **Differently: a broken login blocks the later tests.** In the original, each test that logs in fails at its own checks. In the port, the `logged_in` stage fails, and the runner blocks each case that takes it.
- **Not ported:** the `runner` fixture and `AuthActions.logout`. `test_blog.py` uses neither.

## Faults of the runner

- **A blocked case still gets a `↺ … ran by replay` line.** In `OUTPUT_mutant-login.txt`, each case blocked by `logged_in` shows `↺ test_index ran by replay: …` and then `⊘ test_index blocked: …`. The case was blocked before its act, so nothing ran by replay.
- **A passing `in` check prints the whole response body as its reading.** The body is cut in the middle, but each line is still long. This is noise, not a wrong result.
- The form expressed every test of the original. No workaround of the runner was needed.

## The mutant run

A copy of flaskr is in the scratch folder `p1010/stage-porter/mutant-login/`. One line of `auth.py` changes, so that a correct password fails the login:

```diff
-        elif not check_password_hash(user["password"], password):
+        elif check_password_hash(user["password"], password):
```

`OUTPUT_mutant-login.txt` holds the run of the port against it: `14 cases: 4 passed · 1 failed · 0 refused · 9 blocked`.

- `seeded` passes. The seeding does not depend on the login.
- `logged_in` fails both of its checks: `Location` reads `None` where `'/'` is expected, and the session user id reads `None` where `1` is expected.
- The three `anonymous_*` cases pass. They take `seeded`, not `logged_in`, and test the redirect of an anonymous client.
- The other 9 cases are blocked. Each line names the stage: `blocked: it takes the product of logged_in (test_blog.py:87), which failed`.
- So the login is probed once, by its own checks. No later test reports the broken login as a failure of its own checks.
