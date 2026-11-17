# A test that expects a raise names more than its type, so selection refuses an except that only passes

## Context and Problem Statement

A test can expect a raise by a `try` whose `except` only passes.
On `main` at `ee8ac272`, such a test passes whenever its statement raises that type, for any reason.

| fact | source |
|---|---|
| of 166 tests that expect a raise by its type alone, 12 still passed when the `raise` they expect became `pass`, and a `match=` of the first message would have failed each of them | [the mutation run](https://github.com/kalonji-tools/purba/blob/7048e8c82ee75e7dcfbe2e0b8111e80869451f1c/prototypes/1021-raise-message/mutation/runs/summary.txt) |
| of 131 tests with a `match=`, 1 still passed, and of 65 tests that check the caught value, 1 still passed | the same |
| of 259 raises that the pinned suites expect by the type alone, 183 have two or more `raise` statements of that type in the project's own code | [Must a test that expects a raise name its message as well as its type?](https://github.com/kalonji-tools/purba/issues/1021#issuecomment-6919967513) |
| on `main` at `ee8ac272`, an `except` that only passes passes when it names a tuple, when it is a bare `except:`, and in a `try` of two statements | [Refuse a try whose except only passes, since it expects a raise by its type alone](https://github.com/kalonji-tools/purba/issues/1034) |
| on `main` at `ee8ac272`, `except Closed as e:` makes another `try`, so its statement that does not raise gives `[second-path]` | the same |

## Considered Options

- **A test expects a raise by its type alone.** [Must a test that expects a raise name its message as well as its type?](https://github.com/kalonji-tools/purba/issues/1021#issuecomment-6919967513) weighed it. It let a test pass on another raise, as the table counts.
- **A test that expects a raise always gives a `match=`.** The person said on the same issue that "raises should always match". It failed no test that a check of the caught value let pass. It also refuses an exact check of the message.
- **A test gives a `match=` for a broad type only.** The same issue weighed it. It leaves unchecked a raise of a type that the project raises at two or more places, as the table counts.
- **A test expects a raise by `with raises(…)`.** [The prototype of the form](https://github.com/kalonji-tools/purba/blob/107b09a66bcdc7a4a8cba9cdd421beb96608e287/prototypes/1015-form/FORM.md?plain=1#L84) holds it. [A name that a test uses joins with a milestone](purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome), and `main` has no such name yet.
- **The fix names `except SomeError as e:`, and a `try` of one path still refuses `as`.** [Must a test that expects a raise name its message as well as its type?](https://github.com/kalonji-tools/purba/issues/1021#issuecomment-6919967513) chose the words. On `main` at `ee8ac272`, a tester who writes that fix gets `[second-path]` when the statement does not raise, and the line names the shape `one except SomeError:`.
- **Selection refuses an `except` that only passes in a `try` of one path alone.** [Refuse a try whose except only passes, since it expects a raise by its type alone](https://github.com/kalonji-tools/purba/issues/1034) asked for it as filed. A test would still pass with an `except` that names a tuple, a bare `except:`, or a `try` of two statements.
- **Only `pass` counts as an `except` that only passes.** The same issue named `pass` alone. `...` and a string literal do no more than `pass`.

## Decision Outcome

**Reach:** `crates/selection/**` `crates/assertion/**` `tests/expect_raise.bats` `tests/expect_raise/**` `tests/rules.bats` `tests/rules/**`

**Area:** selection, strictness

**A [test](../../CONTEXT.md#test) that expects a raise names more than its type, so selection refuses an `except` that only passes as `type-alone`.**
`type-alone` is a check of the [practice tier](../../CONTEXT.md#practice-tier).
A tester who departs from that names the departure in a [suppression](../../CONTEXT.md#suppression) with a reason, and the test runs.
The refused line names `except SomeError as e:` as its fix.
For that reason, a `try` of one path admits `as`, as [a test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) says.

- **An `except` only passes when each statement of its body is `pass`, `...` or a `str` literal.** A comment is no statement.
- **Selection reads each `try` in the test's body, in a block at any depth.** A `try` of any shape counts. It can hold two statements, a tuple of types, a bare `except:`, an `as`, an `else`, a `finally` or an `except*`. Selection does not read a nested `def` or `class`, nor a helper.
- **Each such `except` gives one reason at its own line, in line order among the other reasons of the test.**

| the `except` | its refused line |
|---|---|
| `except Closed:` or `except Closed as e:` | `[type-alone] line 17 expects Closed by its type alone; bind except Closed as e: and check str(e)` |
| `except (Closed, KeyError):`, or `except Closed, KeyError:` on Python 3.14 | `[type-alone] line 35 expects (Closed, KeyError) by its type alone; bind except (Closed, KeyError) as e: and check str(e)` |
| `except:` | `[type-alone] line 44 expects any raise; bind except SomeError as e: and check str(e)` |
| `except* Closed:` | `[type-alone] line 36 expects Closed by its type alone; bind except* Closed as e: and check str(e)` |

- **`with raises(…)` and `with warns(…)` keep this rule when they join the public surface.** A `match=` or a check of the caught value then names more than the type, as [Must a test that expects a raise name its message as well as its type?](https://github.com/kalonji-tools/purba/issues/1021#issuecomment-6919967513) decides.

**Downside:**

- **An `except` that records only that it ran, such as `seen.append(1)`, still checks the type alone.** Selection reads no fault in its text.
- **Selection does not refuse an `except` whose body is a bytes literal or an f-string.** A bytes literal does no more than `pass`.
- **The fix of a tuple, of an `except*` or of a `try` of two statements makes no `try` of one path.** When its statement does not raise, execution refuses the case as `[second-path]`.
- **Selection does not refuse an `except` that only passes in a nested `def` or `class`, or in a helper.**
- **Selection also refuses an `except` that guards against an old Python and expects no raise.** six's `test_MAXSIZE` holds one.

## Confirmation

`mise run test:rust` runs `crates/selection/tests/select.rs`.
It checks that `pass`, `...` and a string literal each give a reason, and that a comment is no statement.
It checks a tuple, a bare `except:`, an `as`, a `try` of two statements, a second `except`, an `except*` and a `try` in a block.
It checks that an `else: pass`, a `finally: pass`, an `except` that does more and a `try` in a nested `def` give no reason.
It checks that a tuple with no parentheses gains them in its fix.
It checks that a suppression of `type-alone` lets the test run, with the `except` in its warning.

`crates/assertion/tests/rewrite.rs` checks that the statement of a `try` of one path with `as` calls the hook of a missed `try`.

`mise run test:wheel` runs `tests/expect_raise.bats`.
It checks each refused line and a suppressed `except` on the installed purba.
It checks a `try` of one path with `as` whose statement raises, and one whose statement does not.
`tests/rules.bats` checks that a `try` of one path with `as` fails when its statement does not raise.

[A test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) holds the `try` of one path.
