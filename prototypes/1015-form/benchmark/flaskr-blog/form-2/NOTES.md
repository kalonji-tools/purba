# Notes on the port of flaskr's test_blog.py

The port is `test_blog.py`, with the tutorial's conftest in `flaskr_fixtures.py`. `data.sql` is a
copy of the original's file. The runner passed all 12 cases on the first attempt and refused none.
The two type checkers found nothing. Against a login that always fails, 9 of 12 cases fail. The 3
cases that pass are `test_login_required`, which never logs in.

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

| conftest fixture | Port |
|---|---|
| `app` (a temp database file per test, `yield`, then close and unlink) | `App(Samples[Flask], Fingerprints[Flask])`, a stock: `sample()` builds the app and database, `discard()` unlinks the file, `fingerprint()` hashes the file's bytes |
| `client(app)` | `client(app: Flask = use(App))` |
| `auth(client)`, `AuthActions` | `auth(client: FlaskClient = use(client))`, `AuthActions` unchanged except for type hints |
| `runner(app)` | not ported: `test_blog.py` does not use it |

The tests keep the names `app`, `client` and `auth` for their inputs. In each case, `app`, `client`
and `auth` share one app and one database.

## Changes that the form forced

1. **`app` is a stock class, not a function with `yield`.** Rule: "a `yield`" is refused, and "A
   stock is an input whose state lives outside the process, such as a database file. Its fixture
   subclasses `Samples[T]` … For each check, the fixture also subclasses `Fingerprints[T]` …". The
   cleanup after `yield` moved to `discard()`.
2. **`os.close(db_fd)` runs right after `mkstemp()`, not at teardown.** This follows from change 1:
   `discard(value)` gets only the app, not the descriptor. sqlite opens the file by its path, so
   the early close changes nothing for the tests.
3. **Each act runs before every check.** Rule: "The act ends at the first check, and only checks
   follow it." In `test_index`, the anonymous `client.get("/")` is kept in `anonymous`, and the
   login and second `get` follow it. All six checks come after the second `get`.
4. **A request inside an assert moved into the act, into a named variable.** Rule: "A reading
   reads; it does not act." This applies to `client.post("/1/update")`, `client.post("/1/delete")`
   and `client.get("/")` in `test_author_required`, to `client.post(path)` in
   `test_exists_required`, and to `client.get("/create")` and `client.get("/1/update")` in
   `test_create` and `test_update`. The status checks of `test_create` and `test_update` now come
   after the database read, not before the POST.
5. **The database checks moved out of the `with app.app_context():` block.** The act reads `count`
   or `post` inside the block, and the checks follow it. This follows from change 4: the status
   check must come after the act too, and it does not need the app context. Rule: "Checks at its
   end read under it; checks after it read once it is undone." Checks inside the block would also
   be allowed.
6. **Each parametrized item is a named case at module level.** Rule: "A case is
   `name = case(test, input=…, …)` at module level, named by its variable."
7. **Fixtures are `use(…)` defaults, not parameter names.** Rule: "Share a setup through a fixture:
   the default of an input, `use(factory)`."

## Changes that the type checkers forced

None. ty 0.0.99 and pyright 1.1.411 found nothing on the first run (`CHECKERS.txt`). I wrote the
type hints (`Flask`, `FlaskClient`, `TestResponse`, `AuthActions`) before I ran the checkers.

## Faults of the runner

None hit. The runner refused nothing and gave no false refusal.

One report text is hard to read but correct. In the mutant run, `test_delete` says
`they differ from character 1: expected '', read 'auth/login'`. These are the rests of `'/'` and
`'/auth/login'` after the common `/`.

## Where the port checks less, more or differently

- **More: no check may change the database.** The stock's `fingerprint()` hashes the database file
  before and after each check. I tested this outside the port with a temporary probe file, which I
  then deleted: `assert client.post("/create", …).status_code == 302` was refused with "reading it
  changed the stock App". The original has no such guard.
- **Differently: every check runs.** pytest stops a test at its first failed assert. The runner
  runs every check. For example, in the mutant run, `test_create` shows both `302` for the page
  and `1` for the count.
- **Differently: the order of the reads.** In `test_create` and `test_update`, the original checks
  the GET status before the POST. The port reads it into `page` before the POST and checks it
  after the database read. The same values are compared.
- **Differently: `test_index` makes both requests before any check.** The anonymous response is
  kept in `anonymous` and checked after the login. In the original, a failed anonymous check stops
  the test before the login. In the port, the login and the second request always run.
- **Less: none.** The original has 18 assert statements, which run 22 times over its 12 items.
  Each maps to one check of the port with the same comparison and the same expected value: 22
  checks over 12 cases.
