# A stage as a default input (#1010, Q3) — throwaway prototype

The `use(stage)` half of the recommendation held. The "a fixture is one call" half fell as worded.
Few real fixtures act, so falsifier 1 did not fire. But the one-call test refuses 87% of real fixtures, and most of them do not act. It also lets through a fixture that acts inside one call.
Both blind readers picked the `use(stage)` port. Both said it costs one hop to find the setup.

It starts from the prototype of #980 (`proto/980-rules-at-run-time` at 9a9fa15), and changes only what Q3 needs.
Never merge it.

## The question

Where does an arrange + act live?

| answer | what a later test writes |
|---|---|
| ⓐ both homes, as today: a fixture may act, and no check sees it | `shop: Shop = use(shop_with_ann)` |
| ⓑ a fixture is one call; a longer body is refused with "make it a stage" | `shop: Shop = use(Shop)`, and the act goes in a stage |
| ⓒ ⓑ, plus a stage as a default input | `signed: Signed = use(ann)`, where `ann = case(test_sign_up, name="ann")` |

ⓒ is falsified if (1) most real fixtures act, so ⓑ refuses most of them at a high cost, or (2) a real file ported to `use(stage)` reads worse than its fixture version.

## What the runner adds

- `use(case)` in `cases.py`: the input is that case's product. `resolve` in `run.py` replays the case first, with its checks, as #980 replays the earlier case of a chain.
- A stage that failed, or was refused before or during its run, blocks each test that uses it. The line names the stage and its place: `blocked: it takes the product of logged_in (test_blog.py:42), which failed` (`stages_of`, `OUTCOME`).
- When the runner has no verdict for the stage (a stage of another file), the replay names the check that failed: `the stage wrong fails its check len(...) == lines at line 40: expected 9, read 1`.
- A stage whose test returns nothing is refused before the run.
- Rule ⓑ, behind `FORM_FIXTURE=one-call`: a fixture whose body is more than one `return` of at most one call is refused, and the refusal says "make it a stage" (`fixture_fault`).
- A test file that another test file imports, for a stage, is now loaded rewritten first. Under #980's runner, its asserts stayed Python's.
- A long reading is cut at a space, so the report shows no word in part. `cases.py` and `run.py` are #980's, wrapped to 100 columns; `runall.sh` folds each saved output to 100 bytes.

## Verdict

| falsifier | result | why |
|---|---|---|
| 1. most fixtures act | **did not fire** | 29 of 119 fixtures act on the code under test (24%). Counting any call beyond the constructor, 47 do (39%). In saleor, the business-logic suite, 28 of 77 (36%), or 39 of 77 (51%) on the broad count. |
| …so ⓑ refuses most fixtures | **fired anyway, for the wrong reason** | The one-call test refuses 104 of 119 (87%). 60 of those do not act even on the broad count: a config, a test double, a list of rows. It reads the shape of a fixture, not whether it acts. It also accepts 3 saleor fixtures that act inside one call (`return priced_checkout_factory(checkout_with_item)`). The probe `test_fixture_that_acts_in_one_call` shows the same hole. |
| 2. the `use(stage)` port reads worse | **did not fire** | Both blind readers picked the stage version: 7 of 9 and 8 of 9 tests, the rest the same. |

So `use(stage)` is worth having. A rule that sends an acting fixture to a stage needs another test than "one call". This prototype does not find one.

## Survey of real fixtures (`runs/survey.txt`)

Three suites. Each was shallow-cloned and read as data, never run.

| project | commit | fixtures | (i) one object | (ii) config / double / data | (iii) acts | (iv) teardown | acts, strict | acts, broad | ⓑ refuses | ⓑ refuses, no act | ⓑ accepts, acts | depth 1 / 2 / 3 / 4 / 5+ (max) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| flask (`tests/`, `examples/`) | 49943b5a | 23 | 6 | 8 | 0 | 9 | 1 | 7 | 17 | 10 | 0 | 13 / 9 / 1 / 0 / 0 (3) |
| loguru (`tests/`) | 28349be3 | 19 | 6 | 3 | 0 | 10 | 0 | 1 | 15 | 14 | 0 | 18 / 1 / 0 / 0 / 0 (2) |
| saleor (`saleor/checkout/tests/fixtures/`) | 5e1679b1 | 77 | 5 | 44 | 28 | 0 | 28 | 39 | 72 | 36 | 3 | 1 / 3 / 6 / 0 / 67 (12) |
| **total** | | **119** | **17** | **55** | **28** | **19** | **29** | **47** | **104** | **60** | **3** | 32 / 13 / 7 / 0 / 67 (12) |

- The classes were read by hand, one per fixture (`survey/labels.py`). A fixture with a teardown is (iv) even when it acts: flask's tutorial `app` creates the database, loads rows, and yields, so it is (iv) and counts in "acts, strict".
- **Acts, strict:** the fixture calls a business operation of the code under test, such as `add_variant_to_checkout`, `add_voucher_to_checkout` or `init_db`. **Acts, broad:** it calls anything of the code under test beyond its constructor, such as `app.config.update`, a route, a context push, `set_country`, or `fetch_checkout_info`. ORM rows built with `objects.create` count as data (ii).
- **ⓑ refuses** is counted mechanically by `survey/dump.py`, by the same test the runner applies.
- **Depth** follows pytest's scope: the same file, a `conftest.py` above it, or saleor's plugin modules. pytest's own fixtures do not count. Saleor's deepest chain has five checkout stages over seven product generators: `checkout_preorder_with_charged_payment → checkout_with_billing_address → checkout_with_shipping_method → checkout_with_shipping_address → checkout_with_variants → stock → variant → product → …`. That is the pipeline shape. Flask's deepest is `auth → client → app`.
- (iv) does not fit purba's form in any version: a case's process ends instead of a teardown. Those fixtures become `before_each`, a stock, or a `with` in the act. Rule ⓑ refuses all 19.

## The port (`ports/`)

`test_blog.py` of Flask's tutorial (flaskr), whose `app` fixture acts: it runs `init_db()` and loads `data.sql`. Each test then calls `auth.login()` in its act.
The code under test is copied to `undertest/flaskr-tutorial/` (BSD, `LICENSE.flask`), with one template line split in two; the page it renders is the same. The original test file is `ports/test_blog_original.py.txt`.

- `blog-fixtures/`: today's form. `app_with_data()` and `client_with_data()` are fixtures, and `login(client)` stays in each act, as in the original.
- `blog-stages/`: ⓒ. `test_app_with_data` checks the users and the post, then returns the app. `test_log_in` takes `use(app_with_data)`, checks the redirect, and returns the logged-in client. Tests take `use(logged_in)` or `use(app_with_data)`, and no test has a case line for its setup.

| run | fixtures form | stages form |
|---|---|---|
| as written | 13 passed (22 checks) | 15 passed (25 checks) |
| `FORM_FIXTURE=one-call` | 13 refused. Each names `client_with_data`, the wrapper the test names, not `app_with_data`, which acts | 15 passed |
| mutant: the test user's password hash loses a character, so the login fails | 4 passed, 9 failed on 13 scattered checks (`page.status_code == 200`, `count == 2`, `Title is required.`…); none names the login | 5 passed, 1 failed (`logged_in › response.headers['Location'] == '/'`), 9 blocked, each naming `logged_in (test_blog.py:42)` |
| time, best of 3 (`runs/timing.txt`) | 0.79 s | 0.89 s |

## Blind reading (`runs/blind_reading.txt`)

Two fresh agents got a neutral primer of the form and the two versions. The files were unlabelled, in a `shuf` order for each reader.

| reader | pick | strongest reasons | best point for the other |
|---|---|---|---|
| 1 | stages (7 of 9 tests, 2 the same) | a failed setup is blocked and named, not a confusing failure; the act holds only what is tested, with no `login` in it | fixtures are easier to find: no hop through `use(logged_in) → test_log_in → use(app_with_data)` |
| 2 | stages (8 of 9 tests, 1 the same) | the premises are checks, so an unseen `data.sql` cannot pass or fail a test; a failed setup is named | fixtures are simpler, and need no rule that `use(case)` runs another test |

Both readers distrusted the same things: a stage is named `test_…` and counts as a test; its checks run again in each case that uses it; some tests take `app` and some take a client; and `mkstemp` leaks its file in both versions.

## Probes (`probes/`)

Each docstring says what the runner should do, and each run matches (`runs/probes.txt`, `runs/probes_one-call.txt`, `runs/probes_far-alone.txt`). They cover `use(stage)`, a stage on a stage, a failed stage, a stage refused at run time (no check ran), a stage that returns nothing, fan-in, a stage of another file, rule ⓑ, and the one-call hole.
A stage's act runs again in each case that uses it, as a fixture's does. `runs/probes_replays.txt` counts `sign_up ann` 9 times in one run of the probe file.

## Holes kept in mind

- A test that takes the same stage twice gets one product: the replay is kept for the case. This was not probed.
- A stage of another file is replayed with its checks, but the runner has no verdict for it: a refusal at run time there (a path, no check) is not seen.
- The survey classes are one reader's judgement, read from source. Saleor's fixtures stand on a database, which purba would treat as a stock. This prototype does not weigh that.

## Run it

```sh
uv venv -p 3.12 .venv && uv pip install -p .venv/bin/python flask==3.1.2
bash runall.sh                                    # every folder above, into runs/
# the survey: clone each project at its commit (above) into one folder, then
python -I survey/dump.py <clone>/flask > survey/flask.json            # likewise loguru
python -I survey/dump.py <clone>/saleor/saleor/checkout/tests/fixtures > survey/saleor.json
python -I survey/dump.py <clone>/saleor > survey/saleor_all.json      # the fixtures saleor's tests can see, for depth
# saleor was a sparse clone: git sparse-checkout set --no-cone /saleor/checkout/tests/ /saleor/order/tests/ /saleor/tests/ conftest.py '/saleor/**/fixtures/' fixtures.py
python -I survey/tally.py
```

CPython 3.12.15, flask 3.1.2.
