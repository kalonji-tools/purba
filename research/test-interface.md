# What a test is in other runners: what it takes, what it returns, and what may change around it

This file answers [#688](https://github.com/kalonji-tools/purba/issues/688).
It feeds [#689](https://github.com/kalonji-tools/purba/issues/689).
It is never merged.

## Answer

Every runner in this survey says what a test is through its shape.
Rust and Go refuse a wrong shape before any test runs.
Rust refuses it at compile time; Go refuses it while it generates the test binary's `main`, and then no test in the package runs.
JUnit checks the shape when it discovers tests, but by default it only logs a warning, skips the method, and exits 0.
The Python and JavaScript runners mostly find out at the call.
Three runners pass a test whose body never ran: `unittest` (an `async def` or a generator method), Ward (a generator) and Vitest (a generator).
Only JUnit gives a test its inputs by type, through `ParameterResolver`, and its guide says resolvers may also go by name or annotation.
Go hands every test one typed context object.
pytest and Vitest go by parameter name, Ward by default value, and Rust and `unittest` give a test no input at all.
Only Rust gives a return value a meaning of its own: an `Err` or a failing `ExitCode` fails the test.
Jest and Vitest wait on a returned promise.
Go refuses any return value, and JUnit skips the method.
pytest and `unittest` warn and pass, and Ward and Vitest ignore a return.
No runner requires a test to name everything that affects it.
Each one has setup that applies to a test the test does not name.
Rust comes closest: it has no hooks at all, but tests in one process share statics.
In the seven runners that collect by name or marker, a function that just misses the rule is silently not collected.
Jest and Vitest register a test by a call, and a call made after the file's tests ended is silently dropped.

| runner (version run) | unit run and reported | how a test gets its inputs | what a test may return | async / generator test | setup the test does not name | how a misused test is reported, and when |
|---|---|---|---|---|---|---|
| pytest 9.1.1 | a function or method; each parametrized case and each subtest gets its own line | by parameter name, from a fixture of that name; a parameter with a default is skipped | anything; a value other than `None` passes, with a warning | async: fails at the call, body not run; generator: collection error that stops the session | autouse fixtures, `conftest.py` hooks, xunit-style setup, `usefixtures` | unfilled parameter: error at setup; class with `__init__`: not collected, warning |
| `unittest` (CPython 3.15.0) | a method of a `TestCase`; only failing subtests get a line | none: the method takes `self` only | anything; a value other than `None` passes, with a `DeprecationWarning` | ⚠️ `async def` in a plain `TestCase`, and any generator: **`ok`, body not run** | `setUp`, `setUpClass`, `setUpModule`, cleanups | extra parameter: error at the call |
| Ward 0.68.0b0 (archived) | a `@test` function; each `each` case gets its own line | by default value: `x=fixture_fn` | anything, ignored, no warning | async: native; ⚠️ generator and async generator: **pass, body not run, no warning** | session hooks; a fixture runs only when a test injects it | unfilled parameter: failure at the call (`TypeError`) |
| `cargo test` (rustc 1.95.0) | a `#[test]` fn; no parametrization | none: a test with an argument does not compile | a `Termination` type: `()`, `Result<(), E: Debug>`, `ExitCode`, `!`; `Err` fails | async, `gen`, `unsafe`: compile error | no hooks; statics are shared inside one process | compile error in the test build; a fn missing `#[test]` is silently not run |
| cargo-nextest 0.9.136 | the same fn, one process per test | as `cargo test` | as `cargo test` | as `cargo test` | setup scripts (experimental) can set env vars for a test | as `cargo test` |
| `go test` (go1.27.2) | `TestXxx`; each `t.Run` subtest gets its own line | one context object, `t *testing.T` | nothing: any result is refused | none exist; an iterator shape is refused like any other wrong signature | `TestMain`, `init()`, package variables | wrong signature: whole package refused while building the test main (`[setup failed]`) |
| JUnit Jupiter 6.1.3 | a `@Test` method; each parametrized invocation and each dynamic test is its own node | by `ParameterResolver`: by type, by annotation, or by name | `void` only; `@TestFactory` must return dynamic nodes | `CompletableFuture`: skipped as non-void; Kotlin `suspend`: supported (docs) | `@BeforeEach` and the other lifecycle methods, extensions, auto-detected extensions (off by default) | non-void, static, private: ⚠️ **warning on stderr, not run, exit 0**; unresolved parameter: failure at execution |
| Jest 30.5.2 | a `test()` call; each `test.each` row gets its own line | a `done` callback when the function declares a parameter | a promise or `undefined`; anything else fails at the call | async: native; generator: run through `co`; async generator: fails, body not run | `beforeEach`/`beforeAll`, `setupFiles`, `setupFilesAfterEnv` | no function: the whole file fails at load |
| Vitest 5.0.5 | a `test()` call; each `test.each`/`test.for` case gets its own line | a context object; fixtures by destructured name, read from the function's source | anything; a thenable is awaited, the rest is ignored | async: native; ⚠️ generator and async generator: **pass, body not run** | hooks, `setupFiles`, `globalSetup`, `auto` fixtures | no function: `todo`, exit 0; undefined fixture name: `undefined`, passes |

Every row is [A]: each runner was run in this session (keys PY, RS, GO, JV, JS in Appendix A), and its source was read at a tag.
Ward's row ran on CPython 3.13.15, because Ward was not tried on 3.15.

### ⚠️ Where a test is not run, or passes without its body running, with at most a warning

| runner | case | what the report shows | body ran? | tier | evidence |
|---|---|---|---|---|---|
| `unittest` | `async def test_x` in a plain `TestCase` | `ok`, plus a `DeprecationWarning` and a `RuntimeWarning` | no | [A] | PY-M4, Appendix D.1 |
| `unittest` | generator method, `assert False` before or after `yield` | `ok`, plus a `DeprecationWarning` | no | [A] | PY-M4, D.1 |
| `unittest` | generator or async generator in `IsolatedAsyncioTestCase` | `ok`, plus a `DeprecationWarning` | no | [A] | PY-M4, D.1 |
| Ward | generator or async generator test | `PASS`, no warning | no | [A] | PY-M6, D.2 |
| Vitest | generator or async generator test | `✓`, exit 0 | no | [A] | JS-M30, D.3 |
| Vitest | `test('x')`, `test('x', 'a string')`, `test(namedFn)` | `todo`, exit 0 | no | [A] | JS-M32, M36, M40 |
| Vitest | `test.each([])`, `test.for([])` | no test, exit 0 | — | [A] | JS-M41 |
| Vitest | a fixture name that `extend` never defined | the value is `undefined`; the test passes | yes | [A] | JS-M23 |
| Vitest | a `(done)` test that never calls `done` | `✓` | yes | [A] | JS-M26 |
| Jest, Vitest | `test()` called from a timer after the file's tests ended | nothing, exit 0 | no | [A] | JS-M12, M35 |
| Vitest | `test()` called from a timer during `afterAll` | nothing, exit 0 | no | [A] | JS-M39 |
| JUnit | `@Test` returning a value, `static` or `private` | a `WARNING` on stderr; missing from the tree and the counts; exit 0 | no | [A] | JV-M3, M5, M19, D.4 |
| JUnit | non-static inner class without `@Nested` | a `WARNING` on stderr; exit 0 | no | [A] | JV-M6 |
| JUnit | abstract `@Test` in an interface, overridden with no annotation | nothing | no | [A] | JV-M13 |
| JUnit | a test class whose name misses the launcher's default pattern | nothing, `0 tests found`, exit 0 | no | [A] | JV-M17 |
| `go test` | a method `func (S) TestM(t *testing.T)` | `ok`, vet clean | no | [A] | GO-M06 |
| `go test` | `testFoo`; a test function in a file that is not `_test.go`; a file excluded by a build tag | `ok` or `[no test files]`, exit 0 | no | [A] | GO-M13, M14, M17 |
| `go test` | `Example` with no `// Output:` comment (documented) | compiled, not run | no | [A] | GO-M15 |
| `go test` | `Testfoo`, or an `Example` with parameters, under `-vet=off` | not collected | no | [A] | GO-M12c, M24b |
| `go test` | a goroutine calls `t.Error` after its test returned, and no later test runs | `PASS` | yes | [A], one run, timing-dependent | GO-M09d |
| `cargo test` | a `pub fn` that is missing `#[test]`; a test in a `#[cfg(not(test))]` module; a file no `mod` line declares; `tests/sub/x.rs` | nothing | no | [A] | RS-M2, M4, M7 |
| `cargo test` | a private fn missing `#[test]`; a nested `#[test]` fn; `#[test]` on a macro call | a compile warning only; the run exits 0 | no | [A] | RS-M2, ce-macro-call |
| `cargo test` | returns `ExitCode::FAILURE` | `FAILED`, and no reason is printed | yes | [A] | RS-M2 |
| pytest | `check_x`, `Test_upper`, a class not named `Test*` | nothing | no | [A] | PY-M1 |
| pytest | `Test` class with `__init__` or `__new__` | `PytestCollectionWarning`; exit 0 when the other tests pass | no | [A] | PY-M1 |
| pytest | a parameter with a default, when a fixture of the same name exists | passes with the default; the fixture is not injected | yes | [A] | PY-M3 |
| pytest, `unittest`, Ward | returns `False` | passes (pytest and `unittest` warn; Ward does not) | yes | [A] | PY-M1, M4, M6 |
| `unittest`, Ward | a function or method that misses the name rule or `@test` | nothing | no | [A] | PY-M5, M6 |

purba itself, at `d369fee`: a test that runs an assert and then returns `False` prints `passed` [B], as the issue reports; this was not re-run here.

## 1. The unit

| runner | the unit | parametrized case or subtest | tier | source |
|---|---|---|---|---|
| pytest | a test function, or a method of a `Test*` class | each parametrized case is an item with its own line: `test_param[1] PASSED`, `test_param[2] FAILED` | [A] | PY-M1 |
| pytest | subtests are built in since 9.0.0: "**Support for subtests** has been added." | each subtest gets its own line (`SUBPASSED[sub] (i=0)`, `SUBFAILED[sub] (i=1)`), and the parent test fails with "contains 1 failed subtest" | [A] | [changelog.rst](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/changelog.rst?plain=1#L496); PY-M1 |
| `unittest` | one `TestCase` instance per test method: `self.suiteClass(map(testCaseClass, testCaseNames))` | no parametrize; with `-v` a failing `subTest` gets its own line, and a passing subtest gets none; "Ran 2 tests" counts methods | [A] | [loader.py#L97](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/loader.py#L97); PY-M4 |
| Ward | a function decorated with `@test("description")` | each `each(...)` case gets its own line and outcome: `PASS test_unit:3[1/3] param 1` | [A] | PY-M6 |
| `cargo test` | one `#[test]` fn, held as `StaticTestFn(fn() -> Result<(), String>)` | none: libtest has no parametrization; a grep of all 30 libtest files for `parametri`, `fixture`, `setup`, `teardown` and `before_each` found nothing | [A] | [types.rs#L85](https://github.com/rust-lang/rust/blob/1.99.1/library/test/src/types.rs#L85); RS-S9 |
| cargo-nextest | the same fn; "cargo-nextest then executes each individual test in a separate process, in parallel." | as `cargo test` | [A] | [how-it-works.md#L73](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/site/src/docs/design/how-it-works.md?plain=1#L73) |
| `go test` | Test, subtest (`t.Run`), Example, Benchmark, Fuzz target | each subtest gets its own `=== RUN` and `--- PASS`/`FAIL` line, and its own `-json` events (`"Test":"TestTable/wrong"`); `-run TestTable/wrong` selects one row | [A] | GO-M01a, M01b, M01d |
| JUnit | a `@Test` method | `@ParameterizedTest`, `@RepeatedTest`, `@TestFactory` and `@TestTemplate` each make a container, and each invocation is its own node with its own outcome (`[2] 2 ✘`) | [A] | [definitions.adoc#L22-L26](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/writing-tests/definitions.adoc?plain=1#L22-L26); JV-M1 |
| JUnit | dynamic tests get no lifecycle of their own: "`@BeforeEach` and `@AfterEach` methods and their corresponding extension callbacks are executed for the `@TestFactory` method but not for each _dynamic test_." | — | [A] | [dynamic-tests.adoc#L40-L44](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/writing-tests/dynamic-tests.adoc?plain=1#L40-L44) |
| Jest | a `test()` call, inside `describe` or at the top of a file | each `test.each` row is a test: `✓ each case 1`, `✕ each case 3` | [A] | JS-M1 |
| Vitest | a `test()` call | each `test.each` and `test.for` case gets its own line and outcome | [A] | JS-M21 |

- Every runner that has parametrization gives each case its own line and outcome: pytest, Ward, JUnit, Jest, Vitest, and Go through subtests [A].
- `unittest` is the exception for subtests: a passing subtest has no line (PY-M4) [A]. Whether a newer CPython lists them: [C].

## 2. What a test takes

| runner | how inputs are given | refused signatures, and when | tier | source |
|---|---|---|---|---|
| pytest | by parameter name: `getfuncargnames` returns "the names of a function's mandatory arguments", which leaves out a parameter with a default | a parameter nothing fills: ERROR at **setup**, "fixture 'nothing_provides_this' not found"; a keyword-only parameter with no default is the same | [A] | [compat.py#L108-L170](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/compat.py#L108-L170); PY-M1 |
| pytest | — | a positional-only parameter is not filled and not reported at setup: it FAILS at the **call** with `TypeError: test_posonly() missing 1 required positional argument: 'x'` | [A] | PY-M3 |
| pytest | — | a parameter with a default, `*args` and `**kwargs` pass; a default wins over a fixture of the same name, silently | [A] | PY-M1, M3 |
| pytest | — | parametrizing a parameter that has a default: collection error, "function already takes an argument 'y' with a default value", session interrupted | [A] | PY-M3 |
| `unittest` | none: the method is called as `result = method()`; data reaches it through `self`, usually from `setUp` | an extra parameter: ERROR at the **call**, `TypeError: T.test_extra() missing 1 required positional argument: 'x'`; a parameter with a default passes | [A] | [case.py#L636-L637](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/case.py#L636-L637); PY-M4 |
| `unittest` | — | a `TestCase` whose `__init__` does not take `methodName`: discovery crashes, and no test in the file runs | [A] | PY-M5 |
| Ward | by default value: `x=fixture_fn`, `x=each(...)` or a plain value, read with `bind_partial()` and `apply_defaults()` | a parameter nothing fills: FAIL at the **call**, `TypeError`; a keyword-only parameter with a fixture default also fails, likely because assertion rewriting drops `__kwdefaults__` (inferred, not isolated) | [A] run; [C] cause | [testing.py#L572-L593](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/testing.py#L572-L593), [_rewrite.py#L129-L136](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/_rewrite.py#L129-L136); PY-M6 |
| `cargo test` | none; the harness calls `$test_fn()` with no arguments, and a test calls helpers itself | `#[test] fn t(x: i32)`: **compile** error "functions used as tests can not have any arguments"; `fn t<T>()`: "functions used as tests can not have any non-lifetime generic parameters"; `fn t<'a>()` is accepted | [A] | [test.rs#L566](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_builtin_macros/src/test.rs#L566), [#L576](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_builtin_macros/src/test.rs#L576); RS-M1 |
| `cargo test` | the Reference: "The `test` attribute may only be applied to [free functions] that are monomorphic, that take no arguments, and where the return type implements the [`Termination`] trait." | the error appears only in the test build: `cargo build` of the same crates exited 0 | [A] | [testing.md#L31](https://github.com/rust-lang/reference/blob/3989fe91f1a601b51684a582ad17b2503f221787/src/attributes/testing.md?plain=1#L31); RS-M1 |
| rstest (a crate, for contrast) | by argument name: "will resolve each argument name by call the fixture function." | — | [A] | [lib.rs#L106](https://github.com/la10736/rstest/blob/v0.28.0/rstest/src/lib.rs#L106) |
| `go test` | one context object, `t *testing.T`; cmd/go checks it **by syntax** in `isTestFunc`: one parameter, no result, and the parameter is `*T` or `*x.T` | `TestA(t, x int)`, `TestB()`, `TestC(*testing.B)`, `TestV(testing.T)`: "wrong signature for TestA, must be: func TestA(t *testing.T)" while **building the test main**; the whole package is `[setup failed]` and no test in it runs | [A] | [load/test.go#L585-L609](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/load/test.go#L585-L609), [#L805-L818](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/load/test.go#L805-L818); GO-M02–M04, M21 |
| `go test` | — | `TestD[T any](t)`: "test functions cannot have type parameters", same stage | [A] | GO-M05 |
| `go test` | — | the syntax check can be fooled: with a local `type T struct{}`, `TestFake(t *T)` passes it and fails at **compile** of `_testmain.go` | [A] | GO-M22 |
| JUnit | "If a _test class_ constructor, a _test method_, or a _lifecycle method_ […] accepts a parameter, the parameter must be resolved at runtime by a registered `ParameterResolver`." | an unresolved `String` parameter: failed test at **execution**, `ParameterResolutionException: No ParameterResolver registered for parameter [java.lang.String arg0] …`; the body did not run | [A] | [dependency-injection….adoc#L7-L10](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/writing-tests/dependency-injection-for-constructors-and-methods.adoc?plain=1#L7-L10); JV-M2 |
| JUnit | "Generally speaking, parameters may be resolved by _name_, _type_, _annotation_, or any combination thereof." Built-ins resolve `TestInfo` and `TestReporter` by type, and `@TempDir` by annotation | a bare `Path` without `@TempDir` fails the same way | [A] | [parameter-resolution.adoc#L10-L11](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/extensions/parameter-resolution.adoc?plain=1#L10-L11); JV-M2 |
| Jest | a `done` callback when `fn.length > 0` | `(a, b)`: `a` is `done`; if it is never called, the test times out; `(a = 1)` and `(...args)` have length 0 and pass | [A] | [utils.ts#L31-L33](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-circus/src/utils.ts#L31-L33); JS-M2, M3 |
| Vitest | "The first argument for each test callback is a test context." Fixtures from `test.extend` are found by parsing the function's source: `let fnString = filterOutComments(implementation.toString())` | with fixtures registered, a non-destructured `(ctx)` fails at the **call** ("must use object destructuring pattern"); a fixture name never defined is `undefined` and the test passes; `(a, b)`: `b` is `undefined` | [A] | [test-context.md#L12](https://github.com/vitest-dev/vitest/blob/v5.0.5/docs/guide/test-context.md?plain=1#L12), [fixture.ts#L660-L726](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/runtime/runner/fixture.ts#L660-L726); JS-M22–M27 |
| Vitest | `done` | the context is a function that throws "done() callback is deprecated, use promise instead"; calling it fails the test; never calling it passes | [A] | [context.ts#L168](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/runtime/runner/context.ts#L168); JS-M26 |

- By type: JUnit's resolvers, and Go's single `*testing.T` [A]. No other runner in the survey picks an input by its type.
- By name: pytest (parameter name), Vitest (destructured property name, read from source text), rstest (argument name) [A].
- The earliest refusal of a bad parameter list is at compile time (Rust) or while building the test binary (Go) [A].
- In pytest, an unfilled parameter is caught at setup, before the call; in `unittest`, Ward and JUnit it surfaces at the call [A].

## 3. What a test returns

| runner | what a return means | a return the runner does not expect | tier | source |
|---|---|---|---|---|
| pytest | nothing: "Since pytest ignores return values, it might be surprising that the test will never fail based on the returned value." | `False` and `1` PASS with `PytestReturnNotNoneWarning`: "Test functions should return None, but test_returns.py::test_false returned <class 'bool'>." | [A] | [how-to/assert.rst#L530](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/how-to/assert.rst?plain=1#L530), [python.py#L160-L178](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/python.py#L160-L178); PY-M1 |
| pytest | history | 7.2.0 added a deprecation; 8.0.0rc1: "this will stay a warning instead of becoming an error in the future"; 8.4.0: "Tests will now fail, instead of raising a warning, if they return any value other than None."; 8.4.1: "Reintroduced :class:`pytest.PytestReturnNotNoneWarning` which was removed by accident in pytest `8.4`." | [A] | [changelog.rst#L877](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/changelog.rst?plain=1#L877), [#L851](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/changelog.rst?plain=1#L851), [#L2270](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/changelog.rst?plain=1#L2270) |
| `unittest` | nothing | `False` and `1` are `ok`, with "It is deprecated to return a value that is not None from a test case"; deprecated since 3.11, still a warning on `main` | [A] | [case.py#L636-L648](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/case.py#L636-L648); PY-M4 |
| Ward | nothing: `self.fn(**resolved_args)` drops it | `False`, `1`, `None` all PASS, no warning | [A] | [testing.py#L196-L202](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/testing.py#L196-L202); PY-M6 |
| `cargo test` | the outcome: the return must implement `Termination`, and the harness wraps it in `assert_test_result` | `-> bool`, `-> i32`, `-> impl Future`: **compile** error "the trait bound `bool: Termination` is not satisfied" | [A] | [lib.rs#L288-L297](https://github.com/rust-lang/rust/blob/1.99.1/library/test/src/lib.rs#L288-L297); RS-M1 |
| `cargo test` | the Reference: "Tests that return `ExitCode::SUCCESS` pass, and tests that return `ExitCode::FAILURE` fail." and "Tests that do not terminate neither pass nor fail." | `Err("boom from Err")` FAILS and prints `Error: "boom from Err"`; `ExitCode::FAILURE` FAILS with no reason printed; `#[should_panic]` with a `Result` return does not compile | [A] | [testing.md#L55-L61](https://github.com/rust-lang/reference/blob/3989fe91f1a601b51684a582ad17b2503f221787/src/attributes/testing.md?plain=1#L55-L61); RS-M1, M2 |
| `go test` | none possible | `TestE(t) error`, `TestF(t) bool`: "wrong signature", whole package refused while building the test main | [A] | [load/test.go#L588](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/load/test.go#L588); GO-M07, M08 |
| JUnit | none for `@Test`; the guide: test methods "must not return a value (except `@TestFactory` methods which are required to return a value)" | `@Test int`, `boolean`, `CompletableFuture<Void>`: ⚠️ `[WARNING] @Test method 'int ReturnsTest.returnsInt()' must not return a value. It will not be executed.` on stderr; the run exits 0 | [A] | [test-classes-and-methods.adoc#L5-L7](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/writing-tests/test-classes-and-methods.adoc?plain=1#L5-L7), [IsTestableMethod.java#L66-L79](https://github.com/junit-team/junit-framework/blob/r6.1.3/junit-jupiter-engine/src/main/java/org/junit/jupiter/engine/discovery/predicates/IsTestableMethod.java#L66-L79); JV-M3, M19 |
| JUnit | `@TestFactory` returns the tests: "must return a single %1$s or a Stream, Collection, Iterable, Iterator, Iterator provider, or array of %1$s" | `String`, `List<String>`, `void`: warning, not run; declared `Object` returning a `String`: runs, then fails as a container | [A] | [IsTestFactoryMethod.java#L43-L44](https://github.com/junit-team/junit-framework/blob/r6.1.3/junit-jupiter-engine/src/main/java/org/junit/jupiter/engine/discovery/predicates/IsTestFactoryMethod.java#L43-L44); JV-M4 |
| JUnit | the severity is configurable: "If an engine reports an issue with a severity equal to or higher than a configurable _critical_ severity, its tests will not be executed." The default is `ERROR` | with `junit.platform.discovery.issue.severity.critical=WARNING`, the whole engine fails and no test runs, valid ones included; exit 1 | [A] | [discovery-issues.adoc#L20-L27](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/running-tests/discovery-issues.adoc?plain=1#L20-L27); JV-M14 |
| Jest | a promise is awaited; nothing else means anything | `false`, `null`, `{a:1}` FAIL at the call: "test functions can only return Promise or undefined."; a duck-typed thenable passes | [A] | [utils.ts#L290-L301](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-circus/src/utils.ts#L290-L301); JS-M4 |
| Jest | — | `done` plus a returned promise: "Test functions cannot both take a 'done' callback and return something. Either use a 'done' callback, or return a promise." | [A] | [utils.ts#L241](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-circus/src/utils.ts#L241); JS-M4 |
| Vitest | a thenable is awaited; any other value goes to `resolve(result)` | `false`, `{a:1}`, `null` all PASS, no warning | [A] | [context.ts#L100-L110](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/runtime/runner/context.ts#L100-L110); JS-M28 |

- A return value carries a pass-or-fail meaning in Rust only [A].
- A returned promise means "wait for me" in Jest and Vitest [A].
- Two runners refuse a return before running: Go at build, Rust at compile [A]. JUnit refuses it at discovery but only warns [A].
- pytest moved a non-`None` return from a warning to a failure in 8.4.0, and back to a warning in 8.4.1 [A, changelog only; 8.4.0 was not run].

## 4. Async and generator tests

| runner | async test | generator test | tier | source |
|---|---|---|---|---|
| pytest | without a plugin: FAILED at the call, "async def functions are not natively supported."; body not run; an async generator is the same | collection ERROR: "'yield' keyword is allowed in fixtures, but not in tests (test_g)"; the whole session is interrupted, so tests in other files do not run either | [A] | [python.py#L147-L169](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/python.py#L147-L169), [#L240-L248](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/python.py#L240-L248); PY-M1, M2 |
| pytest (history) | 8.4.0: "Async tests will now fail, instead of warning+skipping"; before 4.4.0: "Previously ``async`` functions would not execute at all but still be marked as "passed"." | 8.4.0: yield tests became an error; they "have not been run since pytest 4.0" | [A] | [changelog.rst#L874](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/changelog.rst?plain=1#L874), [#L6236](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/changelog.rst?plain=1#L6236), [#L883](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/changelog.rst?plain=1#L883) |
| `unittest` | ⚠️ in a plain `TestCase`: `ok`, body not run, with "…returned 'coroutine'). Maybe you forgot to use IsolatedAsyncioTestCase as the base class?"; in `IsolatedAsyncioTestCase`: runs, and an `assert False` fails | ⚠️ `ok`, body not run, in both classes | [A] | [case.py#L644-L647](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/case.py#L644-L647), [async_case.py#L125-L133](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/async_case.py#L125-L133); PY-M4 |
| Ward | native: "You can declare any test or fixture as ``async`` in order to test asynchronous code" | ⚠️ PASS, body not run, no warning; an async generator is the same | [A] | [writing_tests.rst#L259](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/docs/source/guide/writing_tests.rst?plain=1#L259); PY-M6 |
| `cargo test` | compile error "async functions cannot be used for tests"; `#[tokio::test]` strips `async` and emits a plain `#[test]` that blocks on a runtime | `gen fn`: compile error "gen functions cannot be used for tests" (and gen blocks are unstable) | [A] | [test.rs#L532-L554](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_builtin_macros/src/test.rs#L532-L554); tokio-macros 2.7.1 [entry.rs](https://github.com/tokio-rs/tokio/blob/tokio-macros-2.7.1/tokio-macros/src/entry.rs#L441); RS-M1 |
| `go test` | none in the language; a goroutine's `t.Error` after its test returned panics the binary, or is lost if the process exits first | an iterator-shaped function is a wrong signature | [A] | GO-M09, M10 |
| JUnit | `@Test CompletableFuture<Void>`: skipped as non-void, with a warning; Kotlin `suspend` test methods are supported since 6.0: "Support for using Kotlin `suspend` functions as test methods"; "When using `suspend` test or lifecycle methods, JUnit internally uses `runBlocking` to execute them." | — | [A] run for Java; [A] docs, not run, for Kotlin | [release-notes-6.0.0.adoc#L17](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/partials/release-notes/release-notes-6.0.0.adoc?plain=1#L17), [test-classes-and-methods.adoc#L94](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/writing-tests/test-classes-and-methods.adoc?plain=1#L94); JV-M3 |
| Jest | native | sync generator: run through `co.wrap(fn)`, the body runs, and an `expect` fails; async generator: FAILS with "test functions can only return Promise or undefined.", body not run | [A] | [utils.ts#L274-L276](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-circus/src/utils.ts#L274-L276); JS-M5, M6 |
| Vitest | native | ⚠️ sync and async generator: `✓`, body not run, exit 0 | [A] run; [B] the mechanism (the async wrapper in `fixture.ts#L342` resolves with the generator object), read from source | JS-M29, M30 |

- Async runs natively in Ward, Jest, Vitest, `IsolatedAsyncioTestCase` and Kotlin on JUnit; through a plugin in pytest and a macro in Rust [A].
- Only Jest runs a generator test as a test [A]. pytest and Rust refuse it [A]. `unittest`, Ward and Vitest pass it without running its body [A].
- pytest itself once passed an async test without running it (before 4.4.0) [A, changelog].

## 5. What changes around a test without the test naming it

| runner | what applies without being named | does anything require the test to name it? | tier | source |
|---|---|---|---|---|
| pytest | a `conftest.py` hook, `setup_module`, `setup_function`, an autouse fixture that set an env var the test then read, `usefixtures`; "\"Autouse\" fixtures are a convenient way to make all tests automatically **request** them." | no | [A] | [how-to/fixtures.rst#L335-L336](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/how-to/fixtures.rst?plain=1#L335-L336); PY-M1 |
| `unittest` | in order: `setUpModule`, `setUpClass`, `setUp`, body, `tearDown`, `addCleanup`, `addClassCleanup`, `tearDownModule`, `addModuleCleanup` | no | [A] | PY-M4 |
| Ward | fixtures run only when injected: "If a fixture is never injected into a test or another fixture, it will never be evaluated."; but a `before_session` hook set an env var that a test naming nothing read, and `preprocess_tests` may "transform or filter out tests" | no | [A] | [first_tests.rst#L116](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/docs/source/tutorials/first_tests.rst?plain=1#L116), [hooks.py#L79](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/hooks.py#L79); PY-M7 |
| `cargo test` | no hooks: libtest has no setup or teardown; the Book's advice is a `common::setup()` the test calls itself; but statics are shared: with one thread, `a_first saw COUNTER=0`, then `b_second saw COUNTER=1` | no hook to name; shared process state is not named | [A] | [ch11-03#L179-L180](https://github.com/rust-lang/book/blob/1500248d8f230566e4ec9f27fcbb8fe9e2898ab1/src/ch11-03-test-organization.md?plain=1#L179-L180); RS-M9 |
| cargo-nextest | one process per test, so statics are not shared (both tests saw `COUNTER=0`); setup scripts (experimental): "Setup scripts can define environment variables that will be exposed to tests that match the script." A test that names nothing read `MY_ENV_VAR=Ok("set-by-setup-script")` | no | [A] | [setup-scripts.md#L153](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/site/src/docs/configuration/setup-scripts.md?plain=1#L153); RS-M10 |
| `go test` | `TestMain` set env and a counter around `m.Run`; `init()` set package variables; a variable one test wrote leaked into the next. "then the generated test will call TestMain(m) instead of running the tests or benchmarks directly." | no; the scoped helpers `t.Cleanup`, `t.Setenv`, `t.Chdir` and `t.Context` are named through `t` | [A] | [testing.go#L376-L377](https://github.com/golang/go/blob/go1.27.3/src/testing/testing.go#L376-L377); GO-M11 |
| JUnit | lifecycle methods, inherited ones, `@ExtendWith`, `@RegisterExtension`, `junit-platform.properties`, `LauncherSessionListener`; a `ServiceLoader` extension ran `SNEAKY beforeEach for t()` once auto-detection was on: "Auto-detection is an advanced feature and is therefore not enabled by default." | no; a parameter is named by type, a lifecycle method is not | [A] | [registering-extensions.adoc#L444-L448](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/extensions/registering-extensions.adoc?plain=1#L444-L448); JV-M10, M11 |
| Jest | `setupFiles`, `setupFilesAfterEnv` (with a `beforeEach` registered in it), top-level and `describe`-level `beforeAll` and `beforeEach` | no | [A] | [Configuration.md#L1813](https://github.com/jestjs/jest/blob/v30.5.2/docs/Configuration.md?plain=1#L1813); JS-M7 |
| Vitest | `setupFiles`, `globalSetup` (through `process.env`), hooks, and `auto` fixtures ("Automatic fixture - runs for every test even if not used"); a non-auto fixture is set up only when the test destructures its name: `if (isAutoFixture(fixture, options) \|\| usedProps.has(fixture.name))` | for non-auto fixtures only | [A] | [test-context.md#L274](https://github.com/vitest-dev/vitest/blob/v5.0.5/docs/guide/test-context.md?plain=1#L274), [fixture.ts#L368](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/runtime/runner/fixture.ts#L368); JS-M23, M31 |

- No runner in the survey requires a test to name everything that affects it [A, across all nine].
- Two runners build an input only when the test names it: Ward and Vitest (non-auto fixtures) [A]. Both also have hooks that a test does not name [A].
- Rust has no hook, so the only unnamed influence measured was shared process state [A]. nextest removes that sharing and adds setup scripts [A].

## 6. How each runner reports a misused test

Stages, earliest first: compile, build of the test binary, discovery or collection, setup, call.

| stage | runner and case | outcome | tier |
|---|---|---|---|
| compile | Rust: an argument, a type generic, `async`, `gen`, `unsafe`, a non-`Termination` return, `#[should_panic]` with `Result` | compile error in the test build; nothing runs | [A] RS-M1 |
| compile | Rust: a private fn missing `#[test]`, a nested `#[test]` fn, `#[test]` on a macro call | ⚠️ warning only; the run exits 0 | [A] RS-M2 |
| build of the test binary | Go: any wrong parameter or any result, found while cmd/go generates the test `main`, before compiling | error; the whole package is `[setup failed]`; no sibling test runs | [A] GO-M02–M08 |
| build of the test binary | Go vet (in the default `go test` set): `Testfoo`, a malformed Example | error; the package is `[build failed]`: "If go vet finds any problems, go test reports those and does not run the test binary." | [A] GO-M12, M16, M23b |
| discovery | JUnit: non-void, `static`, `private` `@Test`; inner class without `@Nested` | ⚠️ warning on stderr, not run, exit 0 by default | [A] JV-M3, M5, M6 |
| discovery | JUnit: `@BeforeEach` that returns a value; non-static `@BeforeAll` | `ERROR`: the whole Jupiter engine fails, no test runs | [A] JV-M9 |
| collection | pytest: generator test; parametrizing a defaulted parameter | error; the session is interrupted | [A] PY-M1, M3 |
| collection | pytest: `Test` class with `__init__` | ⚠️ not collected, warning | [A] PY-M1 |
| collection | `unittest`: `__init__` without `methodName` | discovery crashes, no test runs | [A] PY-M5 |
| collection (file load) | Jest: `test('x')` with no function, a non-function, `test(namedFn)`; an async `describe` | the file fails, and its other tests do not run | [A] JS-M8, M13, M14, M17 |
| collection (file load) | Vitest: the same three `test()` calls | ⚠️ `todo`, exit 0 | [A] JS-M32, M36, M40 |
| setup | pytest: a parameter no fixture fills | error for that test | [A] PY-M1 |
| call | pytest: `async def`, positional-only parameter | failure for that test | [A] PY-M1, M3 |
| call | `unittest`, Ward: an unfilled parameter | error (`unittest`) or failure (Ward), `TypeError` | [A] PY-M4, M6 |
| call | JUnit: unresolved parameter | failure, `ParameterResolutionException` | [A] JV-M2 |
| call | Jest: a non-promise return; `done` plus a promise; an async generator | failure | [A] JS-M4, M6 |
| call | `unittest`, Ward, Vitest: async or generator shapes in the ⚠️ table above | pass, body not run | [A] |

- Error versus failure: pytest reports an unfilled parameter as an `ERROR` and an `async def` as `FAILED`; `unittest` reports a `TypeError` as an error [A]. Ward's outcomes are `PASS`, `FAIL`, `SKIP`, `XFAIL`, `XPASS` and `DRYRUN`, with no error [A] ([testing.py#L444-L449](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/testing.py#L444-L449)).
- Meszaros draws the line at whether an assertion failed (item 7).
- Three runners refuse a bad test by stopping more than that test: Go (the package), pytest's generator check (the session) and Jest's missing function (the file) [A]. JUnit does the same only when its critical severity is lowered [A].

## 7. Definitions

### xUnit Test Patterns

The book was not fetched. Its author's site, xunitpatterns.com, carries the draft text, and says: "Note that the material here is now somewhat out of step with the book content because it hasn't been updated based on the results of copy editing." [A] ([index](http://xunitpatterns.com/)). Every quote below is the site's wording [A], fetched with `curl` and converted with pandoc (XP-S1–S9).

| term | verbatim | page |
|---|---|---|
| test | "A procedure, whether manually executed or automated, that can be used to verify that the system under test (SUT) is behaving as expected. Often called a test case." | [test](http://xunitpatterns.com/test.html) |
| Test Method | "Encode each test as a single Test Method on some class." / "We define each test as a method, procedure or function that implements the four phases" | [Test Method](http://xunitpatterns.com/Test%20Method.html) |
| Testcase Object | "Most members of the xUnit family create a separate Testcase Object for each test to isolate the tests from each other as prescribed by Independent Test" | [Testcase Object](http://xunitpatterns.com/Testcase%20Object.html) |
| test fixture, ambiguous | "Unfortunately, several different communities have chosen the term test fixture to mean slightly different things." | [test fixture - ambiguous](http://xunitpatterns.com/test%20fixture%20-%20ambiguous.html) |
| test fixture, xUnit | "In generic xUnit a test fixture is all the things we need to have in place in order to run a test and expect a particular outcome. Think of it as the preconditions of the test." | same |
| test fixture, NUnit | "In NUnit and VbUnit, test fixture is what we call the Testcase Class." | same |
| SUT | "The "system under test". It is short for "whatever thing we are testing" and is always defined from the perspective of the test." | [SUT](http://xunitpatterns.com/SUT.html) |
| Four-Phase Test | "The four parts are fixture setup, exercise SUT, result verification and fixture teardown." | [Four Phase Test](http://xunitpatterns.com/Four%20Phase%20Test.html) |
| setup as input | "The fixture setup phase of the test establishes the prior state of the test which is an important input to the test." | same |
| Fresh Fixture | "Each test constructs its own brand-new test fixture for its own private use." | [Fresh Fixture](http://xunitpatterns.com/Fresh%20Fixture.html) |
| Shared Fixture | "We reuse the same instance of the test fixture across many tests." | [Shared Fixture](http://xunitpatterns.com/Shared%20Fixture.html) |
| Implicit Setup | "Build the test fixture common to several tests in the setUp method." Its cost: "It can however lead to Obscure Tests (page X) caused by a Mystery Guest by making the test fixture used by each test less obvious." | [Implicit Setup](http://xunitpatterns.com/Implicit%20Setup.html) |
| Inline Setup | "Each Test Method creates it's own Fresh Fixture by calling the appropriate constructor methods to build exactly the test fixture it requires." | [Inline Setup](http://xunitpatterns.com/Inline%20Setup.html) |
| Mystery Guest (a cause of Obscure Test) | "The test reader is not able to see the cause and effect between fixture and verification logic because part of it is done outside the Test Method." | [Obscure Test](http://xunitpatterns.com/Obscure%20Test.html) |
| Interacting Tests (a cause of Erratic Test) | "Tests depend on each other in some way." | [Erratic Test](http://xunitpatterns.com/Erratic%20Test.html) |
| Implicit Teardown | "The Test Automation Framework calls our clean up logic in the tearDown method after every Test Method." | [Implicit Teardown](http://xunitpatterns.com/Implicit%20Teardown.html) |
| failed | "A test is considered to have failed when an assertion fails." | [XUnitBasics](http://xunitpatterns.com/XUnitBasics.html) |
| error | "A test is considered to have an error when either the SUT or the test itself fails in an unexpected way." | same |
| Test Discovery | "The Test Automation Framework discovers all the tests that belong to the test suite automatically." By "a Test Method naming convention such as "starts with 'test'"" or by "a method attribute (e.g. "[Test]") or annotation (e.g. "@Test")" | [Test Discovery](http://xunitpatterns.com/Test%20Discovery.html) |
| Parameterized Test | "We pass the information needed to do fixture setup and result verification to a utility method that implements the entire test lifecycle." | [Parameterized Test](http://xunitpatterns.com/Parameterized%20Test.html) |
| what a Test Method takes | "Most members of the xUnit family that implement Test Discovery (page X) will only create Testcase Objects (page X) for "no arg" methods that start with "test"" | same |

- The site describes "no arg" as what frameworks do; it never states that a Test Method must take no argument [C].
- The site says nothing about what a Test Method returns [C].
- "Mystery Guest", "General Fixture", "Interacting Tests" and "Test Run War" have no page of their own (404); they are causes inside Obscure Test and Erratic Test [A].

### Each runner's own one-sentence definition

| runner | verbatim | tier | source |
|---|---|---|---|
| pytest | "In the simplest terms, a test is meant to look at the result of a particular behavior, and make sure that result aligns with what you would expect." | [A] | [anatomy.rst#L6-L7](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/explanation/anatomy.rst?plain=1#L6-L7) |
| pytest, fixture | "In testing, a `fixture <https://en.wikipedia.org/wiki/Test_fixture#Software>`_ provides a defined, reliable and consistent context for the tests." | [A] | [explanation/fixtures.rst#L14-L15](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/explanation/fixtures.rst?plain=1#L14-L15) |
| `unittest` | "A :dfn:`test case` is the individual unit of testing." | [A] | [unittest.rst#L30](https://github.com/python/cpython/blob/v3.15.2/Doc/library/unittest.rst?plain=1#L30) |
| `unittest`, fixture | "A :dfn:`test fixture` represents the preparation needed to perform one or more tests, and any associated cleanup actions." | [A] | [unittest.rst#L24-L25](https://github.com/python/cpython/blob/v3.15.2/Doc/library/unittest.rst?plain=1#L24-L25) |
| Ward | none for "test"; for a fixture: "A `fixture` is a function that provides tests with the data they need in order to run." | [A] for the fixture; no definition of "test" in the pages fetched | [fixtures.rst#L6](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/docs/source/guide/fixtures.rst?plain=1#L6) |
| Rust Book | "At its simplest, a test in Rust is a function that’s annotated with the `test` attribute." | [A] | [ch11-01#L21](https://github.com/rust-lang/book/blob/1500248d8f230566e4ec9f27fcbb8fe9e2898ab1/src/ch11-01-writing-tests.md?plain=1#L21) |
| Rust Reference | "The *`test` [attribute][attributes]* marks a function to be executed as a test." | [A] | [testing.md#L15](https://github.com/rust-lang/reference/blob/3989fe91f1a601b51684a582ad17b2503f221787/src/attributes/testing.md?plain=1#L15) |
| cargo-nextest | none found in the 9 pages fetched | [C] | RS-S5 |
| `go help testfunc` | "A test function is one named TestXxx (where Xxx does not start with a lower case letter) and should have the signature," | [A] | GO-M23a |
| JUnit | "Test:: a node in the test tree that verifies expected behavior when executed (e.g. a `@Test` method)." | [A] | [definitions.adoc#L8-L9](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/writing-tests/definitions.adoc?plain=1#L8-L9) |
| JUnit | "Test Method:: any instance method that is directly annotated or meta-annotated with `@Test`, `@RepeatedTest`, `@ParameterizedTest`, `@TestFactory`, or `@TestTemplate`." | [A] | [definitions.adoc#L22-L24](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/writing-tests/definitions.adoc?plain=1#L22-L24) |
| Jest | none that defines "test"; the closest: "All you need in a test file is the `test` method which runs a test." | [A] | [GlobalAPI.md#L475](https://github.com/jestjs/jest/blob/v30.5.2/docs/GlobalAPI.md?plain=1#L475) |
| Vitest | "`test` or `it` defines a set of related expectations. It receives the test name and a function that holds the expectations to test." | [A] | [api/test.md#L22](https://github.com/vitest-dev/vitest/blob/v5.0.5/docs/api/test.md?plain=1#L22) |

JUnit's guide is the only one with a definitions list; Go's definition is a name and a signature [A].

## Across the survey

These hold across all nine runners, not one row.

| observation | runners | tier |
|---|---|---|
| A misuse that the runner sees before any test runs is refused for more than one test, or for none | Go refuses the package, pytest's generator check stops the session, Jest stops the file; JUnit skips the one method and exits 0 | [A] |
| A shape check that only warns leaves the run green | JUnit (non-void, static, private), pytest (`__init__` class, non-`None` return), `unittest` (non-`None` return, async, generator) | [A] |
| A test whose body never ran is reported as passed | `unittest`, Ward, Vitest; and pytest before 4.4.0 | [A] |
| A near-miss name or marker is never collected, and nothing says so | pytest, `unittest`, Ward, `cargo test`, nextest, `go test`, JUnit (class name, abstract override); Jest and Vitest drop a late `test()` call | [A] |
| An input is chosen by type | JUnit (resolvers), Go (one fixed type) | [A] |
| An input is built only when the test names it | Ward, Vitest (non-auto fixtures), JUnit (parameters) | [A] |
| the same, for pytest's non-autouse fixtures | pytest | [B], not isolated in a run |
| Every input is named, and nothing applies unnamed | none | [A] |

## Appendix A. Commands

The scratch directory was `$P=/home/snregales/.claude/jobs/f70fe198/tmp/r688-scratch`.
Each runner had its own subdirectory, each case its own fixture directory, and every output file ends with the exit code.
Sources (S) were fetched on 2026-10-27 with `curl -sfL https://raw.githubusercontent.com/<owner>/<repo>/<tag>/<path>`.

| key | command |
|---|---|
| PY-S1 | pytest `9.1.1`: `src/_pytest/{python,fixtures,warning_types,subtests,compat,unittest,hookspec}.py`; `doc/en/{changelog,deprecations}.rst`, `doc/en/explanation/{goodpractices,fixtures,anatomy}.rst`, `doc/en/how-to/{fixtures,subtests,parametrize,xunit_setup,assert,skipping,unittest}.rst`, `doc/en/reference/{fixtures,reference}.rst` |
| PY-S2 | cpython `v3.15.2`: `Lib/unittest/{case,async_case,loader,suite,result}.py`, `Doc/library/unittest.rst`; `Lib/unittest/case.py` also at `v3.16.0a1` and `main` |
| PY-S3 | ward `release/0.68.0b0`: `ward/{_collect,_fixtures,_suite,_testing,testing,fixtures,hooks,models,_errors,_run,_rewrite}.py`, `docs/source/{index,guide/*,tutorials/first_tests,reference/hooks}.rst` |
| PY-S6 | pytest `8.4.0`: `src/_pytest/python.py` |
| PY-M1–M3 | `nix-shell -p python313`, a venv with `pip install pytest==9.1.1`; in each fixture dir `python -I -m pytest -v -rA -p no:cacheprovider` |
| PY-M4–M5 | `nix-shell -p python315 --run 'python3 -I -m unittest discover -v'` in each fixture dir |
| PY-M6–M7 | a venv with `pip install ward==0.68.0b0`; `python -I -m ward test --path . --test-output-style test-per-line --progress-style none --no-capture-output` (M7 without `-I`, with `PYTHONPATH` set, so that the hook module imports) |
| RS-S2–S8 | rust `1.99.1`: `compiler/rustc_builtin_macros/src/{test,test_harness,diagnostics}.rs`, `library/std/src/process.rs`, `library/test/src/*.rs`, `src/doc/rustc/src/tests/index.md`; reference `3989fe91`: `src/attributes/testing.md`; book `1500248d`: `src/ch11-0{0,1,2,3}-*.md`; nextest `cargo-nextest-0.9.150`: 9 pages under `site/src/docs/`; rstest `v0.28.0`; tokio `tokio-macros-2.7.1`: `tokio-macros/src/{entry,lib}.rs`; cargo `0.100.0`: `doc/book/src/reference/cargo-targets.md` |
| RS-S9 | `grep -rn -i -c -E 'setup\|teardown\|before_?each\|after_?each\|fixture\|parametri' library/test/src` over the 30 files: 0 |
| RS-M1 | `nix-shell -p cargo rustc cargo-nextest`; `cargo test --offline` in each compile-error crate `run/ce-*/`, and `cargo build --offline` in four of them |
| RS-M2–M5 | `cargo test --offline --no-fail-fast`; with `-- --include-ignored`; `-- --list`; `--examples` |
| RS-M6b–M8 | `cargo nextest list`; `cargo nextest run --no-fail-fast`; with `--run-ignored all` |
| RS-M9–M10 | shared state: `cargo test -- --test-threads=1 --nocapture`; `cargo nextest run --no-capture` with a setup script in `.config/nextest.toml` |
| GO-S1–S2 | go `go1.27.3`: `src/testing/{testing,example,fuzz,benchmark}.go`, `src/cmd/go/internal/load/test.go`, `src/cmd/go/internal/test/{test,testflag}.go`, `src/cmd/go/alldocs.go`, `src/cmd/vendor/golang.org/x/tools/go/analysis/passes/tests/{tests,doc}.go`, `src/go/doc/example.go` |
| GO-M01–M24 | `nix-shell -p go_1_27`, with `GOCACHE`, `GOPATH`, `GOMODCACHE` in scratch and `GOTOOLCHAIN=local GOPROXY=off GOENV=off`; one module per case; `go test -count=1 -v .` (suffix a), `go vet .` (suffix b), plus `-json`, `-run`, `-vet=off`, `-tags integration`; M23a `go help testfunc`, M23b `go help test` |
| JV-S1–S9 | junit-framework `r6.1.3`: guide pages under `documentation/modules/ROOT/pages/`, release notes 6.0.0 and 6.1.0, and `junit-jupiter-engine` predicates, selector resolvers, `LifecycleMethodUtils`, `ParameterResolutionUtils`, `MethodReflectionUtils` |
| JV-M1–M19 | `nix-shell -p jdk`; `javac -cp <jar> -d run/out/<case> run/fx/<case>/*.java`; `java -jar junit-platform-console-standalone-6.1.3.jar execute --disable-banner --disable-ansi-colors --details=tree --class-path run/out/<case> --scan-class-path [extra] 2>&1`; M14 adds `--config=junit.platform.discovery.issue.severity.critical=WARNING`; M18 uses `--select-class=Widget`; M19 splits stdout and stderr |
| JS-S1–S3 | jest `v30.5.2`: `packages/jest-circus/src/{utils,index,eventHandler,run}.ts`, `docs/{GlobalAPI,GettingStarted,Configuration}.md`; vitest `v5.0.5`: `packages/vitest/src/runtime/runner/{fixture,run,suite,context,collect,errors}.ts`, `docs/guide/test-context.md`, `docs/api/{test,hooks,describe}.md`, `docs/config/{setupfiles,globalsetup}.md` |
| JS-M1–M18 | `nix-shell -p nodejs`; `npm install --no-save jest@30.5.2 vitest@5.0.5`; `CI=true NO_COLOR=1 jest --ci --verbose --reporters=default --rootDir <case>` |
| JS-M21–M41 | `CI=true NO_COLOR=1 vitest run --reporter=verbose --root <case>` |
| XP-S1–S9 | `curl -sfL 'http://xunitpatterns.com/<Page%20Name>.html'` for the index, glossary and 63 pages (7 more returned 404); `pandoc -f html -t plain --wrap=none`; `curl -sI` for `Last-Modified` |
| L | `gh api repos/O/R/commits/<branch> --jq .commit.committer.date`; `gh api repos/O/R/releases/latest --jq '.tag_name+" "+.published_at'`; for repos with no releases, the tag's commit or tagger date |

Jest 30.5.2 picks an agent reporter when it detects an agent environment, so its first run printed no per-test line; every later Jest run passes `--reporters=default` [A] (JS-M0).

## Appendix B. Versions

Sources were read at the newest tag; runs used the version nixpkgs, pip, npm or Maven Central gave.

| tool | source read | version run |
|---|---|---|
| pytest | 9.1.1 (and 8.4.0 for history) | 9.1.1, CPython 3.13.15 |
| `unittest` | CPython v3.15.2 | CPython 3.15.0 |
| Ward | release/0.68.0b0 (commit `f41c6164f392`; links use the SHA, because the tag name holds a slash) | 0.68.0b0, CPython 3.13.15 |
| `cargo test` | rust 1.99.1, cargo 0.100.0 | rustc and cargo 1.95.0 |
| cargo-nextest | 0.9.150 | 0.9.136 |
| `go test` | go1.27.3 | go1.27.2 |
| JUnit | r6.1.3 | console launcher 6.1.3, OpenJDK 21.0.12.1 |
| Jest | v30.5.2 | 30.5.2, node 24.21.0 |
| Vitest | v5.0.5 | 5.0.5, node 24.21.0 |

Rust's compiler notes differ between 1.95 and 1.99.1, so compiler messages here are quoted from the 1.95 runs only [A].

## Appendix C. Liveness

| project | default branch, last commit | latest release or tag | date |
|---|---|---|---|
| pytest-dev/pytest | `main`, 2026-10-27 | release 9.1.1 | 2026-06-19 |
| python/cpython | `main`, 2026-10-27 | tag v3.15.2 (no GitHub releases) | 2026-10-25 (tagger) |
| darrenburns/ward | `master`, 2024-07-05; **archived** | release/0.68.0b0 | 2023-12-18 |
| rust-lang/rust | `main`, 2026-10-27 | release 1.99.1 | 2026-10-16 |
| rust-lang/reference | `master`, 2026-10-20 | no releases or tags | — |
| rust-lang/book | `main`, 2026-09-02 | release nostarch-third-printing | 2025-12-19 |
| rust-lang/cargo | `master`, 2026-10-27 | tag 0.100.0 (no GitHub releases) | 2026-08-28 (tag commit) |
| nextest-rs/nextest | `main`, 2026-10-26 | release cargo-nextest-0.9.150 | 2026-10-26 |
| la10736/rstest | `master`, 2026-10-18 | release v0.28.0 | 2026-10-18 |
| tokio-rs/tokio | `master`, 2026-10-27 | release tokio-1.54.1; tag tokio-macros-2.7.1 | 2026-10-21 |
| golang/go | `master`, 2026-10-27 | tag go1.27.3 (no GitHub releases) | 2026-10-20 (tag commit) |
| junit-team/junit-framework (was junit5) | `main`, 2026-10-27 | release r6.1.3 | 2026-08-07 |
| jestjs/jest | `main`, 2026-10-02 | release v30.5.2 | 2026-09-18 |
| vitest-dev/vitest | `main`, 2026-10-26 | release v5.0.5 | 2026-10-15 |
| xunitpatterns.com | no repo; static site | the book, 2007 | `Last-Modified` 2015-06-26 on every page sampled |

## Appendix D. Raw output

Excerpts, copied from the run files; each case wrote a marker file only if its body ran.

### D.1 `unittest`, PY-M4 (CPython 3.15.0)

Only `async_iso_body_ran` exists in the fixture directory afterwards.

```
test_async_iso (test_async.Iso.test_async_iso) ... FAIL
test_asyncgen_in_iso (test_async.Iso.test_asyncgen_in_iso) ... /nix/store/…-python3-3.15.0/lib/python3.15/unittest/case.py:747: DeprecationWarning: ("It is deprecated to return a value that is not None from a test case (<bound method Iso.test_asyncgen_in_iso of <test_async.Iso testMethod=test_asyncgen_in_iso>> returned 'async_generator')",)
ok
test_gen_in_iso (test_async.Iso.test_gen_in_iso) ... …DeprecationWarning: (… returned 'generator')",)
ok
test_async_in_plain (test_async.Plain.test_async_in_plain) ... …DeprecationWarning: It is deprecated to return a value that is not None from a test case (… returned 'coroutine'). Maybe you forgot to use IsolatedAsyncioTestCase as the base class?
…/unittest/case.py:691: RuntimeWarning: coroutine 'Plain.test_async_in_plain' was never awaited
ok
test_gen_after (test_async.Plain.test_gen_after) ... …DeprecationWarning: … returned 'generator')
ok
test_gen_before (test_async.Plain.test_gen_before) ... …DeprecationWarning: … returned 'generator')
ok
Ran 6 tests in 0.010s

FAILED (failures=1)
EXIT=1
```

### D.2 Ward, PY-M6

Only `async_body_ran` exists afterwards.

```
Found 4 tests and 0 fixtures in 0.00 seconds.

 FAIL  test_async:5 async assert False
 PASS  test_async:10 generator: yield then assert False
 PASS  test_async:16 generator: assert False then yield
 PASS  test_async:22 async generator
```

### D.3 Vitest and Jest, JS-M30 and JS-M6

The same fixture shape in both: two failing generator bodies that push to `RAN`, then a test that prints `RAN`.

```
$ vitest run --reporter=verbose
stdout | gen.test.js > report which bodies ran
RAN=[]

 ✓ gen.test.js > generator failing body 2ms
 ✓ gen.test.js > async generator failing body 0ms
 ✓ gen.test.js > report which bodies ran 1ms

      Tests  3 passed (3)
EXIT=0
```

```
$ jest --ci --verbose --reporters=default
    RAN=["g1","g2:7"]
FAIL ./gen.test.js
  ✕ generator failing body (5 ms)
  ✓ generator yields promise then passes
  ✕ async generator failing body (2 ms)
  ✓ report which bodies ran (13 ms)
  ● async generator failing body
    test functions can only return Promise or undefined.
    Returned value: Object {}
```

### D.4 JUnit, JV-M3

```
WARNING: TestEngine with ID 'junit-jupiter' encountered 3 non-critical issues during test discovery:

(1) [WARNING] @Test method 'java.util.concurrent.CompletableFuture<java.lang.Void> ReturnsTest.returnsFuture()' must not return a value. It will not be executed.
(2) [WARNING] @Test method 'int ReturnsTest.returnsInt()' must not return a value. It will not be executed.
(3) [WARNING] @Test method 'boolean ReturnsTest.returnsFalse()' must not return a value. It will not be executed.
╷
├─ JUnit Jupiter ✔
│  └─ ReturnsTest ✔
│     └─ voidControl() ✔
[         1 tests found           ]
[         1 tests successful      ]
EXIT CODE: 0
```

### D.5 pytest, PY-M1 and PY-M2

```
test_async.py::test_x FAILED                                             [100%]
async def functions are not natively supported.
You need to install a suitable plugin for your async framework, for example:
  - anyio
```

```
collecting ... collected 1 item / 1 error
________________________ ERROR collecting test_b_gen.py ________________________
'yield' keyword is allowed in fixtures, but not in tests (test_g)
!!!!!!!!!!!!!!!!!!!! Interrupted: 1 error during collection !!!!!!!!!!!!!!!!!!!!
=============================== 1 error in 0.09s ===============================
EXIT=2
```

### D.6 Go, GO-M02a and GO-M06a

```
# example.com/m02_extra_param
…/p_test.go:14:1: wrong signature for TestA, must be: func TestA(t *testing.T)
FAIL	example.com/m02_extra_param [setup failed]
FAIL
# ---- exit: 1 ----
```

```
=== RUN   TestOK
    p_test.go:12: TestOK body ran
--- PASS: TestOK (0.00s)
PASS
ok  	example.com/m06_method	0.004s
# ---- exit: 0 ----
```

The method `func (S) TestM(t *testing.T)` in the same file has no line.

### D.7 Rust, RS-M1 and RS-M2

```
error: async functions cannot be used for tests
 --> src/lib.rs:1:9
  |
1 | #[test] async fn t() {}
  |         -----^^^^^^^^^^
  |         |
  |         `async` because of this
[exit code: 101]
```

```
---- tests::result_err stdout ----
Error: "boom from Err"

failures:
    tests::exitcode_failure
    tests::exitcode_from_3
    …
    tests::result_exitcode_ok_failure
test result: FAILED. 5 passed; 7 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`tests::exitcode_failure`, `tests::exitcode_from_3` and `tests::result_exitcode_ok_failure` have no stdout block above the list.
