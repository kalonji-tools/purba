# A tag on a test

A tag is a word that a tester puts on a test, so that a query can pick the tests that carry it.

```python
import purba


@purba.tag("smoke")
def test_parse():
    assert int("7") == 7


@purba.tag("slow")
@purba.tag("db")
def test_migrate():
    assert 3 * 3 == 9
```

- Write `@purba.tag("<word>")` above the `def`, with `import purba` at the top of the file.
- The word is a string literal. purba reads it from the text of the file, without importing it. A name or an expression in its place, such as `@purba.tag(SLOW)`, refuses the test: "[foreign-decorator] the decorator purba.tag is not purba's, and purba cannot see what it does".
- A test carries several tags with one decorator each.
- A tag needs no declaration anywhere else.
- A tag changes nothing in how the test runs. It only lets a query pick it.

## Picking tests by tag

```sh
purba 'tag(smoke)'
purba 'tag(slow) & !tag(db)'
purba 'tag(smoke) | tests/test_internals.py'
```

`QUERY.md` holds the rest of the query. A tag that no test carries is an error, exit 3, and purba names the nearest tag:

```text
purba: invocation error: the query selects no test of the 1 tests in the 1 files that purba parsed
  `tag(smok)` picks no test; the nearest tag is `smoke`
```
