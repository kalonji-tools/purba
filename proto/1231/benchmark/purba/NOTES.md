# Port notes

Every input and expected value is the original's. The 21 pytest cases became 24 purba cases: the dateutil test split in four.

## Where the port differs, and why

- **Fixtures are helpers.** `writer` became `make_writer()`. `reset_logger` became a helper that each test calls first. There is no teardown reset: each test resets before it acts. (TESTS.md: no fixtures.)
- **Each parametrize case is its own test**, such as `test_format_a`…`_f` and `test_invalid_markups_red`. A loop would be one case, and `[computed-expected]` refuses an expected value that is an input. (TESTS.md: no parametrize.)
- **`pytest.raises` became `add_error()`.** It returns the caught exception or `None`. Then `isinstance`, plus `re.search(pattern, str(error)) is not None` for `match=`, with the pattern unchanged. (TESTS.md: no other test framework.)
- **dateutil: four tests, each tagged `@purba.tag("smoke")`.** The original asserts between acts (`[assert-before-expectation] line 9`). Each state is a test, with its setup replayed. (TESTS.md, expectations come last.)
- **`assert ymd.has_month` became `assert ymd.has_month is True`**, and likewise each positive check. The values are `bool`, so the check is the same. (TESTS.md, `[not-one-comparison]`.)

## Rules missing, wrong or unclear

1. **`[computed-expected]` is not in any rule.** The first run printed: `[computed-expected] line 20 compares to the input expected, and an expected value is a literal until purba has cases`.
2. **`[second-path]` is not in any rule.** It blocked `add_error` while that helper had a separate `return None`: `[second-path] line 148 reaches line 51 of add_error, which did not run, and the lines before the first expectation take one path`. It is unclear:
   - It looks only at calls before the first expectation. `assert isinstance(add_error(-1, TypeError), TypeError)` passed with the same unrun line.
   - The same logic written as `error = None … error = caught; return error` passes. The check counts lines run, not paths.
3. **The private-name warning is not in any rule:** `⚠ line 12 reads the private name dateutil.parser._parser._ymd`. It changed neither the count nor the exit code.
4. **"blocked" is missing from TESTS.md's list of case lines.** A blocked case prints `project error: …`, but the count prints `1 blocked; 1 project error`. Only RERUN.md shows "blocked".
5. **TESTS.md says `assert ymd.has_month` is refused.** In the original, only the closing assert (line 30) got `[not-one-comparison]`; line 9 got `[assert-before-expectation]`.

## Surprises

`purba --help` is read as a query: "`--help` picks no test; the nearest id is `tests`".
