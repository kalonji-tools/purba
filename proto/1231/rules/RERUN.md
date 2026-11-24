# Running the failures again

When a case of a run fails or raises, purba prints one more line on stderr after the count: the failures, as one query.

```text
tests/test_a.py::test_fails failed
    assert 1 == 2
      expected  2
      reading   1
tests/test_a.py::test_raises raised: KeyError: 'k'
6 cases: 2 passed, 1 failed, 1 raised, 2 blocked; 2 project errors
purba: the failures as one query: tests/test_a.py::test_fails | tests/test_a.py::test_raises
```

To run them again, copy the query, in quotes:

```sh
purba 'tests/test_a.py::test_fails | tests/test_a.py::test_raises'
```

- The query holds each case that ran and did not pass: failed, raised, or blocked as it ran, such as "[no-expectation]" or "[assert-before-expectation]". It also holds each test of a file that did not load.
- A test that purba refused before the run is not in it: one with a parameter, a method, a decorator that is not purba's, or "[not-one-comparison]".
- A run in which every case passed prints no such line.
- purba keeps nothing between runs: the line on screen is the only record of the failures.
