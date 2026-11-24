# The query

`purba <query>` runs the tests that the query names. Quote a query that holds a space, `|`, `&`, `!` or a parenthesis, so the shell passes it whole.

## What one term names

| term | names |
|---|---|
| `tests` | every test under the folder `tests` |
| `tests/test_add_option_format.py` | every test in that file |
| `tests/test_add_option_format.py::test_format` | that test |
| `tag(smoke)` | each test that carries `@purba.tag("smoke")` (`TAG.md`) |
| `name(format)` | each test whose name holds `format` |
| `path(tests/**)` | each test in a file that the glob matches |

A path is written from the folder where purba starts, with `/`. An id matches only whole parts: `tests/test_add` names no file.

## Combining terms

`!` (not) binds tightest, then `&` (and), then `|` (or). Parentheses group.

```sh
purba 'tag(smoke) & !tests/test_internals.py'
purba 'tests/test_a.py::test_one | tests/test_b.py::test_two'
```

## A query that picks nothing

It is an error, exit 3, and no test runs. purba names the nearest id, tag or name it knows:

```text
purba: invocation error: the query selects no test of the 1 tests in the 1 files that purba parsed
  `tag(smok)` picks no test; the nearest tag is `smoke`
```
