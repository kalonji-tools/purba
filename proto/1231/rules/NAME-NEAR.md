# A name near the prefix `test`

purba runs a function as a test only when its name starts with `test` (`TESTS.md`). A function whose name is close to that prefix, but does not start with it, is probably a test with a typo in its name. purba does not run it, and says so:

```text
purba: name-near: tests/test_lend.py:13: tset_checkout is not a test, and its name is near the prefix test
```

- The line goes to stderr, before the first case, and names the file, the line of the `def`, and the function.
- It fires for a function at the top level of a test file whose name does not start with `test` or `_`, and whose first word, up to the first `_`, is within one edit of `test`, case ignored: one letter changed, added or dropped, or two neighbours swapped. So `tset_checkout`, `tst_checkout`, `Test_checkout` and `teSt_checkout` each give one. `checkout_test` and `_tset_checkout` give none, and `tests_checkout` is a test.
- It changes no count and no exit code, and it is printed on every run of a query that reads the file.
- A method in a class is not checked.
