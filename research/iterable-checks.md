# Which checks of an iterable do test tools offer, and what does each show when it fails?

This file answers [#1023](https://github.com/kalonji-tools/purba/issues/1023).
It is never merged.
It names no shape, API or rule for purba; it reports what the tools do.
Every source was read on 2026-11-16, at the version or commit its row in the [source register](#source-register) gives.
Each claim carries a strength mark: **[A]** means I read the primary source (code, the tool's docs or the tool's own tests) or ran the tool myself; **[B]** means a secondary source or my inference.
Probe output is quoted verbatim, with machine paths and addresses cut; the [probe appendix](#appendix-the-probes) says how each probe ran.

## Answer

The tools agree on the easy part and split on the four places #1016 found weak.
Every tool has an ordered-equality check, and every assertion library has a "contains this item" check.
For an ordered list, most tools name the first index that differs or print a diff; Rust's `assert_eq!` and Chai print both values and leave the reader to compare.
Past that, the tools differ in what they check and in what they print.

**A property of each item.**
Matcher libraries have it: Hamcrest (`everyItem`, `only_contains`), AssertJ (`allMatch`, `allSatisfy`), GoogleTest (`Each`) and googletest-rust (`each`).
Plain-assert runners do not: pytest, unittest, Rust's `assert!`, node:assert, Chai and testify have no "each item" check.
pytest's plain `assert all(...)` shows only `where False = all(<generator object ...>)`.
pytest rewrote `all()` into a loop in 4.6.0 and reverted it two releases later, because the rewrite broke nested comprehensions, `if` filters and `all()` inside a larger expression.
The issue that asked for it is still open, seven years on.
The runners that lack the check offer a per-item loop instead: `subTest` in unittest, the `subtests` fixture in pytest 9, `assertAll` in JUnit.
When an "each" check fails, tools disagree on how much they name.
googletest-rust and AssertJ list every failing item; GoogleTest (C++) and Java Hamcrest name the first; PyHamcrest and Jest's `arrayOf` name none and print the whole collection.

**A lazy product.**
No tool I read checks a lazy product while it is produced and reports where it broke.
If the code under test raises at item 3, every probed Python tool lets the exception escape; none names the index.
Many checks turn the iterator into a list first: unittest's `assertCountEqual`, AssertJ's `toIterable()` and `Stream` support, Jest's `toContain`, Chai's `members`.
Some refuse it: unittest's `assertSequenceEqual`, Jest's `toHaveLength`, testify, and googletest-rust (at compile time).
Some compare the iterator object itself and get a wrong answer.
pytest compares a generator to a list by identity, so the check always fails.
node:assert's `deepStrictEqual` passed two generators that yield different items.

**An iterator that one check empties.**
In Python and JavaScript a second check sees an empty or shortened iterator, and its failure does not say why.
Worse, two matcher libraries (PyHamcrest and, by the same design, Java Hamcrest) iterate twice inside one check: once to decide, once to describe.
On a generator, PyHamcrest then prints a false reason: `but: No item matched: <1>` for a list whose real fault was item 2.
Rust is the one language where the compiler stops a second use: `itertools::assert_equal` takes the iterator by value, so a second check is error E0382.
googletest-rust refuses an iterator outright, because its `Matcher` trait needs `Copy` so it can read the value twice.

**Items in any order that can be neither hashed nor sorted.**
Most "any order" checks need only equality, not a hash or an order: unittest's `assertCountEqual`, AssertJ's `containsExactlyInAnyOrder`, Chai's `members`, testify's `ElementsMatch`, Hamcrest's `containsInAnyOrder`.
They pair items by a quadratic search.
That search is greedy, and greedy pairing is exact for equality but wrong for overlapping matchers.
PyHamcrest fails `[1, 2]` against `(greater_than(0), equal_to(1))`, and Java Hamcrest documents the same trap.
GoogleTest (C++ and Rust) runs a maximum bipartite matching instead; googletest-rust passes that case.
AssertJ's `satisfiesExactlyInAnyOrder` searches every assignment, so by its code it would pass too (not probed).
unittest's own history shows the sort trap: `assertItemsEqual` once sorted first, gave wrong answers for sets (a partial order), and was rewritten to count and renamed `assertCountEqual` in 3.2.
GoogleTest's `WhenSorted` still sorts with `<`.

**What a failure shows.**
Four patterns recur: the first differing index (pytest, unittest, JUnit, itertools, Hamcrest); every differing index (AssertJ `containsExactly`, googletest-rust `each`); the items missing and the items extra, with no position (pytest sets, unittest sets and counts, AssertJ, testify, GoogleTest); and a full textual diff (Jest, node:assert, pytest `-v`, pretty_assertions).
Chai prints both collections and names no item.
Three tools print an empty value for a generator: Chai prints `expected  to have the same members`, Jest prints `Received: {}`, Vitest prints `expected  to deeply equal`.

## 1. The tools, and why I chose them

The issue asks for Python, Rust and at least two other languages in wide use, covering both matcher libraries and the checks that runners ship.
I took the most used runner and the most used assertion library per language, plus a matcher library where one exists, because matchers are where collection checks live.
The usage numbers come from ecosyste.ms (one source, [B]) and, for C++, from JetBrains' 2025 survey (one source, [B]).
They justify the pick; they are not a ranking.

| language | tool | kind | why this one (usage, read 2026-11-16) |
|---|---|---|---|
| Python | pytest 9.2.0 | runner, plain `assert` rewriting | 950 M downloads last month; 283 k dependent repositories |
| Python | unittest (CPython 3.15.2) | runner in the standard library, `assert*` methods | ships with Python |
| Python | PyHamcrest 2.1.0 | matcher library | 2.6 M downloads last month; the most used Python matcher library I found (assertpy: 0.98 M) |
| Rust | `assert_eq!` / `assert!` (Rust 1.100.0) | standard macros | every Rust test |
| Rust | itertools 0.15.0 `assert_equal` | the only iterator-aware assertion in a top crate | 1.8 G total downloads; 35 k dependent repositories |
| Rust | googletest-rust 0.14.4 | matcher library | small (8.8 M total downloads, 7 dependent packages), but the only Rust port of a full container-matcher set |
| Rust | pretty_assertions 1.4.1 | diff output for `assert_eq!` | 239 M total downloads |
| Java | JUnit Jupiter 6.1.3 | runner and `Assertions` | 124 k dependent repositories (junit 4: 1.6 M) |
| Java | AssertJ 3.27.7 | fluent assertion library | 80 k dependent repositories |
| Java | Java Hamcrest 3.0 | matcher library (the original) | 10 k (`hamcrest`) plus 42 k (`hamcrest-library`) dependent repositories |
| JavaScript | Jest 30.5.2 (`expect`) | runner and matchers | 236 M downloads last month; 2.1 M dependent repositories |
| JavaScript | Chai 6.3.0 | assertion library | 783 M downloads last month |
| JavaScript | node:assert (Node 24.22.1) | runtime's assertion module | ships with Node; `node:test` uses it |
| JavaScript | Vitest 5.1.0 | runner, Jest-style `expect` | 687 M downloads last month; probed for lazy behaviour only |
| Go | testify 1.12.1 | assertion library | 280 k dependent repositories; Go's `testing` package has no assertions |
| C++ | GoogleTest 1.18.0 (gMock matchers) | runner and matchers | used by 31% of 1,800 C++ developers, the top framework (next: CppUnit 8%) |

I left out property-based tools: they generate inputs, and their checks are the host tool's checks.

## 2. The tools one by one

Each table uses the tool's own names and grouping.
"Lazy" says what the check does with a lazy or one-shot iterator.

### Python: pytest (plain `assert`)

pytest has no collection checks of its own.
It rewrites `assert` and, on failure, adds an explanation for some operand types ([A] PT1).
Its docs list three: long strings, long sequences ("first failing indices") and dicts ([A] PT2).

| what you write | what pytest explains | lazy | failure output (probe, pytest 9.0.3) |
|---|---|---|---|
| `a == b`, both sequences | first differing index, then the extra items | a generator is not a `Sequence`, so this branch is skipped | `At index 2 diff: 4 != 3` then `Use -v to get more diff` [A] |
| `a == b`, both sets | extra items on each side | not applicable | `Extra items in the left set:` / `3` / `Extra items in the right set:` / `4` [A] |
| `a <= b` etc., sets | the one-sided difference | not applicable | `Extra items in the left set:` / `5` [A] |
| `a == b`, any two iterables | a full `ndiff` of the pretty-printed values, only with `-v` | prints the generator's repr; does not consume it | `assert <generator object gen at 0x…> == [1, 2, 3]`, and under `-vv`: `+ <generator object gen at 0x…>` `- [` `-     1,` … [A] |
| `x in c` | nothing beyond the operands | `in` consumes a generator up to the match | `assert 3 in [1, 2]`; after two earlier checks: `assert 1 in <generator object gen at 0x…>` [A] |
| `all(...)` / `any(...)` | the call's result only | `all` stops at the first false item | `assert False` / `+  where False = all(<generator object test_pt_all_gen.<locals>.<genexpr> at 0x…>)` [A] |
| `len(c) == n` | the call's result | not applicable | `assert 3 == 2` / `+  where 3 = len([1, 2, 3])` [A] |

Sources: `_compare_any.py` picks the explanation by type, and its last branch runs for any two non-string iterables ([A] PT1, lines 81-84).
`_compare_sequence.py` yields `At index {i} diff:` and breaks at the first difference ([A] PT1, lines 46-96).
The `all()` output above matches the output quoted in pytest issue #5062 in 2019 ([A] PT4), so it has not changed.
The probe also shows that `sorted()` of dicts raises `TypeError: '<' not supported between instances of 'dict' and 'dict'` before any comparison; that is Python, not pytest.

For "each item" pytest 9.0 added a `subtests` fixture ([A] PT2, `how-to/subtests.rst`; changelog entry #1367).
Each failing iteration is reported on its own: `SUBFAILED[custom message] (i=1)` and `SUBFAILED[custom message] (i=3)`, then `contains 2 failed subtests` ([A] PT2, quoted from the docs, not probed).

### Python: unittest

| check | asserts | lazy | failure output |
|---|---|---|---|
| `assertSequenceEqual`, `assertListEqual`, `assertTupleEqual` | same items, same order | refuses anything without `len()` | `Lists differ: [1, 2, 4, 5] != [1, 2, 3]` / `First differing element 2:` / `4` / `3` / `First list contains 1 additional elements.` / `First extra element 3:` / `5`, then an `ndiff` [A] probe; on a generator: `First sequence has no length.    Non-sequence?` [A] probe |
| `assertCountEqual` | same items, same counts, any order | materialises both with `list()`; accepts iterators | `Element counts were not equal:` / `First has 2, Second has 1:  {'a': 1}` / `First has 0, Second has 1:  {'b': 3}` [A] probe |
| `assertSetEqual` | set equality | needs `.difference` | `Items in the first set but not the second:` / `1` / `Items in the second set but not the first:` / `3` [A] probe |
| `assertIn`, `assertNotIn` | membership | `in` consumes an iterator | `3 not found in <generator object gen at 0x…>` [A] probe |
| `assertDictEqual` | dict equality | not applicable | `d1 != d2` plus an `ndiff` [A] UT1 |
| `subTest` | not a check: a context that reports each failing iteration on its own | runs the loop you write | `FAIL: test_even (...) (i=1)`, `(i=3)`, `(i=5)`, one block each [A] UT2 |

`assertCountEqual` works on items that are neither hashable nor orderable.
It tries a `Counter`; on `TypeError` it falls back to `_count_diff_all_purpose`, which pairs items by `==` in quadratic time ([A] UT1, `case.py` 1250-1280, `util.py` 138-170).
The docs say the same: it "works with sequences of unhashable objects as well" ([A] UT2).
CPython's own test checks `iter([1, 2, [], 3, 4])` against itself ([A] UT3, `testAssertCountEqual`).
unittest has no "each item matches" and no "some item matches" check; `subTest` is its answer to the loop.

### Python: PyHamcrest

| check | asserts | lazy | failure output |
|---|---|---|---|
| `contains_exactly(*m)` (`contains` is deprecated for it) | items match the matchers, in order, same count | iterates the value; does not copy it | `but: item 2: was <4>`; extra: `Not matched: <4>`; short: `No item matched: <3>` [A] PH3 tests, probe |
| `contains_inanyorder(*m)` | a one-to-one pairing in any order | calls `list(item)` | `but: not matched: <{'a': 1}>`; missing: `no item matches: <4> in [<1>, <2>, <3>]` [A] PH3, probe |
| `has_item(m)` | some item matches | iterates; stops at a match | `but: was <[1, 2]>` [A] probe |
| `has_items(*m)` | each matcher matches some item | calls `list(item)` to decide | see the one-shot trap below |
| `only_contains(*m)` | every item matches one of the matchers | calls `list(item)` | `but: was <[1, 2, -3]>` (no item named) [A] probe |
| `empty()`, `has_length(n)` | size | needs `len()` | `but: was <[1]>`; on a generator: `but: was <generator object gen at 0x…>` [A] probe |
| `is_in(seq)` | the value is a member | not applicable | not probed |

PyHamcrest has no "every item matches one matcher" check under the Java name `everyItem`; `only_contains` is the closest ([A] PH1, the module list).

**The one-shot trap.**
`assert_that` calls `matcher.matches(actual)`, and on failure calls `matcher.describe_mismatch(actual, …)` ([A] PH2, `assert_that.py` 65-73).
Several sequence matchers implement `describe_mismatch` by running `matches` again on the same value ([A] PH1, `issequence_containinginorder.py` 74-75).
On a generator the second run sees an empty iterator, so the reason is false:

```text
assert_that([1, 2, 4], contains_exactly(1, 2, 3))   ->  but: item 2: was <4>
assert_that(gen(1, 2, 4), contains_exactly(1, 2, 3)) ->  but: No item matched: <1>
assert_that(gen({"a": 1}, {"c": 3}), contains_inanyorder({"c": 3}, {"b": 2}))
                                                      ->  but: no item matches: <{'c': 3}>, <{'b': 2}> in []
assert_that(gen(1, 2), has_items(1, 7))              ->  but: a sequence containing <1> was <generator object gen at 0x…>
```

[A] probe.
The last line blames `1`, which is present; `7` is the missing one.
PyHamcrest's own tests run every sequence matcher on a generator form too ([A] PH3, `sequencemixin.py`), but their helper passes the description into one `matches` call, so the second pass never happens there ([A] PH3, `matcher_test.py` 66-70).
A comment in the source names the hazard for `has_items`: "we can't be sure we won't be seeing a one-time sequence here (like a generator); see issue #20" ([A] PH1, `issequence_containing.py` 35-38).

**The greedy trap.**
`contains_inanyorder` tries the matchers left to right and drops a matcher once it matches ([A] PH1, docstring, lines 82-97).
`assert_that([1, 2], contains_inanyorder(greater_than(0), equal_to(1)))` fails with `but: not matched: <2>`, though a valid pairing exists ([A] probe).
Issue #118 reports the same with `anything()`; it is open since 2019, and a maintainer replied that "overgeneral matchers can _consume_ items" and that the matcher "is working as documented" ([A] PH4).

### Rust: `assert_eq!`, `assert!`, pretty_assertions

| check | asserts | lazy | failure output |
|---|---|---|---|
| `assert_eq!(a, b)` | `a == b` | adapters such as `Map` and `Filter` have no `PartialEq`, so you collect first [B] | ``assertion `left == right` failed`` / `  left: [1, 2, 4]` / ` right: [1, 2, 3]` [A] RS1 `panicking.rs` 434-458, probe |
| `assert!(it.all(f))`, `assert!(v.contains(&x))` | a boolean | `all` and `any` stop early and consume | `assertion failed: v.iter().all(\|x\| x % 2 == 0)` (the expression only) [A] probe |
| `pretty_assertions::assert_eq!` | `a == b` | as `assert_eq!` | a coloured line diff: `Diff < left / right >` then `<    4,` / `>    3,` [A] probe |

RFC 2011, which would make `assert!` print its operands, has been an open tracking issue since 2017 ([A] RS2).

### Rust: itertools `assert_equal`

`assert_equal(a, b)` takes two `IntoIterator`s by value and walks them together ([A] IT1, `lib.rs` 5237-5278).
It panics at the first difference and names the iteration: `Failed assertion Some(6) == Some(7) for iteration 2` ([A] probe).
A length difference shows as `None`: `Failed assertion None == Some(3) for iteration 2` ([A] probe).
It does not print either sequence.

Because the iterator moves into the call, a second check on it does not compile ([A] probe):

```text
error[E0382]: use of moved value: `it`
3 |     itertools::assert_equal(it, vec![2, 4, 6]);
  |                             -- value moved here
4 |     itertools::assert_equal(it, vec![2, 4, 6]);
  |                             ^^ value used here after move
```

The compiler suggests `&mut it`, and `it.by_ref()` also borrows.
With `by_ref()`, an earlier check can consume part of the iterator and the later check sees only the rest: `assert!(it.by_ref().any(|x| x == 2))`, then `assert_equal(it, …)` fails with `Failed assertion Some(4) == Some(2) for iteration 0` ([A] probe).

### Rust: googletest-rust

Container matchers ([A] GR1, `matchers/mod.rs`):

| check | asserts | failure output (probe) |
|---|---|---|
| `elements_are![m0, m1, …]` | items match, in order | `Expected: has elements:` / `0. is equal to 1` … / `Actual: [1, 4, 3],` / `where element #1 is 4, which isn't equal to 2` |
| `unordered_elements_are![…]` | one-to-one pairing, any order | `whose element #1 does not match any expected elements and no elements match the expected element #1` [A] GR2 test; items that match no matcher: `whose elements #1, #2 do not match any expected elements` |
| `contains_each![…]` | each matcher pairs with a distinct item | `which has no element matching the expected element #1` |
| `is_contained_in![…]` | each item pairs with a distinct matcher | not probed |
| `each(m)` | every item matches | `whose elements #1, #3 don't match` / `-2, which is less than or equal to 0` / `-4, which is less than or equal to 0` |
| `contains(m)`, `contains(m).times(n)` | some item matches; exactly n match | `which does not contain a matching element`; `Expected: contains n elements which is equal to 2` / `where n is equal to 1` / `which contains 2 matching elements` |
| `not(contains(m))` | no item matches | `Expected: contains no element which is equal to 2` / `which contains a matching element` |
| `superset_of(c)`, `subset_of(c)` | set relations | `whose element 5 is missing` |
| `len(m)`, `is_empty()` | size | `which has length 3, which isn't equal to 2` |
| `pointwise!(m, c)` | item i of each pairs under m | `where element #0 is 1, which isn't equal to 2` [A] GR2 test |
| `container_eq(c)` | equality with a set-style explanation | `which contains all the elements` when only the order differs [A] GR1 test |
| nested, e.g. `each(each(m))` | recursion | `whose element #1 is [3], whose element #0 is 3, which is greater than or equal to 3` |

**Lazy.** The `Matcher` trait is `Matcher<ActualT: Debug + Copy>`.
A source comment gives the reason: "`ActualT` requires `Copy` so that `actual` could be passed to `matches` and if it fails passed to `explain_match`" ([A] GR1, `matcher.rs` 35-38).
So a matcher reads the value twice, and the type system keeps that safe.
An iterator is not `Copy`, and `verify_that!` borrows its subject, so an iterator is a compile error: `` `&Map<Range<{integer}>, …>` is not an iterator `` ([A] probe).
Versions 0.10 and 0.11 said so in prose: "This matcher does not support matching directly against an [`Iterator`]. To match against an iterator, use [`Iterator::collect`] to build a [`Vec`]" ([A] GR3); 0.12.0 changed the trait to this form and the sentence went ([A] GR4, release notes).

**Any order.** `unordered_elements_are!` runs "the well-known Ford-Fulkerson max flow method to find a maximum bipartite matching" ([A] GR1, `match_matrix.rs` 175).
`unordered_elements_are![gt(&0), eq(&1)]` passes on `[1, 2]`, the case Hamcrest fails ([A] probe).
It works on a struct with neither `Hash` nor `Ord` ([A] probe, `P { x: f64 }`).
The docs describe three stages and promise "the best matching it could find" on the third ([A] GR1, lines 85-101).
In my probe `[3, 2, 1]` against three `ge(&3)`, the items 2 and 1 match no matcher, so the check stops at the second stage and names them; I did not probe a case that reaches the third.

### Java: JUnit Jupiter

| check | asserts | lazy | failure output |
|---|---|---|---|
| `assertIterableEquals(expected, actual)` | same items, same order, recursively into nested iterables | calls `iterator()` on each; types need not match | `iterable contents differ at index [2], expected: <3> but was: <9>`; nested: `at index [2][1][1][0]`; length: `iterable lengths differ, expected: <3> but was: <4>` [A] JU1, JU2 |
| `assertArrayEquals` | arrays, deeply | not applicable | not read in detail |
| `assertLinesMatch(expected, actual)` | lines of text, each equal or a regex, with `>>` fast-forward markers | `List` or `Stream` of strings | not read in detail [A] JU3 javadoc |
| `assertAll(Stream<Executable>)` | runs every executable and reports all failures | not read in detail | not probed [A] JU3 javadoc |

JUnit has no "contains", "any order" or "each item" check of its own.
Its user guide says that when more is needed, "the JUnit team recommends the use of third-party assertion libraries such as {AssertJ}, {Hamcrest}, {Truth}, etc." ([A] JU4).

### Java: AssertJ

`AbstractIterableAssert` has about 60 iterable checks ([A] AJ1).
Grouped as the method names group them:

| group | checks | failure output |
|---|---|---|
| exact items | `containsExactly`, `containsExactlyElementsOf`, `isEqualTo` | `to contain exactly (and in same order):` … `but there were differences at these indexes:` / `  - element at index 1: expected "Luke" but was "Han"` / `  - element at index 2: expected "Han" but was "Luke"`; for size or content: `but some elements were not found:` … `and others were not expected:` [A] AJ2 test |
| any order | `containsExactlyInAnyOrder`, `containsExactlyInAnyOrderElementsOf`, `hasSameElementsAs`, `containsOnly`, `satisfiesExactlyInAnyOrder` | `elements not found:` … `and elements not expected:` [A] AJ1 `ShouldContainExactlyInAnyOrder`; `to satisfy all the consumers in any order.` (no item named) [A] AJ1 `ShouldSatisfy` |
| contains | `contains`, `containsAll`, `containsAnyOf`, `containsOnlyOnce`, `containsSequence`, `containsSubsequence`, `startsWith`, `endsWith`, `containsNull`, `isSubsetOf` | `to contain:` … `but could not find the following elements:`; subset: `but found these extra elements:` [A] AJ1 |
| does not contain | `doesNotContain`, `doesNotContainAnyElementsOf`, `doesNotContainSequence`, `doesNotContainSubsequence`, `doesNotHaveDuplicates`, `doesNotContainNull` | not read in detail |
| each item | `allMatch(p)`, `allSatisfy(consumer)`, `are(condition)`, `have(condition)` | `Expecting all elements of:` … `to satisfy given requirements, but these elements did not:` then each failing element and its error [A] AJ2 test; `allMatch`: `to match … predicate but these elements did not:` [A] AJ1 |
| some item | `anyMatch`, `anySatisfy`, `areAtLeastOne`, `haveAtLeastOne` | `to match … predicate but none did.`; `anySatisfy` lists each element's error [A] AJ1, AJ2 |
| no item | `noneMatch`, `noneSatisfy`, `areNot`, `doNotHave` | `to match … predicate but this element did:` [A] AJ1 |
| count | `hasSize`, `hasSizeBetween`, `hasSizeGreaterThan`…, `areExactly(n, c)`, `areAtLeast`, `areAtMost`, `satisfiesOnlyOnce`, `singleElement()` | `Expected size: 2 but was: 3 in:` [A] AJ1 |
| item by item | `satisfiesExactly(c0, c1, …)`, `zipSatisfy(other, f)`, `element(i)`, `first()`, `last()` | not read in detail |
| narrowing | `filteredOn`, `extracting`, `flatExtracting`, `map` | they return a new assertion on the narrowed list |

`allMatch` and `allSatisfy` collect every failing element, not the first ([A] AJ1, `Iterables.java` 1183-1193 and 1321-1332).
They name the elements but not their indexes.
`containsExactly` lists up to 50 differing indexes (`MAX_INDICES_FOR_PRINTING = 50`, [A] AJ1).
`containsExactlyInAnyOrder` needs only `equals`: it copies `actual` into a list and removes the first equal item per expected value ([A] AJ1, `Iterables.java` 1397-1414).
`satisfiesExactlyInAnyOrder` searches all assignments recursively, "whether we can find any specific matching permutation" ([A] AJ1, lines 1255-1265).
`hasOnlyOneElementSatisfying` is deprecated for `singleElement()` ([A] AJ1, line 328).

**Lazy.** AssertJ documents both one-shot types.
`assertThat(Iterator)` returns `IteratorAssert`, whose checks "do not consume any elements"; there are only `hasNext` and `isExhausted`, and `toIterable()` copies the rest into a `List` for the iterable checks ([A] AJ1, `AbstractIteratorAssert.java` 26-29 and 98-100; `IterableAssert.java` 55-56).
`assertThat(Stream)` says: "the `Stream` under test will be converted to a `List` when an assertion requires to inspect its content. Once this is done the `Stream` can't be reused as it has already been consumed", and chained checks are safe because they use that list ([A] AJ1, `Assertions.java` 3597-3602).
A plain `Iterable` is not copied when the assertion is built (`IterableAssert` passes `actual` straight on, [A] AJ1), so a one-shot `Iterable` would be iterated by each chained check ([B], inferred, not probed).

### Java: Hamcrest

| check | asserts | failure output (from the tool's own tests) |
|---|---|---|
| `contains(m…)` | items match, in order | `item 2: value was <4>`; extra: `not matched: <4>`; short: `no item was value with <3>` [A] JH2 |
| `containsInAnyOrder(m…)` | one-to-one pairing, any order (greedy) | `not matched: <4>`; `no item matches: <4> in [<1>, <2>, <3>]` [A] JH2 |
| `containsInRelativeOrder(m…)` | a subsequence, in order | `value with <3> was not found after <WithValue 2>` [A] JH2 |
| `hasItem(m)`, `hasItems(m…)` | some item matches | `mismatches were: [mismatched: b, mismatched: c]`; empty: `was empty` [A] JH2 |
| `everyItem(m)` | every item matches | `an item was "BXB"` (first failing item, no index) [A] JH1 `Every.java` 15-24, JH2 |
| `iterableWithSize`, `emptyIterable`, `hasSize`, `empty` | size | not read in detail |
| `in(c)`, `isIn(c)`, `oneOf(…)` | the value is a member | not read in detail |

The javadoc of `containsInAnyOrder` warns: "each of the specified matchers will only be used once during a given examination, so be careful when specifying matchers that may be satisfied by more than one entry in an examined iterable" ([A] JH1, `IsIterableContainingInAnyOrder.java` 88-90).
That is the same greedy pairing as PyHamcrest, documented on the Java side.

`MatcherAssert.assertThat` calls `matches`, then `describeMismatch` on failure ([A] JH1, `MatcherAssert.java` 9-22).
`TypeSafeDiagnosingMatcher` runs `matchesSafely` in both ([A] JH1, lines 63-80), so a failing collection check iterates twice.
A Java `Iterable` normally returns a fresh iterator each time, so this is safe for collections; for a one-shot `Iterable` it would misreport as PyHamcrest does ([B], inferred, not probed).

### JavaScript: Jest (`expect`)

| check | asserts | lazy | failure output (probe, Jest 30.5.2) |
|---|---|---|---|
| `toEqual`, `toStrictEqual` | deep equality; any object with `Symbol.iterator` is compared by iteration | consumes generators | a line diff: `- Expected  - 1` / `+ Received  + 1` / `  Array [` … `-   3,` / `+   4,` [A] |
| `toContain(x)` | an item `===` x | `[...received]`, so it consumes | `Expected value: 3` / `Received array: [1, 2]`; on a generator: `Received object: {}` [A] |
| `toContainEqual(x)` | an item deep-equal to x | `[...received]` | `Expected value: {"a": 2}` / `Received array: [{"a": 1}]` [A] |
| `toHaveLength(n)` | `.length` | refuses a generator | `Expected length: 2` / `Received length: 3` / `Received array:  [1, 2, 3]`; generator: `Matcher error: received value must have a length property whose value must be a number` [A] |
| `expect.arrayContaining(a)` | a is a subset of the received array | arrays only | `Expected: ArrayContaining [1, 5]` / `Received: [1, 2]` (no item named) [A] |
| `expect.arrayOf(m)` (added in 30.0.0) | every item matches m | arrays only | `Expected: ArrayOf Any<String>` / `Received: [1, -2, 3]` (no item named) [A] |
| `not.` before any of them | negation | as above | not probed |

Sources: `toContain` builds `const indexable = [...received]` ([A] JS1, `matchers.ts` 536); `ArrayContaining` and `ArrayOf` accept only `Array.isArray(other)` ([A] JS1, `asymmetricMatchers.ts` 193-247); `arrayOf` entered in 30.0.0, PR #15567 ([A] JS1, `CHANGELOG.md` 376).
The docs describe `toContain` for arrays and strings only ([A] JS1, `docs/ExpectAPI.md` 578-580); the code accepts any iterable.

**Lazy, probed.**
`expect(gen(1, 2, 3)).toEqual([1, 2, 3])` fails with `Expected: [1, 2, 3]` / `Received: {}` ([A] probe).
Issue #8680 reported exactly this in 2019; a maintainer pointed to a third-party package and closed it ([A] JS2).
Two generators that differ do not fail with a diff: the matcher throws `TypeError: 'caller', 'callee', and 'arguments' properties may not be accessed on strict mode functions or the arguments objects for calls to them` ([A] probe).
Open issue #16541 (2026-11-07) reports the same TypeError for a failing generator comparison ([A] JS3).
Two equal generators pass, and the first check consumes them: `expect(g).toEqual(gen(1, 2))` twice throws the TypeError on the second ([A] probe).
`toContain` on a generator consumes it; a second `toContain(3)` after `toContain(2)` failed with `Received object: {}` ([A] probe).
Open issue #16508 (2026-10-19) reports that `arrayContaining`, `objectContaining` and `arrayOf` treat any `Set`, `Map` or custom iterable as matching any other of its kind ([A] JS3).

**Any order without a hash.**
`toEqual` on two `Set`s of objects compares them in any order by deep equality: `new Set([{a: 1}, {b: 2}])` equals `new Set([{b: 2}, {a: 1}])` ([A] probe).
That drops duplicates, as any `Set` does.
On failure it prints a diff that pairs items by position, not by match: `-     "b": 2,` / `+     "a": 1,` under the first `Object {` ([A] probe).

### JavaScript: Vitest (lazy behaviour only)

I probed Vitest only for the lazy cases; I did not read its source.
`expect(gen(1, 2, 3)).toEqual([1, 2, 3])` fails with `expected  to deeply equal [ 1, 2, 3 ]`, `+ Received:` `{}` ([A] probe).
Two different generators fail with `expected  to deeply equal` / `Compared values have no visual difference.` ([A] probe).
A second `toContain` on a consumed generator fails with `expected [] to include 3` ([A] probe).
`expect.arrayOf` does not exist in Vitest 5.1.0: `TypeError: expect.arrayOf is not a function` ([A] probe).

### JavaScript: Chai

| check | asserts | lazy | failure output (probe, Chai 6.3.0) |
|---|---|---|---|
| `.members(s)`, `.deep.members(s)` | same items, same counts, any order | `Array.from` on both; checks `.iterable` first (line 3280); `Set` support came in 5.1.0 | `expected [ { a: 1 }, { b: 2 } ] to have the same members as [ { b: 2 }, { a: 3 } ]` (no item named) [A] |
| `.ordered.members(s)` | same items, same order | as above | `expected [ 1, 2, 4 ] to have the same ordered members as [ 1, 2, 3 ]` [A] |
| `.include.members(s)` | superset | as above | `expected [ 1, 2 ] to be a superset of [ 1, 5 ]` [A] |
| `.include(x)` | membership in an array, string, `Set`, `Map` or object | refuses a generator | `expected [ 1, 2 ] to include 3`; generator: `the given combination of arguments (generator and number) is invalid for this assertion` [A] |
| `.lengthOf(n)` | length, or size for `Set` and `Map` | needs `length` or `size` | `expected [ 1, 2, 3 ] to have a length of 2 but got 3`; `expected Set{ 1 } to have a size of 2 but got 1` [A] |
| `.oneOf(list)` | the value is a member | not applicable | `expected 3 to be one of [ 1, 2 ]` [A] |
| `.iterable` (added 5.1.0) | the value has `Symbol.iterator` | does not iterate | `expected 5 to be an iterable` [A] |
| `.satisfy(f)` | a predicate on the whole value | not applicable | `expected [ 1, -2 ] to satisfy [Function]` [A] |

Chai has no "each item" or "some item" check.
`members` pairs by a greedy quadratic search with `===` or deep equality, so it needs no hash ([A] CH1, `assertions.js` 3176-3204).
**Lazy, probed.** A generator passes `members` once, because `Array.from` copies it.
The failure message prints the generator as nothing: `expected  to have the same members as [ 1, 3 ]` ([A] probe).
A second `members` check on the same generator fails the same way, with an empty actual ([A] probe).

### JavaScript: node:assert

| check | asserts | lazy | failure output (probe, Node 24.22.1) |
|---|---|---|---|
| `deepStrictEqual(a, b)` | deep equality; `Set` items and `Map` keys in any order | compares a generator as an object | `Expected values to be strictly deep-equal:` / `+ actual - expected` / `  [` … `+   4` / `-   3` [A] |
| `partialDeepStrictEqual(a, b)` (stable since 24.0.0) | only what `b` has; for arrays, an ordered subsequence | as above | `Expected values to be partially and strictly deep-equal:` then a diff [A] |
| `ok(v.includes(x))` | a boolean | not applicable | `The expression evaluated to a falsy value:` / `  assert.ok([1, 2].includes(3))` [A] |

**A false pass.** `assert.deepStrictEqual(gen(1, 2), gen(1, 3))` passed ([A] probe).
The docs explain why: "Only enumerable "own" properties are considered", and `[[Prototype]]`s are compared with `===` ([A] NO1, `assert.md` 883-889).
Two generator objects from one function have no own properties and one prototype, so they are equal whatever they yield ([B], inferred from the docs and the probe).
The docs give `[4, 5, 8]` as passing and `[5, 4, 8]` as failing `partialDeepStrictEqual` against `[1, …, 9]` ([A] NO1, lines 2740-2775).

### Go: testify

| check | asserts | lazy | failure output (probe, Go 1.26.10) |
|---|---|---|---|
| `Equal(t, expected, actual)` | `ObjectsAreEqual` | slices only | `Not equal:` / `expected: []int{1, 2, 4}` / `actual  : []int{1, 2, 3}` then a unified diff [A] |
| `Contains`, `NotContains` | membership in a string, slice, array or map | refuses `iter.Seq` | `[]int{1, 2} does not contain 3`; `[]int{1, 2} should not contain 2`; `iter.Seq`: `(iter.Seq[int])(0x…) could not be applied builtin len()` [A] |
| `ElementsMatch`, `NotElementsMatch` | same items, same counts, any order | refuses `iter.Seq` | `elements differ` / `extra elements in list A:` … / `extra elements in list B:` … / `listA:` … / `listB:` … (each in spew dump form) [A]; `iter.Seq`: `%!q(iter.Seq[int]=0x…) has an unsupported type func, expecting array or slice` [A] |
| `Subset`, `NotSubset` | every item of one is in the other | slices, arrays, maps | `[]int{1, 2} does not contain 5` (first missing item only) [A] |
| `Len`, `Empty`, `NotEmpty` | size | needs `len()` | `"[1 2 3]" should have 2 item(s), but has 3`; `Should be empty, but was [1]` [A] |
| `IsIncreasing`, `IsDecreasing`, `IsNonIncreasing`, `IsNonDecreasing` | order of a slice | slices | `"3" is not less than "2"` [A] |

testify has no "each item" or "some item matches" check.
`ElementsMatch` marks used indexes and pairs by `ObjectsAreEqual` in a quadratic loop, so a struct holding a slice (not comparable with `==`) works ([A] GO1, `assertions.go` 1174-1209; probe with `type P struct{ X []int }`).
The `%!q` in the `iter.Seq` message is a formatting fault: the code formats a function with `%q` ([A] GO1, line 1165).

**Lazy, contested.** PR #1685 (open since 2024-12) would add `iter.Seq` to `Contains` and `ElementsMatch` by copying the sequence into a slice ([A] GO2).
A collaborator objected: "`Contains` with iterators should be implemented in a lazy way and not consume the whole sequence before the checks", and gave an endless sequence as the case ([A] GO2).
The author replied that for an endless sequence "a `Contains` check cannot fail, it can only cause non-termination", using `NotContains` as the example ([A] GO2).
The PR is unmerged.

### C++: GoogleTest (gMock container matchers)

The docs list these, with `ContainerEq` as the "more informative" form of `Eq` ([A] GT1, `matchers.md` 144-195):
`BeginEndDistanceIs`, `ContainerEq`, `Contains(e)`, `Contains(e).Times(n)`, `Each(e)`, `ElementsAre`, `ElementsAreArray`, `IsEmpty`, `IsSubsetOf`, `IsSupersetOf`, `Pointwise(m, c)`, `SizeIs`, `UnorderedElementsAre`, `UnorderedElementsAreArray`, `UnorderedPointwise`, `WhenSorted(m)`, `WhenSortedBy(cmp, m)`.

Failure output, from GoogleTest's own tests ([A] GT2):

| check | output |
|---|---|
| `Each(m)` | `whose element #0 doesn't match, which is 9 less than 10` (first failing element) |
| `UnorderedElementsAre`, a matcher unused | `where the following matchers don't match any elements:` / `matcher #1: is equal to 2` |
| `UnorderedElementsAre`, an element unused | `where the following elements don't match any matchers:` / `element #1: 2` |
| `UnorderedElementsAre`, no perfect pairing | `where no permutation of the elements can satisfy all matchers, and the closest match is 2 of 3 matchers with the pairings:` then the pairs |
| `WhenSorted` | `which is { 1, 2 } when sorted, whose element #0 (1) isn't equal to 2` |
| `SizeIs` | `whose size 2 doesn't match` |
| `ContainerEq` | `which has these unexpected elements: …` / `and doesn't have these expected elements: …` [A] GT3, `gmock-matchers.h` 2580-2625 |

`UnorderedElementsAre` uses a maximum bipartite matching ([A] GT3, `gmock-matchers.cc` 62-124, `MaxBipartiteMatchState`).
`ContainerEq` finds unexpected and missing values by `find`, so a difference in order alone prints neither list ([A] GT3; [B] that the line is then empty, not probed).
`WhenSorted` sorts with `<`, so it needs an ordered type; `Pointwise` is the docs' answer for a type without `==` ([A] GT1, notes under the table).
**Lazy.** The matchers take the container as `const LhsContainer&` through `StlContainerView` and iterate it with `begin()` and `end()` ([A] GT3); a one-shot input range is not mentioned in the docs ([B]).
I did not compile a GoogleTest probe.

## 3. Across the tools

### Where they agree

| check | tools that offer it |
|---|---|
| same items, same order | all of them |
| a given item is present | every library except itertools; Rust std and node:assert only through a boolean (`assert!(v.contains(&x))`, `assert.ok(v.includes(x))`) |
| no given item is present | every library except itertools, often as a negation; Rust std and node:assert only through a boolean |
| size | every library except itertools; Rust std and node:assert only through a boolean |
| same items in any order, by equality only (no hash, no order) | unittest, AssertJ, Hamcrest, googletest-rust, GoogleTest, Chai, testify; Jest and node:assert only through `Set` (duplicates lost) |
| a nested collection reports a path | JUnit (`at index [2][1][1][0]`), googletest-rust (`whose element #1 is [3], whose element #0 is 3`) |

Every Python tool I probed lets an exception from a lazy product escape unchanged; I did not probe this in the other languages.

### Where they disagree

| question | answers |
|---|---|
| Is "each item matches" a check? | yes in matcher libraries (Hamcrest, AssertJ, GoogleTest, googletest-rust) and in Jest 30 (`arrayOf`); no in pytest, unittest, Rust std, Chai, node:assert, testify, JUnit |
| When "each" fails, what is named? | every failing item with its index: googletest-rust; every failing item, no index: AssertJ; the first failing item: GoogleTest C++ (with index), Java Hamcrest (no index); no item: PyHamcrest `only_contains`, Jest `arrayOf` |
| What happens to a one-shot iterator? | refused at compile time: googletest-rust; moved, so a second use does not compile: itertools; refused at run time: unittest `assertSequenceEqual`, Jest `toHaveLength`, Chai `include`, testify; copied once into a list: unittest `assertCountEqual`, AssertJ `toIterable()` and `Stream`, Jest `toContain`, Chai `members`, PyHamcrest `contains_inanyorder`; compared as an object: pytest `==` (always fails), node:assert (passes when it should fail), Jest and Vitest `toEqual` (`Received: {}`) |
| Does one check iterate twice? | yes, by design: PyHamcrest and Java Hamcrest (match, then describe); googletest-rust reads twice but its type demands `Copy` |
| How is "any order" paired? | greedy, first match wins: Hamcrest (Java and Python), Chai, testify, AssertJ `containsExactlyInAnyOrder`, unittest's fallback; maximum bipartite matching: GoogleTest C++ and Rust; full backtracking: AssertJ `satisfiesExactlyInAnyOrder`; by sorting: GoogleTest `WhenSorted` |
| Should `Contains` on an iterator stop at the first match? | testify's collaborator: yes, lazily; the PR author: a lazy `NotContains` on an endless sequence cannot end; unresolved |

Greedy pairing gives the right answer when the pairing test is an equivalence (plain equality), and can give the wrong answer when two matchers overlap ([B], my reasoning; the Hamcrest docs and issue #118 are the evidence that the wrong answer happens).

### Checks no tool offers

I found none of these in the tools above ([B]: an absence across the files I read, not a proof):

- a check that consumes a lazy product and, when the product raises, reports the index at which it raised;
- a warning that an iterator was already empty because an earlier check consumed it (Rust's move rule is the only guard, and it is the language's, not a tool's);
- a check that stops on an endless iterator for a negative claim (`NotContains`), which the testify thread says cannot exist;
- an "each item" check in any runner's plain assertion; the runners offer a reporting loop instead (`subTest`, pytest `subtests`, JUnit `assertAll`);
- an "any order" check by matchers, with maximum matching, outside GoogleTest (C++ and Rust) and AssertJ's `satisfiesExactlyInAnyOrder`;
- a count of matching items ("exactly n") outside GoogleTest, googletest-rust and AssertJ (`areExactly`, `satisfiesOnlyOnce`, `containsOnlyOnce`).

### The four weak places from #1016, side by side

| place | what the tools do |
|---|---|
| a property of each item | matcher libraries have a check; plain-assert runners have a loop with per-iteration reporting; pytest tried a rewrite of `all()` and reverted it |
| a lazy product | tools copy it into a list, refuse it, or compare the iterator object; none checks it while it runs |
| an iterator that one check empties | in Python and JavaScript the second check fails without saying why; a compile error in Rust; Hamcrest consumes it twice inside one failing check and prints a false reason |
| items in any order, neither hashable nor sortable | most tools pair by equality in quadratic time; sorting-based checks need an order; matcher-based checks are greedy except in GoogleTest and one AssertJ check |

## 4. History: checks added and removed, and rewrites of `all()` / `any()`

| when | tool | what happened | source |
|---|---|---|---|
| 2019-05-31 | pytest 4.6.0 | "Unroll calls to `all` to full for-loops with assertion rewriting for better failure messages, especially when using Generator Expressions" (#5062; PR #5103 merged 2019-05-27) | [A] PT3 changelog line 6421; PT4 |
| 2019-06-02 | pytest 4.6.1 | "Fix assertion rewriting of `all()` calls to deal with non-generators" (#5358) | [A] PT3 line 6397 |
| 2019-06-03 | pytest 4.6.2 | "Revert unrolling of `all()`" to fix a `NameError` on nested comprehensions (#5370), wrong handling of generators with `if` (#5371) and a wrong assertion when `all()` is part of an expression (#5372); PR #5373 | [A] PT3 lines 6379-6385; PT4 |
| 2019-06-04 | pytest | Issue #5062 stays open; a maintainer wrote that the fix should be "an api to have collections and collection tests be expressed in a general manner", "meant for users", and that "`assert x == sorted(x)` should work just as well as `assert all(x)`" | [A] PT4 |
| 2010 | CPython 2.7 / 3.2 | `assertItemsEqual` had "a fast-path using sorted()" that "is incorrect" for `[{2,4}, {1,2}]` against its reverse (bpo-10242); it was rewritten on `collections.Counter` with an equality fallback and named `assertCountEqual` in 3.2 | [A] UT4; UT3 test comment "see issue 10242"; UT2 3.2 what's new |
| 3.2 → 3.12 | CPython | `assertDictContainsSubset` was deprecated in 3.2 "because it was misimplemented with the arguments in the wrong order", and removed in 3.12 as "Undocumented and broken" | [A] UT2 what's new 3.2 lines 1827-1832, 3.12 lines 1635-1636 |
| 0.12.0, 2024-08 | googletest-rust | `Matcher` changed to `Matcher<ActualT: Debug + Copy>`, taking the value, not a reference; the line "does not support matching directly against an Iterator" left the docs | [A] GR3, GR4 |
| 5.1.0, 2024-02 | Chai | `members` accepts `Set`s (PR #1583, "Set support in same members"); `iterable` assertion added (PR #1592); 6.3.0 "require callable iterators in iterable assertions" (PR #1861) | [A] CH2 |
| 30.0.0 | Jest | `expect.arrayOf` added (PR #15567) | [A] JS1 |
| 2024-12 → open | testify | `iter.Seq` support in `Contains` and `ElementsMatch` proposed and held over laziness | [A] GO2 |
| deprecations | PyHamcrest, AssertJ | `contains` deprecated for `contains_exactly` (PyHamcrest #66); `hasOnlyOneElementSatisfying` deprecated for `singleElement()` | [A] PH1, AJ1 |

I found no tool that rewrites `any()`; pytest's PR #5103 was titled "Unroll calls to any #5062", but its changelog entry and its revert name `all()` only ([A] PT3, PT4).

## Claims with one source

These rest on one source, or on sources from one project only:

- every usage number in section 1 (ecosyste.ms; JetBrains 2025 for C++);
- AssertJ, Java Hamcrest, JUnit and GoogleTest failure text: each from that project's code and its own tests, not probed;
- a one-shot `Iterable` in AssertJ or Java Hamcrest would be iterated twice ([B], inference from source);
- Vitest's lazy behaviour (one probe, no source read);
- the testify laziness argument (one PR thread);
- GoogleTest `ContainerEq` printing nothing when only the order differs ([B]).

Claims with two sources include pytest's `all()` output (probe and #5062), the pytest revert (changelog and PR #5373), the Hamcrest greedy trap (Python probe, PyHamcrest #118, Java javadoc), Jest's generator TypeError (probe and #16541), Jest's `Received: {}` (probe and #8680), node's generator pass (probe and docs), and unittest's unhashable support (source, docs, CPython's test, probe).

## Appendix: the probes

All probes ran on 2026-11-16, in a scratch directory outside the repository.

| probe | tool versions | how |
|---|---|---|
| Python | pytest 9.0.3 and PyHamcrest 2.1.0 from nixpkgs, Python 3.13.17 | `nix-shell -p 'python3.withPackages (p: [p.pytest p.pyhamcrest])' --run 'python3 -I -m pytest -q --tb=short test_probe.py'`, and with `-vv`; 26 failing cases plus 5 for a generator that raises at item 3 |
| Rust | googletest 0.14.4, itertools 0.15.0, pretty_assertions 1.4.1; rustc 1.99.0 | `cargo test --no-fail-fast`, 19 failing tests, 2 compile-fail files, 1 `by_ref` test |
| JavaScript | expect 30.5.2, jest 30.5.2, chai 6.3.0, vitest 5.1.0; Node 24.22.1 | a script that catches each error and prints its message with colour codes stripped; one Jest test file run by `npx jest`; one Vitest file run by `npx vitest run` |
| Go | testify 1.12.1; Go 1.26.10 | `go test ./...` over 11 failing tests |

The pytest version probed (9.0.3) is older than the source read (9.2.0); the probed `-vv` header reads `Full diff:` where 9.2.0's source reads `Full diff: (-: missing in left side, +: extra in left side)`.
The rustc probed (1.99.0) is one release older than the source read (1.100.0); the panic text matches the 1.100.0 source.

The generator used throughout:

```python
def gen(*xs):
    yield from xs
```

```js
function* gen(...xs) { yield* xs; }
```

## Source register

Fetch method: `git clone --depth 1 --branch <tag>` and `grep`/`sed`; `curl -sSf` from `raw.githubusercontent.com/<repo>/<tag>/…` for CPython, Rust and Node; `gh issue view`, `gh pr view`, `gh api` for trackers; `curl` with a generic user agent for ecosyste.ms.
All rows are [A] unless marked.

| id | source | pinned at |
|---|---|---|
| PT1 | pytest `src/_pytest/assertion/` (`_compare_any.py`, `_compare_sequence.py`, `_compare_set.py`, `_guards.py`, `util.py`) | [9.2.0, `ae1d0ff`](https://github.com/pytest-dev/pytest/tree/ae1d0ff57b8bd4252da9ade7353523ac0ac73281/src/_pytest/assertion) |
| PT2 | pytest docs `how-to/assert.rst`, `how-to/subtests.rst`, `example/reportingdemo.rst` | [9.2.0, `ae1d0ff`](https://github.com/pytest-dev/pytest/tree/ae1d0ff57b8bd4252da9ade7353523ac0ac73281/doc/en) |
| PT3 | pytest `doc/en/changelog.rst`, lines 6153-6159, 6379-6397, 6421, 930 | [9.2.0, `ae1d0ff`](https://github.com/pytest-dev/pytest/blob/ae1d0ff57b8bd4252da9ade7353523ac0ac73281/doc/en/changelog.rst) |
| PT4 | pytest issue [#5062](https://github.com/pytest-dev/pytest/issues/5062) (open) and its comments; issue [#5370](https://github.com/pytest-dev/pytest/issues/5370); PRs [#5103](https://github.com/pytest-dev/pytest/pull/5103) (merge `2b9ca34`), [#5373](https://github.com/pytest-dev/pytest/pull/5373) (merge `63099bc`) | read 2026-11-16 |
| UT1 | CPython `Lib/unittest/case.py` (1040-1280), `Lib/unittest/util.py` (138-170) | [v3.15.2, `5e58eaa`](https://github.com/python/cpython/blob/5e58eaad7c58df58dfa8ea549c87996e01dd1077/Lib/unittest/case.py) |
| UT2 | CPython `Doc/library/unittest.rst` (636-690, 1325-1337); `Doc/whatsnew/2.7.rst`, `3.2.rst`, `3.12.rst` | [v3.15.2, `5e58eaa`](https://github.com/python/cpython/tree/5e58eaad7c58df58dfa8ea549c87996e01dd1077/Doc) |
| UT3 | CPython `Lib/test/test_unittest/test_case.py`, `testAssertCountEqual` (1092-1140) | [v3.15.2, `5e58eaa`](https://github.com/python/cpython/blob/5e58eaad7c58df58dfa8ea549c87996e01dd1077/Lib/test/test_unittest/test_case.py) |
| UT4 | CPython issue [#54451](https://github.com/python/cpython/issues/54451) (bpo-10242) and its comments | read 2026-11-16 |
| PH1 | PyHamcrest `src/hamcrest/library/collection/` | [V2.1.0, `dede7a3`](https://github.com/hamcrest/PyHamcrest/tree/dede7a338d9bb9a3fa21ddd5958b8c64a6df3282/src/hamcrest/library/collection) |
| PH2 | PyHamcrest `src/hamcrest/core/assert_that.py` (65-73) | [V2.1.0, `dede7a3`](https://github.com/hamcrest/PyHamcrest/blob/dede7a338d9bb9a3fa21ddd5958b8c64a6df3282/src/hamcrest/core/assert_that.py) |
| PH3 | PyHamcrest `tests/hamcrest_unit_test/collection/`, `matcher_test.py` | [V2.1.0, `dede7a3`](https://github.com/hamcrest/PyHamcrest/tree/dede7a338d9bb9a3fa21ddd5958b8c64a6df3282/tests/hamcrest_unit_test) |
| PH4 | PyHamcrest issues [#118](https://github.com/hamcrest/PyHamcrest/issues/118) (open), [#20](https://github.com/hamcrest/PyHamcrest/issues/20) | read 2026-11-16 |
| RS1 | Rust `library/core/src/panicking.rs` (434-458) | [1.100.0, `0ef1bc3`](https://github.com/rust-lang/rust/blob/0ef1bc386059596dada8fb65b50425aab2ea637a/library/core/src/panicking.rs) |
| RS2 | rust-lang/rust [#44838](https://github.com/rust-lang/rust/issues/44838), RFC 2011 tracking (open since 2017-09-25) | read 2026-11-16 |
| IT1 | itertools `src/lib.rs` (5224-5278) | [v0.15.0, `37bd72a`](https://github.com/rust-itertools/itertools/blob/37bd72aa6d58e594711d127b52418ca5e58b6091/src/lib.rs) |
| GR1 | googletest-rust `googletest/src/matcher.rs`, `src/matchers/*.rs`, `src/matcher_support/match_matrix.rs` | [v0.14.4, `d5a1f11`](https://github.com/google/googletest-rust/tree/d5a1f11e9ec6f2b9294ec6a971c92c444f633bad/googletest/src) |
| GR2 | googletest-rust `googletest/tests/*_matcher_test.rs` | [v0.14.4, `d5a1f11`](https://github.com/google/googletest-rust/tree/d5a1f11e9ec6f2b9294ec6a971c92c444f633bad/googletest/tests) |
| GR3 | googletest-rust `elements_are_matcher.rs` at tags v0.10.0, v0.11.0 (line 68-69) and v0.12.0 | `raw.githubusercontent.com/google/googletest-rust/<tag>/…` |
| GR4 | googletest-rust release [v0.12.0](https://github.com/google/googletest-rust/releases/tag/v0.12.0) notes (2024-08-02) | read 2026-11-16 |
| JU1 | JUnit `junit-jupiter-api/src/main/java/org/junit/jupiter/api/AssertIterableEquals.java`, `AssertionUtils.java` (85-90) | [r6.1.3, `f59f60d`](https://github.com/junit-team/junit-framework/tree/f59f60d2cebdf2224235d81f781b1f310cbc8138/junit-jupiter-api/src/main/java/org/junit/jupiter/api) |
| JU2 | JUnit `jupiter-tests/src/test/java/org/junit/jupiter/api/AssertIterableEqualsAssertionsTests.java` (259-361) | [r6.1.3, `f59f60d`](https://github.com/junit-team/junit-framework/blob/f59f60d2cebdf2224235d81f781b1f310cbc8138/jupiter-tests/src/test/java/org/junit/jupiter/api/AssertIterableEqualsAssertionsTests.java) |
| JU3 | JUnit `Assertions.java` javadoc (`assertIterableEquals`, `assertLinesMatch`, `assertAll`) | [r6.1.3, `f59f60d`](https://github.com/junit-team/junit-framework/blob/f59f60d2cebdf2224235d81f781b1f310cbc8138/junit-jupiter-api/src/main/java/org/junit/jupiter/api/Assertions.java) |
| JU4 | JUnit user guide `documentation/modules/ROOT/pages/writing-tests/assertions.adoc` (55-59) | [r6.1.3, `f59f60d`](https://github.com/junit-team/junit-framework/blob/f59f60d2cebdf2224235d81f781b1f310cbc8138/documentation/modules/ROOT/pages/writing-tests/assertions.adoc) |
| AJ1 | AssertJ `assertj-core/src/main/java/org/assertj/core/` (`api/AbstractIterableAssert.java`, `api/AbstractIteratorAssert.java`, `api/IterableAssert.java`, `api/Assertions.java`, `internal/Iterables.java`, `error/*.java`, `configuration/Configuration.java`) | [3.27.7, `e840716`](https://github.com/assertj/assertj/tree/e84071667f5f8f13084af9dfa54cee5fd9db18db/assertj-core/src/main/java/org/assertj/core) |
| AJ2 | AssertJ `error/ShouldContainExactly_create_Test.java`, `error/ElementsShouldSatisfy_create_Test.java` | [3.27.7, `e840716`](https://github.com/assertj/assertj/tree/e84071667f5f8f13084af9dfa54cee5fd9db18db/assertj-core/src/test/java/org/assertj/core/error) |
| JH1 | Java Hamcrest `hamcrest/src/main/java/org/hamcrest/` (`collection/IsIterableContainingInAnyOrder.java`, `collection/IsIterableContainingInOrder.java`, `core/Every.java`, `core/IsIterableContaining.java`, `MatcherAssert.java`, `TypeSafeDiagnosingMatcher.java`) | [v3.0, `68984b8`](https://github.com/hamcrest/JavaHamcrest/tree/68984b85e869df6a888fffcad87e4b676a8fc0ac/hamcrest/src/main/java/org/hamcrest) |
| JH2 | Java Hamcrest tests under `hamcrest/src/test/java/org/hamcrest/collection/` and `core/` | [v3.0, `68984b8`](https://github.com/hamcrest/JavaHamcrest/tree/68984b85e869df6a888fffcad87e4b676a8fc0ac/hamcrest/src/test/java/org/hamcrest) |
| JS1 | Jest `packages/expect/src/matchers.ts`, `asymmetricMatchers.ts`, `docs/ExpectAPI.md`, `CHANGELOG.md` | [v30.5.2, `202dd8a`](https://github.com/jestjs/jest/tree/202dd8a14777c270607c2416bd5034c3c0abc984/packages/expect/src) |
| JS2 | Jest issue [#8680](https://github.com/jestjs/jest/issues/8680) (closed 2019) | read 2026-11-16 |
| JS3 | Jest issues [#16541](https://github.com/jestjs/jest/issues/16541) and [#16508](https://github.com/jestjs/jest/issues/16508) (both open) | read 2026-11-16 |
| CH1 | Chai `lib/chai/core/assertions.js` (3176-3300) | [v6.3.0, `d86de2a`](https://github.com/chaijs/chai/blob/d86de2a9519bdd0d89aa64c9fbcc0e880513b153/lib/chai/core/assertions.js) |
| CH2 | Chai releases v5.1.0 and v6.3.0; PRs [#1583](https://github.com/chaijs/chai/pull/1583), [#1592](https://github.com/chaijs/chai/pull/1592), [#1861](https://github.com/chaijs/chai/pull/1861) | read 2026-11-16 |
| NO1 | Node `doc/api/assert.md` (883-930, 2688-2775) | [v24.22.1, `ce7aaf1`](https://github.com/nodejs/node/blob/ce7aaf1cee78b8cf0ce17364260535c418d3ed26/doc/api/assert.md) |
| GO1 | testify `assert/assertions.go` (791-1260), `assert/assertion_order.go` | [v1.12.1, `959dbda`](https://github.com/stretchr/testify/blob/959dbdacf1533e155162811ea90c90117a420463/assert/assertions.go) |
| GO2 | testify PR [#1685](https://github.com/stretchr/testify/pull/1685) (open) and its comments | read 2026-11-16 |
| GT1 | GoogleTest `docs/reference/matchers.md`, "Container Matchers" (144-195) | [v1.18.0, `063de7e`](https://github.com/google/googletest/blob/063de7e9578f82b369302001269680b4b1553359/docs/reference/matchers.md) |
| GT2 | GoogleTest `googlemock/test/gmock-matchers-containers_test.cc` (1206-1277, 2340-2465) | [v1.18.0, `063de7e`](https://github.com/google/googletest/blob/063de7e9578f82b369302001269680b4b1553359/googlemock/test/gmock-matchers-containers_test.cc) |
| GT3 | GoogleTest `googlemock/include/gmock/gmock-matchers.h` (2580-2625), `googlemock/src/gmock-matchers.cc` (62-124), `googlemock/include/gmock/internal/gmock-internal-utils.h` (`StlContainerView`) | [v1.18.0, `063de7e`](https://github.com/google/googletest/tree/063de7e9578f82b369302001269680b4b1553359/googlemock) |
| U1 | ecosyste.ms packages API, `registries/<registry>/packages/<name>` for each tool in section 1 [B] | read 2026-11-16 |
| U2 | JetBrains, [The State of C++ 2025](https://lp.jetbrains.com/the-state-of-cpp-2025/), unit-testing question (1,800 respondents) [B] | read 2026-11-16 |
