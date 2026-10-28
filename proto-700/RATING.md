# Rating eight ways over seven tests

The second round of the [#700](https://github.com/kalonji-tools/purba/issues/700) prototype. Never merge it.

The person asked to rate `w08`–`w12` against `w01` (#690), `w03` (the alias) and `w06` (pytest), on more tests than the copy. The weights are readability > usability > clarity.

## What it found

The three blind readers ranked the alias last, and #690 near the bottom, in every test. They said what slowed them: the line names a type, and they had to hop through `fixtures.py` to find the fixture. The ways that name the fixture on the line read fastest. Of those, `= source` (`w09`) read fastest, but every type checker refuses it in every test, and #700 requires that a type checker accept the syntax. Of the rest, **`= use(tx, source)` (`w11`) read fastest, and it is the only way where the type checkers caught a fixture of the wrong type.** Under the rule fixed before the answers came in, the alias still scores 0.07 above `w11`. It gains on the length of its line, not on the readers.

## The cases

| case | test | what it adds |
|---|---|---|
| c1 | `test_copy(src, dst)` | two values of one type |
| c2 | `test_insert(db)` | one input |
| c3 | `test_fetch(client, db)` | two inputs of two types |
| c4 | `test_signup(client, db, settings, clock, mailer)` | five inputs |
| c5 | `seeded(src)` and `test_copy_seeded(seeded, src, dst)` | a fixture takes inputs the same way |
| c6 | `test_roundtrip(engine)` | one test, run on sqlite and on postgres |
| c7 | three tests on a source with three rows | one module replaces the source |

`cases/gen.py` writes every file from one spec. The bodies are the same in every way, and only the signatures differ. `cases/<way>/` holds each way.

## The scores

Each part scores 1 (worst of the eight) to 5 (best). The total is (3 × readability + 2 × usability + 1 × clarity) / 6. The mean over the seven cases:

| way | the line in c1 | readability | usability | clarity | **total** | total, readers only |
|---|---|---|---|---|---|---|
| `w09` provider as the default | `src: Tx[Database] = source` | 4.15 | 3.33 | 3.16 | **3.71** | 4.01 |
| `w03` alias | `src: SourceTx` | 3.01 | 4.03 | 3.57 | **3.45** | 2.45 |
| `w11` typed default | `src: Tx[Database] = use(tx, source)` | 3.07 | 3.96 | 3.16 | **3.38** | 3.95 |
| `w01` #690 | `src: Tx[SourceDb]` | 2.87 | 3.91 | 3.57 | **3.33** | 2.75 |
| `w06` pytest | `src: Tx[Database]` | 3.00 | 3.73 | 2.27 | **3.12** | 2.61 |
| `w10` default typed `Any` | `src: Tx[Database] = use_any(source)` | 2.99 | 3.16 | 3.16 | **3.07** | 3.54 |
| `w08` decorator | `@using(src=source, …)` | 2.99 | 2.99 | 3.16 | **3.02** | 3.30 |
| `w12` `Annotated` | `src: Annotated[Tx[Database], From(source)]` | 1.92 | 2.88 | 3.16 | **2.45** | 2.91 |

"Readers only" counts the readers' rank as all of readability, and drops the length and noise of the line. Under that rule the alias falls from second to last. `runs/score.json` holds every case. `runs/readers_only.txt` holds the second column.

### How each part scores

- **Readability** = ½ the readers' mean rank + ¼ the length of the line + ¼ its noise: framework names (`use`, `tx`, `From`, …) plus bracket depth.
- **Usability** = the mean of five parts. Do the checkers pass the code? Do they catch a fixture of the wrong type? Does go-to-definition reach the fixture? How many lines does the tester write? How many fixture names sit on the line, which a rename or an override must touch?
- **Clarity** = the mean of three parts. Did the readers answer right? Could they answer from the line alone? Can the line name a fixture of the wrong type at all?

A part that is measured on a scale scores by its place among the eight. Ties share their place.

## The measurements

| | `w01` | `w03` | `w06` | `w08` | `w09` | `w10` | `w11` | `w12` |
|---|---|---|---|---|---|---|---|---|
| mypy, pyright, ty pass the code | yes | yes | yes | yes | **no, 21 errors each** | yes | yes | yes |
| a fixture of the wrong type: checkers that report it | cannot be written | cannot be written | 0 | 0 | 3 | 0 | **3** | 0 |
| go-to-definition (jedi) from the line reaches | the `NewType` | the alias | the fixture (real pytest) | the fixture | the fixture | the fixture | the fixture | the fixture |
| lines written, all seven cases | 97 | 96 | 95 | 110 | 98 | 109 | 109 | 123 |
| readers' mean rank, 1 to 8 | 6.76 | **7.95** | 6.28 | 3.53 | **1.43** | 2.91 | 2.38 | 4.76 |
| answered from the line alone | 0% | 0% | 0% | 69% | 69% | 69% | 69% | 69% |

The imports printed the same lines on Python 3.12.15 and 3.14.4, as written and with `from __future__ import annotations` (`runs/cases_probe.txt`). The checkers ran on 3.12 (`runs/cases_checkers.txt`).

### What the readers said

Three fresh agents read every folder, with the eight ways shuffled under the letters A to H. Each had a different shuffle. The brief is `runs/readers/prompt.md`, and the answers and the mapping are in `runs/readers/`.

- **The readers ranked the alias last in 20 of their 21 rankings, and seventh in the other.** It is two hops away from its fixture: alias, then `NewType`, then the fixture that returns it.
- **Two readers ranked `w11` first in most cases, and the third ranked it fourth in every case.** It is the only way that names `tx`, so the line says how a `Database` becomes the `Tx` that the test takes. In `w08`, `w09`, `w10` and `w12` a fixture that returns a `Database` fills a `Tx[Database]`, and nothing says how.
- **Two readers could not tell whether `Tx[Database]` also matches `Tx[SourceDb]`** in `w01` and `w03`, because a `NewType` reads as a subtype.
- **In c7 the override rests on a rule that no file states** in `w01`, `w03` and `w06`: a fixture in the module wins over the one in `fixtures.py`.
- **No way tells a reader that `seeded` and the test share one `src`** (Q5b). Every reader answered "unclear" in nearly every way, so Q5b does not tell the ways apart.

## What did not vary

- The readers are agents, not people. They read every way, so a reader learns each helper once and reads it faster later.
- No resolver ran. Each way is shown and checked, not executed.
- The weights inside each part are mine, and I fixed them before the answers came back. The "readers only" column shows how much one of them moves the result.
- How `@fixture` declares a lifetime does not vary. purba has not decided it.
