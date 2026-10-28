# How Python test and dependency-injection tools let a test name its inputs, their roles and their lifetimes

This file answers [#699](https://github.com/kalonji-tools/purba/issues/699).
It feeds [#700](https://github.com/kalonji-tools/purba/issues/700).
It is never merged.

Tiers: **[A]** run in this session, or source read at a pinned tag, or both; **[B]** reported by a primary source, not run; **[C]** inferred.
Run keys (PT-M1, FA-M2, …) point to Appendix A; raw output is in Appendix D.

## Answer

The survey found three ways for a test to name an input: by parameter name, by the provider itself, and by type.
pytest and Litestar go by parameter name, and so does rodi for a parameter with no annotation.
Ward, FastAPI and dependency-injector put the provider itself on the parameter, as a default value or as a marker.
injector, dishka, lagom, svcs, wireup, rodi and punq go by type.
For two values of one type, each by-type library asks the tester for a second key.
That key can be a `NewType`, a subclass, an `Annotated` marker string, a component string or a qualifier string.
No tool shows a value's lifetime on the test's signature.
Each writes the lifetime at the provider or in the container's config.
FastAPI and dependency-injector are the exceptions: a consumer can write part of the lifetime on its own line. In FastAPI an alias then hides it.
No tool refuses a test that names two values of one type without a role.
Each gives one object or two, decided by the provider's lifetime. Only FastAPI lets the test choose, with `use_cache=False`.
A second provider for one key is refused by lagom, wireup and rodi, and by dishka in strict mode.
injector, svcs, punq and dishka's default let the last provider win, silently.
pytest and Litestar let the closest one win, silently.
One generic transaction provider that serves every role, the shape of `Tx[SourceDb]` in #690, worked only in dishka.
It failed there with a bounded `TypeVar` and `NewType` roles, which is #690's own shape.
Only three ways put the provider on the test's own line: Ward's default value, FastAPI's inline `Depends`, and dependency-injector's `Provide[Container.x]`. pytest-lazy-fixtures puts it on the decorator line.
A by-type signature shows the role as a type name only, and not where the value comes from.

| tool (version run) | 1. how a test names an input | 2. how it names two values of one type | 3. lifetime kinds, and where they are written | 4. the signature shows: role / lifetime / provider | no role given: `a: Database, b: Database` | a second provider for one key | tier |
|---|---|---|---|---|---|---|---|
| pytest 9.2.0 | by parameter name; the annotation is never read | two fixture names | `function` … `session`, at the provider: `@pytest.fixture(scope=…)` | role by name / no / no | one fixture reached by two names: **same object**; one name twice: `SyntaxError` | the closest definition wins, silently, and can change the scope | [A] |
| pytest-cases 3.10.1 | as pytest | role by tuple position: `src, dst = unpack_fixture("src,dst", pair)` | as pytest | as pytest | as pytest | as pytest | [A] |
| pytest-lazy-fixtures 1.4.1 | as pytest | a parameter name, mapped to a fixture in the decorator: `lf("source_tx")` | as pytest | role and provider, on the decorator line / no / yes | as pytest | as pytest | [A] |
| Ward 0.68.0b0 (archived) | by default value: the fixture function; or `@using(src=…)` | two fixture functions | `Scope.Test`, `Module`, `Global`, at the provider | role / no / **yes** | **same object** | n/a: a fixture is a function, not a key | [A] |
| FastAPI 0.162.0 | by marker: `Annotated[T, Depends(fn)]` or `= Depends(fn)`; `T` is never read | two provider callables, usually behind two `Annotated` aliases | per request; the cache (`use_cache=`) and a yield exit (`scope=`) are written **at the consumer**; no app lifetime in `Depends` | inline: role / no / **yes**; alias: role / no / no | **same object** (per-request cache); two with `use_cache=False` | n/a: the callable is the key | [A] |
| Litestar 2.25.0 | by parameter name, marked `NamedDependency[T]`; `T` is validated at request time | two keys in `dependencies={…}` | per request; `use_cache=True` outlives the app; at the provider registration | role / no / no | one provider under two keys: **refused** at app build | the closest layer wins, silently | [A] |
| injector 0.24.0 | by type, with `@inject` | role by `NewType`, or `Annotated[Tx, "source"]` | none (new per injection), `singleton`, thread-local; at the provider or class; no request scope, **no teardown** | role / no / no | **two objects** (no scope); one with `singleton` | the last binding wins, silently | [A] |
| dishka 1.10.1 | by type, behind `FromDishka[T]` | role by `NewType`, or by component: `Annotated[Tx, FromComponent("source")]` | `Scope.APP`, `REQUEST` and four more, at the provider: `@provide(scope=…)` | role / no / partly ("from the container") | **same object** (cached in scope); two with `cache=False` | the last wins, silently; refused under `STRICT_VALIDATION` | [A] |
| lagom 2.7.7 | by type, with `= injectable` or `magic_bind_to_container` | role by `NewType`; ⚠️ an `Annotated` marker is dropped | per resolve, `Singleton`, per call (`ContextContainer`); **in the container config** | role / no / partly | **two objects**; one with `Singleton` | **refused** at definition | [A] |
| svcs 26.2.0 | service locator: `container.get(T)` in the body; since 26.1.0, `svcs.autowire(fn)` fills parameters by annotation | role by `NewType`, subclass or `Annotated` key | per registry (`register_value`) or per container (a factory); **in the container config**, and by how long the caller keeps the container | locator: nothing; autowire: role / no / no | **same object** | the last wins, silently | [A] |
| wireup 2.13.0 | by type, behind `Injected[T]` | role by qualifier: `Annotated[Tx, Inject(qualifier="source")]`; or `NewType` | `singleton`, `scoped`, `transient`, at the provider: `@injectable(lifetime=…)` | role / no / partly | same object (singleton, scoped); two (transient) | **refused** at container creation | [A] |
| dependency-injector 4.49.1 | by provider reference: `= Provide[Container.source_tx]`, or the same in `Annotated`; the annotation is never read | two container attributes | the provider class **in the container config** (`Singleton`, `Factory`, `Resource`), plus `Closing[…]` **at the consumer** for a per-call teardown | role / partly (`Closing`) / **yes** | same object (`Singleton`); two (`Factory`) | n/a: the attribute is the key | [A] |
| rodi 2.1.0 | by type; a parameter with no annotation, by its name | role by `NewType` or subclass; or by an unannotated name | singleton, scoped (one `exec` call), transient, at registration; **no teardown** | role / no / no | same object (singleton, scoped); two (transient) | **refused** at registration | [A] |
| punq 0.9.0 | by the type of a class constructor's parameter | role by `NewType` | transient, singleton, at `register`; no request scope, **no teardown** | role / no / no | ⚠️ **same object**, even for a transient, within one resolve | the last wins, silently | [A] |

Every row ran on CPython 3.13.13.
pytest-cases ran on pytest 9.0.2, because 3.10.1 does not load on pytest 9.2.0 (PC-M0).
that-depends 4.2.1 was read, not run: it has the shape of dependency-injector, and an opt-in by-type form [B] (section 12).

### The copy test, side by side

The scenario is the same in every tool (Appendix A.0).
A source and a target `Database` are each built once per session.
Each test gets a `Tx` on each, and each `Tx` rolls back after the test.
Two tests ran in one session, and every tool printed `db_built 2, tx_opened 4, tx_rolled_back 4`.
In injector, rodi and punq the probe itself did the rollback, because the tool has no teardown.
These are the lines a reader sees at the test:

```python
# pytest: role by parameter name
def test_copy(src: Tx, dst: Tx) -> None: ...

# pytest-lazy-fixtures: role by parameter name, mapped to a fixture in the decorator
@pytest.mark.parametrize(("src", "dst"), [(lf("source_tx"), lf("target_tx"))])
def test_copy(src: Tx, dst: Tx) -> None: ...

# pytest-cases: role by tuple position, unpacked into names
src, dst = unpack_fixture("src,dst", pair)
def test_copy(src: Tx, dst: Tx) -> None: ...

# Ward: role by provider, as the default value
@test("copy")
def _(src: Tx = source_tx, dst: Tx = target_tx) -> None: ...

# Ward: role by provider, in a decorator
@test("copy")
@using(src=source_tx, dst=target_tx)
def _(src: Tx, dst: Tx) -> None: ...

# FastAPI: role by provider, in an Annotated marker
def copy_route(src: Annotated[Tx, Depends(source_tx)], dst: Annotated[Tx, Depends(target_tx)]) -> dict: ...

# FastAPI: the same behind aliases, as its docs advise
SourceTx = Annotated[Tx, Depends(source_tx)]
def copy_route(src: SourceTx, dst: TargetTx) -> dict: ...

# Litestar: role by parameter name, marked
def copy_route(src: NamedDependency[Tx], dst: NamedDependency[Tx]) -> dict: ...

# dependency-injector: role by container attribute
@inject
def test_copy(src: Tx = Closing[Provide[Container.source_tx]], dst: Tx = Closing[Provide[Container.target_tx]]) -> None: ...

# injector, lagom, svcs (autowire), rodi: role by NewType (or subclass) of the adapter
def test_copy(src: SourceTx, dst: TargetTx) -> None: ...                      # + @inject (injector, rodi)
def test_copy(src: SourceTx = injectable, dst: TargetTx = injectable) -> None: ...   # lagom

# injector: role by Annotated marker string
def test_copy(src: Annotated[Tx, "source"], dst: Annotated[Tx, "target"]) -> None: ...

# dishka: role by NewType, behind the injection marker
def test_copy(src: FromDishka[SourceTx], dst: FromDishka[TargetTx]) -> None: ...

# dishka: role by component string
def test_copy(src: Annotated[Tx, FromComponent("source")], dst: Annotated[Tx, FromComponent("target")]) -> None: ...

# dishka: role on the stock, one generic adapter (the shape of #690; subclass roles or an unbound TypeVar only)
def test_copy(src: FromDishka[GTx[SourceDb]], dst: FromDishka[GTx[TargetDb]]) -> None: ...

# wireup: role by qualifier string
def test_copy(src: Annotated[Tx, Inject(qualifier="source")], dst: Annotated[Tx, Inject(qualifier="target")]) -> None: ...

# rodi: role by parameter name, with no annotation
def test_copy(source_tx, target_tx) -> None: ...

# punq: role by NewType, on a class constructor (punq fills no function)
class TestCopy:
    def __init__(self, src: SourceTx, dst: TargetTx) -> None: ...

# svcs: service locator; the signature names only the container
def test_copy(container: svcs.Container) -> None:
    src, dst = container.get(SourceTx, TargetTx)
```

What a reader who did not write the test can learn from that line alone:

| way | the role | the lifetime | where the value comes from | the type is the one received |
|---|---|---|---|---|
| role by parameter name (pytest, Litestar) | yes, if the name says it | no | no: the closest fixture or layer of that name | pytest: not checked (PT-M2); Litestar: checked at request time (LS-M4) |
| role by provider, as a default or in `@using` (Ward) | yes, twice: the parameter and the fixture's name | no | **yes**: an ordinary Python name an editor can follow | not checked (WD-M2) |
| role by provider in an inline marker (FastAPI) | yes | no | **yes** | not checked (FA-M3) |
| role by provider, behind an alias (FastAPI, wireup) | yes, by the alias name | no | no: the alias hides the provider and its options | FastAPI: not checked |
| role by container attribute (dependency-injector) | yes | only `Closing[…]`, if written | **yes**: the container and the attribute | not checked (DI-M2) |
| role by `NewType`, subclass or marker string (injector, dishka, lagom, svcs, wireup, rodi, punq) | yes, by the type or the string | no | no: any provider of that key in the container | yes: the type is the key |
| role by an unannotated name (rodi) | yes | no | no | no type on the line |
| role by tuple position (pytest-cases) | only after `unpack_fixture`, which sits elsewhere | no | no | not checked |

What type checkers do with `NewType`, `type` aliases and `Annotated` is in [the #690 prototype](https://github.com/kalonji-tools/purba/blob/010dda4/proto-690/RESULTS.md), and is not repeated here.

### ⚠️ Where a tool fills an input with a value the reader did not ask for, with no error

| tool | case | what the test got | tier | evidence |
|---|---|---|---|---|
| pytest | `source_db: Tx`, and a fixture `source_db` that returns a `Database` | a `Database`; passed | [A] | PT-M2 |
| pytest | a module redefines a conftest fixture's name | the module's fixture, and its scope changed from `session` to `function`; no warning under `-W error::pytest.PytestWarning` | [A] | PT-M5 |
| Ward | `a: Database = tx` | a `Tx`; passed | [A] | WD-M2 |
| FastAPI | `x: Annotated[int, Depends(get_db)]` | a `Database` | [A] | FA-M3 |
| FastAPI | `Annotated[Database, Depends()]` (no callable) | a `Database` built from the class, whose `__init__` parameters became query parameters: `?name=chosen-by-client` set its name | [A] | FA-M5, FA-M4b |
| Litestar | `Provide(make_db, use_cache=True)` | one value shared across two client sessions and two apps | [A] | LS-M4 |
| injector | a `NewType` or `Annotated` role that nothing binds, while `Database` is bound as a singleton | a fresh `Database('db')`, built from the base class, not the bound one | [A] | IJ-M5 |
| lagom | `Annotated[Database, "source"]`, with that key defined | a fresh `Database('db')`, neither defined value: injection drops the marker | [A] | LG-M2 (LG-M3b), LG-M5 |
| svcs | autowired `db: Database = None`, with `Database` not registered | `None` | [A] | SV-M6 |
| dependency-injector | `= Provide[Database]` (a type, not a provider) | the marker object itself; a warning only with `wire(warn_unresolved=True)` | [A] | DI-M2, DI-M3 |
| dependency-injector | `Provide[Container.source_tx]` on a `Resource`, without `Closing[…]` | the first test's transaction, in the second test: `target not clean at start: ['a', 'b']` | [A] | DI-M4 |
| rodi | `source_db: Database, target_db: Database`, with string keys `source_db` and `target_db` registered | both got one `Database`: an annotation switches the parameter from by-name to by-type | [A] | RD-M2 |
| rodi | classes `UserDB` and `UserDb`, both aliased `user_db` | a function got `UserDb`; a constructor refused through an `assert`, which `python -O` removes | [A] | RD-M5, RD-M6 |
| punq | two parameters of one transient type in one constructor | one object | [A] | PQ-M2 |
| injector, svcs, punq, dishka (default) | a second provider for one key | the last one | [A] | IJ-M2, SV-M3, PQ-M2, DK-M2 |

## 1. pytest 9.2.0

**Scenario** (PT-M1):

```python
@pytest.fixture(scope="session")
def source_db() -> Database: ...

@pytest.fixture
def src(source_db: Database):
    tx = Tx(source_db); yield tx; tx.rollback()
# target_db and dst: the same, for the target

def test_copy(src: Tx, dst: Tx) -> None: ...
```

A generic adapter can be written in three ways, and each moves the role out of the signature (PT-M4):

```python
def test_factory(tx_for, source_db: Database, target_db: Database) -> None: ...   # tx_for(source_db) in the body
src = tx_of("source_db")                                                            # a fixture stamped at module level
@pytest.mark.parametrize("tx", ["source_db", "target_db"], indirect=True)
def test_indirect(tx: Tx) -> None: ...                                              # one role per run, never both
```

| claim | source's own words | tier | source; run |
|---|---|---|---|
| a test names an input by parameter name | "it looks at the parameters in that test function's signature, and then searches for fixtures that have the same names as those parameters." | [A] | [how-to/fixtures.rst#L16-L20](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/how-to/fixtures.rst?plain=1#L16-L20); PT-M1 |
| only names count; a parameter with a default is not an input | `getfuncargnames`: "Return the names of a function's mandatory arguments." | [A] | [compat.py#L108-L154](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/compat.py#L108-L154) |
| nothing in core takes an input by type | `def test_by_type_only(db: Database)` beside a `Database` fixture named `source_db`: ERROR at setup, "fixture 'db' not found" | [A] | PT-M2 |
| `name=` parts the fixture's name from its function's name | "The name of the fixture. This defaults to the name of the decorated function." | [A] | [fixtures.py#L1590-L1596](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/fixtures.py#L1590-L1596); PT-M6 |
| the lifetime is written at the provider | "The scope for which this fixture is shared; one of ``"function"`` (default), ``"class"``, ``"module"``, ``"package"`` or ``"session"``." | [A] | [fixtures.py#L1566-L1571](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/fixtures.py#L1566-L1571); PT-M1 |
| a scope can be computed | "pass a callable to ``scope``. The callable must return a string with a valid scope and will be executed only once - during the fixture definition." | [A] | [how-to/fixtures.rst#L515-L518](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/how-to/fixtures.rst?plain=1#L515-L518); PT-M6 |
| one fixture reached through two names in one test is one object | "Fixtures can be **requested** more than once per test (return values are cached)"; `a is b: True` | [A] | [how-to/fixtures.rst#L284](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/how-to/fixtures.rst?plain=1#L284); PT-M3a |
| one name twice in a signature is refused by Python | `SyntaxError: duplicate argument 'database' in function definition`; the collection error interrupts the session | [A] | PT-M3b |
| a name defined at two levels: the closest wins, with no warning | "The returned FixtureDefs are ordered from least specific (registered higher in the collection tree) to most specific." | [A] | [fixtures.py#L2389-L2391](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/fixtures.py#L2389-L2391), [how-to/fixtures.rst#L1845](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/how-to/fixtures.rst?plain=1#L1845); PT-M5 |
| two `def source_db` in one module: the later one wins | Python rebinds the module's name before pytest collects | [A] run; [C] cause | PT-M5 |
| indirect parametrize gives one fixture one role per run | "Using the ``indirect=True`` parameter when parametrizing a test allows one to parametrize a test with a fixture receiving the values before passing them to a test" | [A] | [example/parametrize.rst#L355-L357](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/example/parametrize.rst?plain=1#L355-L357); PT-M4 |

## 2. pytest-cases 3.10.1 and pytest-lazy-fixtures 1.4.1

pytest-cases was added because `unpack_fixture` names two values that one fixture builds.
pytest-lazy-fixtures was added because `lf("name")` writes each role's fixture in the decorator, above the signature.

```python
# pytest-cases (PC-M1)
@fixture
def pair(source_db: Database, target_db: Database):
    s, d = Tx(source_db), Tx(target_db); yield s, d; s.rollback(); d.rollback()
src, dst = unpack_fixture("src,dst", pair)
def test_copy(src: Tx, dst: Tx) -> None: ...

# pytest-lazy-fixtures (LF-M1); the test id reads test_copy[source_tx-target_tx]
@pytest.mark.parametrize(("src", "dst"), [(lf("source_tx"), lf("target_tx"))])
def test_copy(src: Tx, dst: Tx) -> None: ...
```

| claim | source's own words | tier | source; run |
|---|---|---|---|
| `unpack_fixture` names each element of one fixture | "Creates several fixtures with names `argnames` from the source `fixture`. Created fixtures will correspond to elements unpacked from `fixture` in order." | [A] | [api_reference.md#L480](https://github.com/smarie/python-pytest-cases/blob/3.10.1/docs/api_reference.md?plain=1#L480); PC-M1 |
| `fixture_ref` puts a fixture in a parametrize list | "A reference to a fixture to be used with [`@parametrize`](#parametrize)." | [A] | [api_reference.md#L719](https://github.com/smarie/python-pytest-cases/blob/3.10.1/docs/api_reference.md?plain=1#L719); PC-M1 |
| ⚠️ pytest-cases 3.10.1 does not load on pytest 9.2.0 or 9.1.1; the session exits 3 | `TypeError: IdMaker.__init__() takes 7 positional arguments but 8 were given`; open upstream as [#385](https://github.com/smarie/python-pytest-cases/issues/385) | [A] | PC-M0 |
| `lf` names a fixture inside a parametrize value | "To use your fixtures inside `@pytest.mark.parametrize` you can use `lf` (`lazy_fixture`)." | [A] | [README.md#L27](https://github.com/dev-petrov/pytest-lazy-fixtures/blob/1.4.1/README.md?plain=1#L27); LF-M1 |

## 3. Ward 0.68.0b0 (archived)

Ward's repository is archived; its last commit is 2024-07-05, and its last release 2023-12-18 (Appendix C).

```python
@fixture(scope=Scope.Global)
def source_db() -> Database: ...

@fixture
def source_tx(db: Database = source_db):
    t = Tx(db); yield t; t.rollback()
# target_db and target_tx: the same

@test("copy")
def _(src: Tx = source_tx, dst: Tx = target_tx) -> None: ...      # WD-M1

@test("copy, through using")
@using(src=source_tx, dst=target_tx)
def _(src: Tx, dst: Tx) -> None: ...                               # WD-M3
```

| claim | source's own words | tier | source; run |
|---|---|---|---|
| a test names an input by default value | "By directly binding the fixture as a default argument to our test function, we've told Ward to resolve the fixture and inject it into our test." | [A] | [guide/fixtures.rst#L36](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/docs/source/guide/fixtures.rst?plain=1#L36); WD-M1 |
| the defaults are read with `bind_partial` and `apply_defaults`; `@using` replaces them | `default_binding = signature.bind_partial()`; "Override the signature if @using is present" | [A] | [testing.py#L572-L593](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/testing.py#L572-L593) |
| `@using` names the provider in a decorator | "This approach lets us use positional arguments in our test signature, and declare which fixture each argument refers to using the decorator." | [A] | [guide/fixtures.rst#L44](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/docs/source/guide/fixtures.rst?plain=1#L44); WD-M3 |
| three lifetimes, written at the provider | "Ward supports 3 scopes: ``test`` (default), ``module``, and ``global``." | [A] | [guide/fixtures.rst#L70](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/docs/source/guide/fixtures.rst?plain=1#L70) |
| one fixture is one cache key, so two parameters that name it get one object | "A unique key used to identify fixture in the fixture cache. A string of the form '{path}::{name}'"; `a is b: True` | [A] | [fixtures.py#L41-L47](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/fixtures.py#L41-L47); WD-M2 |
| the annotation is never read | `a: Database = tx` got a `Tx`, and passed | [A] | WD-M2 |

## 4. FastAPI 0.162.0

FastAPI and Litestar are web frameworks, not test runners.
For FastAPI and Litestar, the "test" is one request through the tool's own `TestClient`; "per test" is per request, and "per session" is per app.

```python
@asynccontextmanager
async def lifespan(app: FastAPI):            # the session: one per app, outside Depends
    app.state.source = Database("source"); app.state.target = Database("target"); yield

def source_tx(request: Request):             # per request; exits after the response
    tx = Tx(request.app.state.source); yield tx; tx.rollback()

SourceTx = Annotated[Tx, Depends(source_tx)]
TargetTx = Annotated[Tx, Depends(target_tx)]

@app.post("/copy")
def copy_route(src: SourceTx, dst: TargetTx) -> dict: ...     # FA-M1
```

A generic adapter is a function that makes one dependency per role; each role still needs its own alias, because the cache key is the callable (FA-M7):

```python
def tx_of(get_db):
    def tx(db: Annotated[Database, Depends(get_db)]):
        t = Tx(db); yield t; t.rollback()
    return tx
SourceTx = Annotated[Tx, Depends(tx_of(source_db))]
```

| claim | source's own words | tier | source; run |
|---|---|---|---|
| `Depends` takes the provider, not a type | "It takes a single "dependable" callable (like a function)." | [A] | [param_functions.py#L2391](https://github.com/fastapi/fastapi/blob/0.162.0/fastapi/param_functions.py#L2391); FA-M1 |
| the annotation is not used to resolve | "...doesn't have any special meaning for **FastAPI**. FastAPI won't use it for data conversion, validation, etc. (as it is using the `Depends(CommonQueryParams)` for that)." | [A] | [classes-as-dependencies.md#L162](https://github.com/fastapi/fastapi/blob/0.162.0/docs/en/docs/tutorial/dependencies/classes-as-dependencies.md?plain=1#L162); FA-M3 (`int`, `str` and the default form each got a `Database`) |
| `Depends()` with no callable calls the annotated type | `depends = dataclasses.replace(depends, dependency=type_annotation)`; "It is just a shortcut." | [A] | [utils.py#L496-L500](https://github.com/fastapi/fastapi/blob/0.162.0/fastapi/dependencies/utils.py#L496-L500), [classes-as-dependencies.md#L286](https://github.com/fastapi/fastapi/blob/0.162.0/docs/en/docs/tutorial/dependencies/classes-as-dependencies.md?plain=1#L286); FA-M5 |
| a role is an alias of `Annotated` | "But because we are using `Annotated`, we can store that `Annotated` value in a variable and use it in multiple places" | [A] | [index.md#L127](https://github.com/fastapi/fastapi/blob/0.162.0/docs/en/docs/tutorial/dependencies/index.md?plain=1#L127); FA-M1 |
| one value per callable per request | "it will save the returned value in a … "cache" and pass it to all the "dependants" that need it in that specific request"; the key is `(dependant.call, scopes_for_cache, …)` | [A] | [sub-dependencies.md#L61](https://github.com/fastapi/fastapi/blob/0.162.0/docs/en/docs/tutorial/dependencies/sub-dependencies.md?plain=1#L61), [models.py#L82-L97](https://github.com/fastapi/fastapi/blob/0.162.0/fastapi/dependencies/models.py#L82-L97); FA-M2 |
| `use_cache=False`, at the consumer, calls again | "Set `use_cache` to `False` to disable this behavior and ensure the dependency is called again (if declared more than once) in the same request." | [A] | [param_functions.py#L2407-L2421](https://github.com/fastapi/fastapi/blob/0.162.0/fastapi/param_functions.py#L2407-L2421); FA-M2 |
| a yield dependency's exit is chosen at the consumer | "`"function"`: … end the dependency after the *path operation function* ends, but **before** the response is sent back"; "If not specified and the dependency has `yield`, it will have a `scope` of `"request"` by default." | [A] | [dependencies-with-yield.md#L187-L200](https://github.com/fastapi/fastapi/blob/0.162.0/docs/en/docs/tutorial/dependencies/dependencies-with-yield.md?plain=1#L187-L200); FA-M6 |
| `scope` arrived in 0.121.0 | "In version 0.121.0, FastAPI added support for `Depends(scope="function")` for dependencies with `yield`." | [A] | [advanced-dependencies.md#L81](https://github.com/fastapi/fastapi/blob/0.162.0/docs/en/docs/advanced/advanced-dependencies.md?plain=1#L81) |
| a request-scoped dependency may not depend on a function-scoped one | "The dependency "outer_request" has a scope of "request", it cannot depend on dependencies with scope "function"." (at route definition) | [A] | [utils.py#L335-L345](https://github.com/fastapi/fastapi/blob/0.162.0/fastapi/dependencies/utils.py#L335-L345); FA-M6 |
| an app lifetime lives outside `Depends`: a lifespan, or a cache on the provider | "The `lifespan` parameter of the `FastAPI` app takes an **async context manager**"; "as we are using the `@lru_cache` decorator on top, the `Settings` object will be created only once" | [A] | [events.md#L83](https://github.com/fastapi/fastapi/blob/0.162.0/docs/en/docs/advanced/events.md?plain=1#L83), [settings.md#L251](https://github.com/fastapi/fastapi/blob/0.162.0/docs/en/docs/advanced/settings.md?plain=1#L251); FA-M1, FA-M7 |
| a bare `db: Database` is read as a request body and refused at route definition, with a misleading message | "Invalid args for response field! Hint: check that <class 'subject.Database'> is a valid Pydantic field type."; the same with `response_model=None` | [A] | [utils.py#L519-L535](https://github.com/fastapi/fastapi/blob/0.162.0/fastapi/dependencies/utils.py#L519-L535); FA-M4, FA-M4b |

## 5. Litestar 2.25.0

Litestar was added as the by-name counterpart of FastAPI: it keys each provider by parameter name, on four layers.

```python
def source_tx(source_db: NamedDependency[Database]):    # a provider's own input is found by name too
    tx = Tx(source_db)
    try: yield tx
    finally: tx.rollback()

app = Litestar(route_handlers=[copy_route], dependencies={
    "source_db": Provide(make_source, use_cache=True, sync_to_thread=False),   # the "session"
    "target_db": Provide(make_target, use_cache=True, sync_to_thread=False),
    "src": Provide(source_tx), "dst": Provide(target_tx),                     # per request
})

@post("/copy", sync_to_thread=False)
def copy_route(src: NamedDependency[Tx], dst: NamedDependency[Tx]) -> dict: ...  # LS-M1
```

| claim | source's own words | tier | source; run |
|---|---|---|---|
| a parameter names its input by its name | "the parameter receiving the dependency must be marked with :data:`~litestar.di.NamedDependency`, and its name must match the dependency key." | [A] | [dependency-injection.rst#L78-L79](https://github.com/litestar-org/litestar/blob/v2.25.0/docs/usage/dependency-injection.rst?plain=1#L78-L79); LS-M1 |
| the marker is `Annotated` with a "named" kind; a "type" kind is planned | `NamedDependency: TypeAlias = Annotated[T, Dependency(kind="named")]`; "3.0 will introduce the 'type' kind" | [A] source; [B] the 3.0 plan | [di.py#L30-L43](https://github.com/litestar-org/litestar/blob/v2.25.0/litestar/di.py#L30-L43) |
| the wrapped type validates the value | "the type wrapped by ``NamedDependency`` is used to validate the injected value"; `msgspec.ValidationError: Expected `int`, got `Database``, response 500 | [A] | [dependency-injection.rst#L350-L353](https://github.com/litestar-org/litestar/blob/v2.25.0/docs/usage/dependency-injection.rst?plain=1#L350-L353); LS-M3, LS-M4 |
| name matching without the marker still works, deprecated | "Inferred dependency field 'db'. Mark the field explicitly with 'NamedDependency[…]'." | [A] | [kwargs_model.py#L330-L343](https://github.com/litestar-org/litestar/blob/v2.25.0/litestar/_kwargs/kwargs_model.py#L330-L343); LS-M3 |
| a closer layer overrides a farther one | "The lower scoped dependency therefore overrides the higher scoped one." | [A] | [dependency-injection.rst#L267-L268](https://github.com/litestar-org/litestar/blob/v2.25.0/docs/usage/dependency-injection.rst?plain=1#L267-L268), [handlers/base.py#L350-L361](https://github.com/litestar-org/litestar/blob/v2.25.0/litestar/handlers/base.py#L350-L361); LS-M2 |
| one provider under two keys is refused, so one generic `tx` cannot serve two roles | "Provider for key b is already defined under the different key a. If you wish to override a provider, it must have the same key." | [A] | [handlers/base.py#L520-L527](https://github.com/litestar-org/litestar/blob/v2.25.0/litestar/handlers/base.py#L520-L527); LS-M2, LS-M5 |
| each key is called once per request | "Note that dependencies will only be called once per request, even with ``Provide.use_cache`` set to ``False``." | [A] | [dependency-injection.rst#L301-L304](https://github.com/litestar-org/litestar/blob/v2.25.0/docs/usage/dependency-injection.rst?plain=1#L301-L304); LS-M2 |
| `use_cache=True` memoizes on the `Provide` object | "the return value of the function will be memoized the first time it is called and then will be used"; `if self.use_cache and self.value is not Empty: return self.value` | [A] | [dependency-injection.rst#L301-L302](https://github.com/litestar-org/litestar/blob/v2.25.0/docs/usage/dependency-injection.rst?plain=1#L301-L302), [di.py#L122-L133](https://github.com/litestar-org/litestar/blob/v2.25.0/litestar/di.py#L122-L133); LS-M4 |
| a generator provider cannot be cached | "Cannot cache generator dependency, consider using Lifespan Context instead." | [A] | [di.py#L95-L98](https://github.com/litestar-org/litestar/blob/v2.25.0/litestar/di.py#L95-L98); LS-M4 |
| generator cleanup runs after the handler, before the response | "The cleanup stage is executed **after** the handler function returns, but **before** the response is sent (in case of HTTP requests)" | [A] source; the timing was not run | [dependency-injection.rst#L112-L113](https://github.com/litestar-org/litestar/blob/v2.25.0/docs/usage/dependency-injection.rst?plain=1#L112-L113) |

## 6. injector 0.24.0

For the DI libraries, the "test" is a plain function that the library calls with its inputs filled, and "per test" is the library's narrowest scope.

```python
SourceDb = NewType("SourceDb", Database); SourceTx = NewType("SourceTx", Tx)   # and TargetDb, TargetTx

class Dbs(Module):
    @singleton
    @provider
    def source(self) -> SourceDb: return SourceDb(Database("source"))
    @provider                                   # no scope: a new Tx per injection
    def source_tx(self, db: SourceDb) -> SourceTx: ...

@inject
def test_copy(src: SourceTx, dst: TargetTx) -> None: ...                         # IJ-M1
@inject
def test_copy_ann(src: Annotated[Tx, "source"], dst: Annotated[Tx, "target"]) -> None: ...
```

injector has no teardown; the probe rolled back each `Tx` after the call.
A generic provider `def tx(self, db: D) -> GTx[D]` gave `UnknownProvider` (IJ-M4).

| claim | source's own words | tier | source; run |
|---|---|---|---|
| a binding's key is the type; a second bind replaces the first | `self._bindings[interface] = self.create_binding(interface, to, scope)` | [A] | [`__init__.py#L543`](https://github.com/alecthomas/injector/blob/0.24.0/injector/__init__.py#L543); IJ-M2 |
| the default scope is none: a new value each time | `return Binding(interface, provider, scope or NoScope)`; "An unscoped provider." | [A] | [`__init__.py#L661`](https://github.com/alecthomas/injector/blob/0.24.0/injector/__init__.py#L661), [`#L861-L865`](https://github.com/alecthomas/injector/blob/0.24.0/injector/__init__.py#L861-L865); IJ-M2 |
| a singleton is written at the provider or the class | "Singletons are declared by binding them in the SingletonScope. This can be done in three ways: 1. Decorating the class with `@singleton`. 2. Decorating a `@provider` decorated Module method with `@singleton`. 3. Explicitly calling `binder.bind(X, scope=singleton)`." | [A] | [docs/scopes.rst#L9-L13](https://github.com/alecthomas/injector/blob/0.24.0/docs/scopes.rst?plain=1#L9-L13) |
| a request scope is left to the user | "For scopes with a transient lifetime, such as those tied to HTTP requests, the usual solution is to use a thread or greenlet-local cache inside the scope." | [A] | [docs/scopes.rst#L64](https://github.com/alecthomas/injector/blob/0.24.0/docs/scopes.rst?plain=1#L64) |
| `NewType` and `Annotated` are keys of their own | 0.14.0: "Added NewType support"; 0.22.0: "Added support for injecting `PEP 593 …`_ `Annotated …`_" | [A] | [CHANGES#L174](https://github.com/alecthomas/injector/blob/0.24.0/CHANGES?plain=1#L174), [CHANGES#L30-L31](https://github.com/alecthomas/injector/blob/0.24.0/CHANGES?plain=1#L30-L31); IJ-M1 |
| the marker survives into the key | `get_type_hints(…, include_extras=True)` | [A] | [`__init__.py#L1329`](https://github.com/alecthomas/injector/blob/0.24.0/injector/__init__.py#L1329) |
| ⚠️ a role that nothing binds is built from its base class | `return type_.__supertype__` … `return type_.__origin__`; with `Database` bound as a singleton, an unbound `SourceDb` got a fresh `Database('db')` | [A] | [`__init__.py#L793-L806`](https://github.com/alecthomas/injector/blob/0.24.0/injector/__init__.py#L793-L806); IJ-M5 |
| an unbound class is built by default | ":param auto_bind: Whether to automatically bind missing types."; `auto_bind=False` refuses at the call | [A] | [`__init__.py#L486`](https://github.com/alecthomas/injector/blob/0.24.0/injector/__init__.py#L486); IJ-M2, IJ-M5 |

## 7. dishka 1.10.1

```python
class Dbs(Provider):
    @provide(scope=Scope.APP)
    def source(self) -> SourceDb: return SourceDb(Database("source"))
    @provide(scope=Scope.REQUEST)
    def source_tx(self, db: SourceDb) -> Iterator[SourceTx]:
        tx = Tx(db); yield SourceTx(tx); tx.rollback()

def test_copy(src: FromDishka[SourceTx], dst: FromDishka[TargetTx]) -> None: ...      # DK-M1
run = wrap_injection(func=test_copy, container_getter=lambda a, k: container, manage_scope=True)

class Role(Provider):              # one instance per component: Role("source"), Role("target")
    @provide(scope=Scope.APP)
    def db(self) -> Database: ...
    @provide(scope=Scope.REQUEST)
    def tx(self, db: Database) -> Iterator[Tx]: ...   # finds Database in its own component

def test_copy_c(src: Annotated[Tx, FromComponent("source")], dst: Annotated[Tx, FromComponent("target")]) -> None: ...

@provide(scope=Scope.REQUEST)                         # one generic adapter for every role (DK-M4, DK-M5)
def tx(self, db: D) -> Iterator[GTx[D]]: ...
def test_copy_g(src: FromDishka[GTx[SourceDb]], dst: FromDishka[GTx[TargetDb]]) -> None: ...
```

| case of the generic adapter | result | run |
|---|---|---|
| subclass roles, `D = TypeVar("D", bound=Database)` | passed twice; counts 2, 4, 4 | DK-M4 |
| `NewType` roles, `TypeVar` with no bound | passed twice; counts 2, 4, 4 | DK-M5 |
| `NewType` roles, `TypeVar(bound=Database)` (the shape of #690) | refused at the call: `TypeError: issubclass() arg 1 must be a class` | DK-M4, re-run in this session |

| claim | source's own words | tier | source; run |
|---|---|---|---|
| `FromDishka[T]` is a marker over the type | `return Annotated[item, FromComponent()]` | [A] | [depends_marker.py#L34-L35](https://github.com/reagento/dishka/blob/1.10.1/src/dishka/entities/depends_marker.py#L34-L35) |
| a role by `NewType` is the first documented answer | "While they have special meaning you distinguish them by creating new types … `MainDbConnection = NewType("MainDbConnection", Connection)`" | [A] | [components.rst#L19-L30](https://github.com/reagento/dishka/blob/1.10.1/docs/advanced/components.rst?plain=1#L19-L30); DK-M1 |
| a component is a string that isolates a group of providers | "**Component** - is an isolated group of providers within the same container identified by a string." | [A] | [components.rst#L43-L44](https://github.com/reagento/dishka/blob/1.10.1/docs/advanced/components.rst?plain=1#L43-L44); DK-M1 |
| a provider finds its own component first | "If the same type is provided in multiple components, it is searched only within the same component as its dependant, unless it is declared explicitly." | [A] | [components.rst#L140-L141](https://github.com/reagento/dishka/blob/1.10.1/docs/advanced/components.rst?plain=1#L140-L141); DK-M1 |
| the lifetime is set at the provider, and the caller enters the scope | "Firstly, when creating provider, your attach dependency to its scope. Then, when you use container you enter scope and dependency is retained once it is requested until you exit that scope." | [A] | [scopes.rst#L7](https://github.com/reagento/dishka/blob/1.10.1/docs/advanced/scopes.rst?plain=1#L7) |
| the scope ladder | "``[RUNTIME]`` → ``APP`` → ``[SESSION]`` → ``REQUEST`` → ``ACTION`` → ``STEP``" | [A] | [scopes.rst#L17](https://github.com/reagento/dishka/blob/1.10.1/docs/advanced/scopes.rst?plain=1#L17) |
| a generator provider is torn down at scope exit | "It supports finalization of dependency if you make it a generator." | [A] | [provide.rst#L6](https://github.com/reagento/dishka/blob/1.10.1/docs/provider/provide.rst?plain=1#L6); DK-M1 |
| one value per key per scope, by default | "By default the result is cached within scope. You can disable it providing ``cache=False`` argument." | [A] | [provide.rst#L13](https://github.com/reagento/dishka/blob/1.10.1/docs/provider/provide.rst?plain=1#L13); DK-M2 |
| a second provider of one key replaces the first, unless strict | `if factory.when_override in (None, BoolMarker(True)): res_factories = []`; `implicit_override: bool = False` | [A] | [uniter.py#L62-L64](https://github.com/reagento/dishka/blob/1.10.1/src/dishka/graph_builder/uniter.py#L62-L64), [validation_settings.py#L9-L19](https://github.com/reagento/dishka/blob/1.10.1/src/dishka/entities/validation_settings.py#L9-L19); DK-M2 |
| the strict refusal | "It means that you have 2 factories for the same type without specifying that the second one should replace the first one." | [A] | [errors.rst#L167-L168](https://github.com/reagento/dishka/blob/1.10.1/docs/errors.rst?plain=1#L167-L168); DK-M2 |
| a generic provider is documented, and its bound is checked | "You can create generic factories, use ``type[T]`` to access resolved value of ``TypeVar``. ``TypeVar`` can have bound or constraints, which are checked." | [A] | [generics.rst#L16](https://github.com/reagento/dishka/blob/1.10.1/docs/advanced/generics.rst?plain=1#L16) |
| the bound check calls `issubclass`, which a `NewType` fails | `return issubclass(t2, bound1)` | [A] | [type_match.py#L60-L72](https://github.com/reagento/dishka/blob/1.10.1/src/dishka/dependency_source/type_match.py#L60-L72); DK-M4 |

## 8. lagom 2.7.7

```python
container = Container()
container[SourceDb] = Singleton(lambda: SourceDb(Database("source")))

@context_dependency_definition(container)
def _source_tx(c: Container) -> Iterator[SourceTx]:
    tx = Tx(c[SourceDb])
    try: yield SourceTx(tx)
    finally: tx.rollback()

per_test = ContextContainer(container, context_types=[SourceTx, TargetTx])

@bind_to_container(per_test)
def test_copy(src: SourceTx = injectable, dst: TargetTx = injectable) -> None: ...   # LG-M1
```

A generic definition `def tx(db: D) -> GTx[D]` is accepted, and the call fails: `UnresolvableType … GTx[__main__.SourceDb]` (LG-M4).

| claim | source's own words | tier | source; run |
|---|---|---|---|
| a second definition of one type is refused | `if dep in self._registered_types: raise DuplicateDefinition()`; "The type has already been defined somewhere else" | [A] | [container.py#L168-L169](https://github.com/meadsteve/lagom/blob/2.7.7/lagom/container.py#L168-L169), [exceptions.py#L53-L54](https://github.com/meadsteve/lagom/blob/2.7.7/lagom/exceptions.py#L53-L54); LG-M2 |
| `NewType` is the documented role | `Now = NewType("Now", datetime)` … `container[Now] = lambda: datetime.now()` | [A] | [cookbook.md#L67-L70](https://github.com/meadsteve/lagom/blob/2.7.7/docs/cookbook.md?plain=1#L67-L70); LG-M1 |
| a singleton is written in the container config | `container[AnotherDependency] = Singleton(lambda c: AnotherDependency("Some Config"))` | [A] | [explicit_definitions.md#L37](https://github.com/meadsteve/lagom/blob/2.7.7/docs/explicit_definitions.md?plain=1#L37) |
| per-call teardown comes from a `ContextContainer` | "Lagom provides the `ContextContainer` to help with this. It takes an existing lagom container and a list of resources that should be managed within the context." | [A] | [clean_up.md#L6-L7](https://github.com/meadsteve/lagom/blob/2.7.7/docs/clean_up.md?plain=1#L6-L7); LG-M1 |
| ⚠️ injection drops an `Annotated` marker | `annotations = get_type_hints(func)`, with no `include_extras`; `reflect(t): (Database)`; the defined `Annotated[Database, "source"]` is reachable only as `c[Annotated[…]]` | [A] | [reflection.py#L98](https://github.com/meadsteve/lagom/blob/2.7.7/lagom/util/reflection.py#L98); LG-M5, re-run in this session |
| 2.7.7 changes no code | "No code change - only build infrastructure" | [A] | [CHANGELOG.md#L2-L3](https://github.com/meadsteve/lagom/blob/2.7.7/CHANGELOG.md?plain=1#L2-L3) |

## 9. svcs 26.2.0

```python
registry.register_value(SourceDb, SourceDb(Database("source")))     # once per registry
registry.register_factory(SourceTx, source_tx)                      # generator: once per container

def test_copy(container: svcs.Container) -> None:                    # SV-M1: locator
    src, dst = container.get(SourceTx, TargetTx)

def test_copy(src: SourceTx, dst: TargetTx) -> None: ...             # SV-M2: autowire
svcs.autowire(test_copy)(container)                                 # the caller supplies the container
```

No generic adapter: each role needs its own factory.

| claim | source's own words | tier | source; run |
|---|---|---|---|
| two of one type: new types as keys | "You can achieve this by creating new types by subclassing the common type or by using {any}`typing.NewType` or {any}`typing.Annotated`." | [A] | [core-concepts.md#L42](https://github.com/hynek/svcs/blob/26.2.0/docs/core-concepts.md?plain=1#L42); SV-M1 |
| the type-checker caveat | "However, the latter two don't work with Pyright and for `Annotated` [it's unclear if it's even _supposed_ to work]" | [B]: Pyright not run | [core-concepts.md#L44](https://github.com/hynek/svcs/blob/26.2.0/docs/core-concepts.md?plain=1#L44) |
| a `type` alias cannot be a key | "The {pep}`695` {keyword}`type` keyword is currently not supported" | [B] | [core-concepts.md#L86](https://github.com/hynek/svcs/blob/26.2.0/docs/core-concepts.md?plain=1#L86) |
| a registry lives as long as the app | "A registry should live as long as your application lives and there should be only one per application." | [A] | [core-concepts.md#L12](https://github.com/hynek/svcs/blob/26.2.0/docs/core-concepts.md?plain=1#L12); SV-M1 |
| a container lives as long as the caller chooses | "A container lives as long as you want the instances within to live – for example, as long as a request lives." | [A] | [core-concepts.md#L137](https://github.com/hynek/svcs/blob/26.2.0/docs/core-concepts.md?plain=1#L137); SV-M1 |
| teardown on `close()` | "If a factory is a [generator] that *yields* the instance instead of returning it, it will be wrapped in a context manager automatically. At the end, you run {meth}`svcs.Container.close()` and all context managers will be exited." | [A] | [core-concepts.md#L177-L178](https://github.com/hynek/svcs/blob/26.2.0/docs/core-concepts.md?plain=1#L177-L178); SV-M1 |
| a second factory for one key replaces the first | "Repeated registrations overwrite previous ones" | [A] | [_core.py#L292](https://github.com/hynek/svcs/blob/26.2.0/src/svcs/_core.py#L292); SV-M3 |
| signature injection is new | "New `svcs.autowire()` and `svcs.aautowire()` that can be used to automatically resolve dependencies based on type annotations." (26.1.0) | [A] | [CHANGELOG.md#L68](https://github.com/hynek/svcs/blob/26.2.0/CHANGELOG.md?plain=1#L68) |
| autowire looks up the annotation verbatim | "a parameter annotated with, for example, `Annotated[Database, \"primary\"]` or a `NewType` is resolved only if the service is registered under exactly that type – not under the plain `Database`." | [A] | [autowiring.md#L99-L100](https://github.com/hynek/svcs/blob/26.2.0/docs/autowiring.md?plain=1#L99-L100); SV-M6 |
| ⚠️ a default hides a missing service | "If a parameter cannot be resolved because the service has not been registered, the default value is injected instead." | [A] | [autowiring.md#L69-L70](https://github.com/hynek/svcs/blob/26.2.0/docs/autowiring.md?plain=1#L69-L70); SV-M6 |
| no pytest integration | no hit for `pytest` or `fixture` in `src/svcs` | [A] | SV-S1 |

## 10. wireup 2.13.0

```python
@injectable(qualifier="source")                       # singleton: once per container
def source_db() -> Database: ...

@injectable(lifetime="scoped", qualifier="source")    # once per scope; cleanup at scope exit
def source_tx(db: Annotated[Database, Inject(qualifier="source")]) -> Iterator[Tx]: ...

@inject_from_container(container)                     # one scope per call
def test_copy(src: Annotated[Tx, Inject(qualifier="source")], dst: Annotated[Tx, Inject(qualifier="target")]) -> None: ...  # WU-M1

SourceTx = Annotated[Tx, Inject(qualifier="source")]  # the alias its docs advise (WU-M8)
def test_copy(src: SourceTx, dst: TargetTx) -> None: ...
```

Under pytest 9.2.0 the decorated test runs only with `hide_annotated_names=True`; without it, pytest reports `fixture 'src' not found` (WU-M2b).
A `TypeVar` factory was refused: "has an unknown dependency on ~D" (WU-M3).

| claim | source's own words | tier | source; run |
|---|---|---|---|
| a marker selects the injected parameters | "Only parameters annotated with `Injected[T]` or `Annotated[T, Inject(...)]` are injected. Unannotated parameters are left alone for the caller to provide." | [A] | [function_injection.md#L39-L40](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/function_injection.md?plain=1#L39-L40); WU-M1 |
| two of one type: qualifiers | "When you have multiple implementations of the same type in the same container, use **qualifiers** to distinguish between them." | [A] | [interfaces.md#L94-L96](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/interfaces.md?plain=1#L94-L96); WU-M1 |
| the key is the type plus the qualifier | `return klass if qualifier is None else (klass, qualifier)` | [A] | [types.py#L58](https://github.com/maldoinc/wireup/blob/v2.13.0/wireup/ioc/types.py#L58) |
| two of one type: `NewType` | "Use `typing.NewType` to create distinct types so each primitive can be requested independently." | [A] | [factories.md#L75-L76](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/factories.md?plain=1#L75-L76); WU-M3 |
| an alias hides the qualifier | "If you repeatedly inject the same qualified dependency, consider creating a type alias once and reusing it" | [A] | [interfaces.md#L138](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/interfaces.md?plain=1#L138); WU-M8 |
| the lifetime is written at the provider | "Wireup provides three lifetime options that control instance creation and sharing. Configure the lifetime using the `@injectable` decorator." | [A] | [lifetimes_and_scopes.md#L16-L17](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/lifetimes_and_scopes.md?plain=1#L16-L17); WU-M1, WU-M3 |
| the three lifetimes | "Singleton \| Once per container"; "Scoped \| Once per scope"; "Transient \| Every resolution" | [A] | [lifetimes_and_scopes.md#L23-L25](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/lifetimes_and_scopes.md?plain=1#L23-L25); WU-M3 |
| the decorator opens one scope per call | "Creates a new scope before the function runs"; "Closes the scope when the function returns (triggering cleanup)" | [A] | [function_injection.md#L23-L25](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/function_injection.md?plain=1#L23-L25); WU-M1 |
| a duplicate is refused at container creation | "Wireup rejects two registrations for the same type and qualifier if they come from different sources."; `DuplicateServiceRegistrationError: Cannot register type <class 'subject.Database'> as it already exists.` | [A] | [what_wireup_validates.md#L32](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/what_wireup_validates.md?plain=1#L32); WU-M3 |
| an unqualified request, with only qualified providers, is refused at decoration | "`@inject_from_container(...)` functions are checked at module import time."; "Parameter 'a' … has an unknown dependency on <class 'subject.Database'>." | [A] | [what_wireup_validates.md#L21](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/what_wireup_validates.md?plain=1#L21); WU-M3 |
| an unqualified provider beside a qualified one is the default | "When `Cache` is requested without a qualifier, the default will be injected." | [A] | [interfaces.md#L156-L157](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/interfaces.md?plain=1#L156-L157); WU-M8 |
| generics are concrete subclasses, not generic factories | "you can keep the common logic in a generic base class and register concrete subclasses that bind the actual type." | [A] | [generic_dependencies.md#L2](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/generic_dependencies.md?plain=1#L2); WU-M3 |
| no pytest plugin | "expose the app container through pytest fixtures, and apply overrides there." | [A] | [testing.md#L9](https://github.com/maldoinc/wireup/blob/v2.13.0/docs/pages/testing.md?plain=1#L9) |

## 11. dependency-injector 4.49.1

Added because the consumer names the provider by a container attribute, and the type is never read.

```python
class Container(containers.DeclarativeContainer):
    source_db = providers.Singleton(Database, "source")
    target_db = providers.Singleton(Database, "target")
    source_tx = providers.Resource(open_tx, db=source_db)   # open_tx: a generator, yield then rollback
    target_tx = providers.Resource(open_tx, db=target_db)

@inject
def test_copy(src: Tx = Closing[Provide[Container.source_tx]], dst: Tx = Closing[Provide[Container.target_tx]]) -> None: ...  # DI-M1
```

| claim | source's own words | tier | source; run |
|---|---|---|---|
| the marker names what to inject | "Wiring marker specifies what dependency to inject, e.g. ``Provide[Container.bar]``." | [A] | [wiring.rst#L11-L12](https://github.com/ets-labs/python-dependency-injector/blob/4.49.1/docs/wiring.rst?plain=1#L11-L12) |
| the marker is read from the default, or from `Annotated` metadata; the type is never read | `if get_origin(parameter.annotation) is Annotated: candidates = get_args(parameter.annotation)[1:]` / `else: candidates = (parameter.default,)`; `a: int = Provide[Container.other_db]` got a `Database` | [A] | [wiring.py#L705-L717](https://github.com/ets-labs/python-dependency-injector/blob/4.49.1/src/dependency_injector/wiring.py#L705-L717); DI-M1, DI-M2 |
| `Singleton` keeps the first object | "It memorizes the first created object and returns it on the rest of the calls." | [A] | [singleton.rst#L16-L17](https://github.com/ets-labs/python-dependency-injector/blob/4.49.1/docs/providers/singleton.rst?plain=1#L16-L17); DI-M2 |
| `Factory` makes a new object | ":py:class:`Factory` provider creates new objects." | [A] | [factory.rst#L16](https://github.com/ets-labs/python-dependency-injector/blob/4.49.1/docs/providers/factory.rst?plain=1#L16); DI-M2 |
| a per-call lifetime is written at the consumer, with `Closing` | "You can compound ``Resource`` provider with :ref:`wiring` to implement per-function execution scope." … "With the ``Closing`` marker framework calls resource ``shutdown()`` method when function execution is over." | [A] | [resource.rst#L327-L337](https://github.com/ets-labs/python-dependency-injector/blob/4.49.1/docs/providers/resource.rst?plain=1#L327-L337); DI-M1, DI-M4 |
| a per-context lifetime exists | "objects are context local singletons - the same context will receive the same instance, but different execution contexts will have their own separate instances." | [A] source; not run | [context_local_resource.rst#L18-L19](https://github.com/ets-labs/python-dependency-injector/blob/4.49.1/docs/providers/context_local_resource.rst?plain=1#L18-L19) |
| ⚠️ a marker that names a type is not resolved | `value=Provide[<class 'subject.Database'>] type=Provide isinstance Database=False`; only `wire(warn_unresolved=True)` warns | [A] | [wiring.py#L772-L777](https://github.com/ets-labs/python-dependency-injector/blob/4.49.1/src/dependency_injector/wiring.py#L772-L777); DI-M2, DI-M3 |

## 12. rodi 2.1.0, punq 0.9.0 and that-depends 4.2.1

rodi was added because it resolves by type, and also by parameter name.
punq was added because it keeps a list of registrations per key, and shows what "picks one" looks like.
that-depends was read only: it has dependency-injector's shape.

```python
# rodi (RD-M1): role by subclass of Tx, by type; or by an unannotated parameter name
class SourceTx(Tx):
    def __init__(self, db: SourceDb) -> None: super().__init__(db)
container.add_singleton_by_factory(lambda: Database("source"), SourceDb)
container.add_scoped(SourceTx)
@inject()
def test_copy(src: SourceTx, dst: TargetTx) -> None: ...
@inject()
def test_copy_by_name(source_tx, target_tx) -> None: ...

# punq (PQ-M1): a class constructor only
container.register(SourceDb, factory=lambda: Database("source"), scope=punq.Scope.singleton)
container.register(SourceTx, SourceTxImpl)        # transient: one per resolve
class TestCopy:
    def __init__(self, src: SourceTx, dst: TargetTx) -> None: ...
```

| tool | claim | source's own words | tier | source; run |
|---|---|---|---|---|
| rodi | three ways to resolve | "types resolution by signature types annotations (_type hints_)" … "types resolution by names and aliases (_convention over configuration_)" | [A] | [README.md#L12-L14](https://github.com/Neoteroi/rodi/blob/v2.1.0/README.md?plain=1#L12-L14); RD-M1 |
| rodi | an annotated parameter resolves by its annotation; an unannotated one by its name | `if param.annotation is _empty: … return self.get(key, context)` / `else: … return self.get(param.annotation, context)` | [A] | [\_\_init\_\_.py#L1040-L1049](https://github.com/Neoteroi/rodi/blob/v2.1.0/rodi/__init__.py#L1040-L1049); RD-M1, RD-M2 |
| rodi | registering a class makes name aliases | `self._aliases[to_standard_param_name(key_name)].add(key)` | [A] | [\_\_init\_\_.py#L1299-L1314](https://github.com/Neoteroi/rodi/blob/v2.1.0/rodi/__init__.py#L1299-L1314); RD-M1 |
| rodi | a second provider for one key is refused | "A service with key 'Database' is already registered and would be overridden by value …" | [A] | RD-M2, re-run in this session |
| rodi | an ambiguous alias: a constructor refuses with an `assert`; a function takes the first of a set | `assert (len(aliases) == 1), "Configured aliases cannot be ambiguous"`; `for _type in _types: break` | [A] | [\_\_init\_\_.py#L585-L600](https://github.com/Neoteroi/rodi/blob/v2.1.0/rodi/__init__.py#L585-L600), [#L1576-L1578](https://github.com/Neoteroi/rodi/blob/v2.1.0/rodi/__init__.py#L1576-L1578); RD-M5, RD-M6 |
| rodi | a scope is one `exec` call; no teardown | `with self.create_scope(scoped) as context: return method(*[fn(context) for fn in fns])`; `dispose` only clears references | [A] | [\_\_init\_\_.py#L1082-L1083](https://github.com/Neoteroi/rodi/blob/v2.1.0/rodi/__init__.py#L1082-L1083), [#L279-L285](https://github.com/Neoteroi/rodi/blob/v2.1.0/rodi/__init__.py#L279-L285); RD-M1 |
| punq | a constructor's needs come from `__init__` hints | `return get_type_hints(cls.__init__, None, self._localns)` | [A] | [\_\_init\_\_.py#L222-L224](https://github.com/bobthemighty/punq/blob/v0.9.0/punq/__init__.py#L222-L224); PQ-M1 |
| punq | a function factory with a typed parameter is not filled | `TypeError source_tx_factory() missing 1 required positional argument: 'db'` | [A] run; [C] cause (the line above reads the function's `__init__`) | PQ-M1 |
| punq | two lifetimes | "transient: create a fresh instance for each `resolve` call" / "singleton: re-use a single instance for every `resolve` call" | [A] | [\_\_init\_\_.py#L159-L168](https://github.com/bobthemighty/punq/blob/v0.9.0/punq/__init__.py#L159-L168); PQ-M2 |
| punq | ⚠️ within one `resolve`, a key is built once, even for a transient | `if registration.cache: context[registration.service] = result` | [A] | [\_\_init\_\_.py#L605-L606](https://github.com/bobthemighty/punq/blob/v0.9.0/punq/__init__.py#L605-L606); PQ-M2 |
| punq | ⚠️ a second provider: the last wins; `resolve_all` returns both | `impl = self.impls.pop()` | [A] | [\_\_init\_\_.py#L384-L390](https://github.com/bobthemighty/punq/blob/v0.9.0/punq/__init__.py#L384-L390); PQ-M2 |
| punq | per-request values: a child container | "Punq supports "child" containers for this purpose." | [A] source; not run | [\_\_init\_\_.py#L677-L684](https://github.com/bobthemighty/punq/blob/v0.9.0/punq/__init__.py#L677-L684) |
| that-depends | marker, as dependency-injector | "mark certain parameters as `Provide[...]`" | [B] | [injection.md#L3](https://github.com/modern-python/that-depends/blob/4.2.1/docs/introduction/injection.md?plain=1#L3) |
| that-depends | an opt-in by-type form | "you need to bind providers to the type they will provide", then `v: float = Provide()` | [B] | [type-based-injection.md#L8-L19](https://github.com/modern-python/that-depends/blob/4.2.1/docs/introduction/type-based-injection.md?plain=1#L8-L19) |

## Across the survey

These hold across the 14 tools run, not one tool.

| observation | tools | tier |
|---|---|---|
| No tool refuses a test that names two values of one type with no role | all 14; the nearest is Litestar, which refuses one provider under two keys when the app is built (LS-M2) | [A] |
| With no role, one object or two is decided by the provider's lifetime, not by the test | injector, dishka, lagom, wireup, dependency-injector, rodi, punq; FastAPI by `use_cache`, which the consumer writes | [A] |
| Two parameters that reach one provider get one object, whatever its lifetime | pytest, Ward, FastAPI (default), svcs, punq (within one resolve) | [A] |
| The lifetime is written at the provider or in the container config | all 14 | [A] |
| Part of the lifetime can also be written at the consumer | FastAPI (`use_cache`, `scope`), dependency-injector (`Closing`) | [A] |
| No tool shows the lifetime on the test's line, unless the consumer writes it there | all 14 | [A] |
| The test's line names the provider | Ward, FastAPI (inline), dependency-injector, pytest-lazy-fixtures (in the decorator) | [A] |
| The annotation is never read, so it can lie | pytest, Ward, FastAPI, dependency-injector | [A] |
| A role is a new key: `NewType` | injector, dishka, lagom, svcs, wireup, rodi, punq | [A] |
| A role is a marker string | injector (`Annotated`), svcs (`Annotated` key), dishka (component), wireup (qualifier) | [A] |
| A role written as an `Annotated` marker can be silently dropped | lagom | [A] |
| A role that nothing provides is built from the base class, silently | injector (auto-bind) | [A] |
| A second provider for one key is refused | lagom, wireup, rodi; dishka under `STRICT_VALIDATION` | [A] |
| A second provider for one key silently wins | injector, svcs, punq, dishka (default): the last; pytest, Litestar: the closest | [A] |
| One generic adapter provider serves every role | dishka only, with subclass roles or an unbound `TypeVar` | [A] |
| The tool tears a per-test value down by itself | pytest, Ward, FastAPI, Litestar, dishka, lagom, svcs, wireup, dependency-injector (with `Closing`) | [A] |
| The tool has no teardown | injector, rodi, punq | [A] |
| An alias shortens the line and hides the provider | FastAPI, wireup | [A] |

## The distinct ways found

Each way, with the copy test's line, for #700 to prototype.
The order is the order of the sections, not a ranking.

| way | the line | tools |
|---|---|---|
| role by parameter name | `def test_copy(src: Tx, dst: Tx)` | pytest, Litestar |
| role by parameter name, mapped to a provider in a decorator | `@using(src=source_tx, dst=target_tx)` or `@parametrize(("src", "dst"), [(lf("source_tx"), lf("target_tx"))])` | Ward, pytest-lazy-fixtures |
| role by tuple position | `src, dst = unpack_fixture("src,dst", pair)` | pytest-cases |
| role by provider, as the default value | `def _(src: Tx = source_tx, dst: Tx = target_tx)` | Ward |
| role by provider, in an `Annotated` marker | `def copy_route(src: Annotated[Tx, Depends(source_tx)], dst: Annotated[Tx, Depends(target_tx)])` | FastAPI |
| role by an alias that hides a marker | `def test_copy(src: SourceTx, dst: TargetTx)`, where `SourceTx = Annotated[Tx, …]` | FastAPI, wireup |
| role by container attribute | `def test_copy(src: Tx = Provide[Container.source_tx], dst: Tx = Provide[Container.target_tx])` | dependency-injector; that-depends (read, not run) |
| role by `NewType` (or subclass) of the adapter | `def test_copy(src: SourceTx, dst: TargetTx)` | injector, dishka, lagom, svcs, wireup, rodi, punq |
| role by `NewType` of the stock, in one generic adapter | `def test_copy(src: FromDishka[GTx[SourceDb]], dst: FromDishka[GTx[TargetDb]])` | dishka; purba #690 |
| role by `Annotated` marker string | `def test_copy(src: Annotated[Tx, "source"], dst: Annotated[Tx, "target"])` | injector, svcs |
| role by component string | `def test_copy(src: Annotated[Tx, FromComponent("source")], dst: Annotated[Tx, FromComponent("target")])` | dishka |
| role by qualifier string | `def test_copy(src: Annotated[Tx, Inject(qualifier="source")], dst: Annotated[Tx, Inject(qualifier="target")])` | wireup |
| role by an unannotated parameter name | `def test_copy(source_tx, target_tx)` | rodi |
| role in the body, through a locator | `src, dst = container.get(SourceTx, TargetTx)` | svcs |

## Appendix A. Commands

The scratch directory was `$R=/home/snregales/.claude/jobs/f70fe198/tmp/r699`.
Each tool had its own directory, and every run saved its output to `runs/<key>.txt`.
Sources (S) were cloned with `git clone --depth 1 --branch <tag>` into their own directory.
Unless noted, a run is `uv run --no-project --python 3.13 --with <pkg>==<ver> python -I <probe> $R`.
The pytest and Ward runs used a venv and `PYTHONPATH=$R` without `-I`, because `-I` drops `PYTHONPATH`.

### A.0 The shared scenario, `subject.py`

Every probe imported it unchanged.

```python
COUNTS = {"db_built": 0, "tx_opened": 0, "tx_rolled_back": 0}

class Database:
    def __init__(self, name: str = "db") -> None:
        COUNTS["db_built"] += 1; self.name = name; self.id = next(_ids); self.rows: list[str] = []

class Tx:
    def __init__(self, db: Database) -> None:
        COUNTS["tx_opened"] += 1; self.db = db; self.pending: list[str] = list(db.rows)
    def read(self) -> list[str]: return list(self.pending)
    def write(self, rows: list[str]) -> None: self.pending.extend(rows)
    def rollback(self) -> None:
        COUNTS["tx_rolled_back"] += 1; self.pending = list(self.db.rows)

def copy(src: Tx, dst: Tx) -> None:
    dst.write(src.read())

def check_copy(src: Tx, dst: Tx) -> None:
    assert src.db is not dst.db, f"one database twice: {src.db!r}"
    assert dst.read() == [], f"target not clean at start: {dst.read()}"
    src.write(["a", "b"]); copy(src, dst)
    assert dst.read() == ["a", "b"], f"copy failed: {dst.read()}"
```

### A.1 Runs

| key | what it runs |
|---|---|
| PT-M1 | pytest: the scenario; `python -m pytest -v -p no:cacheprovider -s` |
| PT-M2 | pytest: a parameter named by type only; an annotation that lies (`-rA`) |
| PT-M3a, M3b | pytest: one fixture under two names; one name twice in a signature |
| PT-M4 | pytest: three generic-adapter forms |
| PT-M5 | pytest: a fixture name at two levels, and twice in one module; `-W error::pytest.PytestWarning` |
| PT-M6 | pytest: `name=`, and a computed scope |
| PC-M0 | pytest-cases 3.10.1 on pytest 9.2.0: plugin load |
| PC-M1 | pytest-cases 3.10.1 on pytest 9.0.2: `unpack_fixture` and `fixture_ref` |
| LF-M1 | pytest-lazy-fixtures 1.4.1 on pytest 9.2.0: the scenario through `lf` |
| WD-M1–M3 | Ward: the scenario; no role, a lying annotation, a parameter with no default; `@using`; `python -m ward test --path . --test-output-style test-per-line --progress-style none --no-capture-output` |
| FA-M1 | FastAPI: the scenario, with a lifespan and two yield providers, two requests through `TestClient` |
| FA-M2 | FastAPI: one callable twice; `use_cache=False`; the default-value form |
| FA-M3 | FastAPI: `Annotated[int, Depends(get_db)]`, `x: int = Depends(get_db)`, `Annotated[str, …]` |
| FA-M4, M4b | FastAPI: a bare `db: Database`; the same with `response_model=None`; `?name=` on the shortcut |
| FA-M5 | FastAPI: `Annotated[Database, Depends()]` twice, and its OpenAPI parameters |
| FA-M6 | FastAPI: exit order of `scope="function"` and the default; reverse nesting |
| FA-M7 | FastAPI: the generic adapter `tx_of(get_db)` |
| LS-M1 | Litestar: the scenario |
| LS-M2 | Litestar: one provider under two keys; two functions; one key read twice; one key on two layers |
| LS-M3 | Litestar: `NamedDependency[int]` fed a `Database`; unmarked name; marker with no key |
| LS-M4 | Litestar: M3 with `debug=True`; `use_cache=True` across sessions and apps; on a generator |
| LS-M5 | Litestar: one generic `tx(db)` under `src` and `dst` |
| IJ-M1 | injector: the scenario, by `NewType` and by `Annotated` |
| IJ-M2 | injector: no role (unbound, no scope, singleton, `auto_bind=False`); a duplicate bind, in one module and in two |
| IJ-M4 | injector: a generic provider |
| IJ-M5 | injector: an unbound `NewType` and `Annotated` role, with and without a `Database` singleton bound; `auto_bind=False` (added in this session's red team) |
| DK-M1 | dishka: the scenario, by `NewType` and by component, through `wrap_injection(…, manage_scope=True)` |
| DK-M2 | dishka: no role at `APP` and `REQUEST`, with and without `cache`; no provider; a duplicate provider, default and `STRICT_VALIDATION` |
| DK-M4 | dishka: a generic adapter with `TypeVar(bound=Database)`, `NewType` roles then subclass roles |
| DK-M5 | dishka: the same with an unbound `TypeVar` and `NewType` roles |
| LG-M1 | lagom: the scenario |
| LG-M2 | lagom: no role (nothing defined, plain, `Singleton`, `magic_bind`); a duplicate definition (LG-M3); `Annotated` roles (LG-M3b) |
| LG-M4 | lagom: a generic definition |
| LG-M5 | lagom: an `Annotated` key read directly, and through reflection |
| SV-M1, M2 | svcs: the scenario, as a locator and through `autowire` |
| SV-M3 | svcs: no role; a duplicate factory; an unregistered type |
| SV-M6 | svcs: verbatim lookup, and a default in place of a missing service |
| WU-M1 | wireup: the scenario, two calls of a decorated test |
| WU-M2, M2b | wireup under pytest 9.2.0, with and without `hide_annotated_names=True` |
| WU-M3 | wireup: no role by lifetime; an unqualified request; a duplicate; `NewType` roles; a `TypeVar` factory |
| WU-M8 | wireup: the alias signature; an unqualified default beside a qualified provider |
| DI-M1 | dependency-injector: the scenario, default and `Annotated` forms |
| DI-M2 | dependency-injector: no role (`Singleton`, `Factory`); a lying annotation; `Provide[Database]`; no marker; `wire(warn_unresolved=True)` |
| DI-M3 | dependency-injector: `Provide[Database]` with the default `wire()`, `-W always` |
| DI-M4 | dependency-injector: DI-M1 without `Closing` |
| RD-M1 | rodi: the scenario, by type and by name |
| RD-M2 | rodi: no role by lifetime; a duplicate; by name; annotated names; a duplicate alias |
| RD-M5, M6 | rodi: two classes with one alias, under `PYTHONHASHSEED` 0–3; then under `python -O` |
| PQ-M1 | punq: the scenario; a function factory |
| PQ-M2 | punq: no role; a duplicate; `resolve_all`; a string key |

### A.2 Sources

| key | tag (commit) | files read |
|---|---|---|
| PT-S1 | pytest `9.2.0` (`ae1d0ff57b8bd4252da9ade7353523ac0ac73281`) | `src/_pytest/{compat,fixtures}.py`, `doc/en/how-to/fixtures.rst`, `doc/en/example/parametrize.rst` |
| PC-S1 | pytest-cases `3.10.1` (`d495c03c1beffd03dfa893acad70c6af6d5bf906`) | `docs/api_reference.md` |
| LF-S1 | pytest-lazy-fixtures `1.4.1` (`de9b74942e57450d6d2c29b0d2bdc715ef8e22f3`) | `README.md` |
| WD-S1 | Ward `release/0.68.0b0` (`f41c6164f3923660e3b03c3797913fab23496699`; links use the SHA, because the tag holds a slash) | `ward/{fixtures,testing,_fixtures,models}.py`, `docs/source/guide/fixtures.rst` |
| FA-S1 | FastAPI `0.162.0` (`7a0a3f0b286c6c865cf4398ea0e957a87b8bffa4`) | `fastapi/{param_functions,utils}.py`, `fastapi/dependencies/{utils,models}.py`, docs under `docs/en/docs/{tutorial/dependencies,advanced}/` |
| LS-S1 | Litestar `v2.25.0` (`ac33702eea7aa3c99d62711bce2d55bdada6b944`) | `litestar/di.py`, `litestar/handlers/base.py`, `litestar/_kwargs/kwargs_model.py`, `litestar/_signature/utils.py`, `docs/usage/dependency-injection.rst` |
| IJ-S1 | injector `0.24.0` (`6de2990f98d1b3076c73d2b92100638fdc0cfaa3`) | `injector/__init__.py`, `CHANGES`, `docs/{scopes,terminology}.rst` |
| DK-S1 | dishka `1.10.1` (`b2f6d3f1e2035de3388e91c28a3d2973b93da0d9`) | `src/dishka/{entities,graph_builder,dependency_source,integrations}/…`, `docs/advanced/{components,scopes,generics}.rst`, `docs/provider/provide.rst`, `docs/errors.rst` |
| LG-S1 | lagom `2.7.7` (`a924fda74c0a949206ff0da6ac86349f789b254b`) | `lagom/{container,decorators,exceptions,markers}.py`, `lagom/util/reflection.py`, `docs/{clean_up,cookbook,explicit_definitions}.md`, `CHANGELOG.md` |
| SV-S1 | svcs `26.2.0` (`9c740a713d1a2a3d65d6aadfcf3e35f4cf352290`) | `src/svcs/{_core,_autowire}.py`, `docs/{core-concepts,autowiring}.md`, `CHANGELOG.md` |
| WU-S1 | wireup `v2.13.0` (`bd4018c506263bca9f76292cf3cec078a25f0967`) | `wireup/{_annotations,_decorators}.py`, `wireup/ioc/{types,registry,registry_validation}.py`, `docs/pages/*.md` |
| DI-S1 | dependency-injector `4.49.1` (`d6749b861915b86ec76566005f09037dc5f4acf2`) | `src/dependency_injector/wiring.py`, `docs/wiring.rst`, `docs/providers/{singleton,factory,resource,context_local_resource}.rst` |
| RD-S1 | rodi `v2.1.0` (`58e86c60f5eb23bf03235827eb117a0226e5437f`) | `rodi/__init__.py`, `README.md` |
| PQ-S1 | punq `v0.9.0` (`437b27ba4ea8005935572be8088a45df18fddc8d`) | `punq/__init__.py`, `README.md` |
| TD-S1 | that-depends `4.2.1` (`bdeac0ad8d663488cd410f2eb78e98487008f1fd`) | `docs/introduction/{injection,type-based-injection}.md` |

## Appendix B. Versions

Each tool's source was read at the tag of the version run, the newest on PyPI on 2026-10-28.
Every run used CPython `3.13.13 (main, May  4 2026, 21:09:13) [Clang 22.1.3 ]`, through uv 0.13.1.

| tool | version run | with |
|---|---|---|
| pytest | 9.2.0 | pluggy 1.7.0 |
| pytest-cases | 3.10.1 | pytest 9.0.2 (it does not load on 9.2.0) |
| pytest-lazy-fixtures | 1.4.1 | pytest 9.2.0 |
| Ward | 0.68.0b0 | — |
| FastAPI | 0.162.0 | pydantic 2.14.1, starlette 1.10.0, httpx 0.28.1 |
| Litestar | 2.25.0 | msgspec 0.22.0, httpx 0.28.1 |
| injector | 0.24.0 | — |
| dishka | 1.10.1 | — |
| lagom | 2.7.7 | — |
| svcs | 26.2.0 | — |
| wireup | 2.13.0 | pytest 9.2.0 for WU-M2 |
| dependency-injector | 4.49.1 | — |
| rodi | 2.1.0 | — |
| punq | 0.9.0 | — |
| that-depends | not run | read at 4.2.1 |

## Appendix C. Liveness

From `gh api repos/<owner>/<repo>` and its releases, on 2026-10-28.

| project | archived | default branch, last commit | latest release | date |
|---|---|---|---|---|
| pytest-dev/pytest | no | `main`, 2026-10-27 | 9.2.0 | 2026-10-27 |
| smarie/python-pytest-cases | no | `main`, 2026-03-02 | 3.10.1 | 2026-03-02 |
| dev-petrov/pytest-lazy-fixtures | no | `master`, 2026-08-12 | 1.4.1 | 2026-08-12 |
| darrenburns/ward | **yes** | `master`, 2024-07-05 | release/0.68.0b0 | 2023-12-18 |
| fastapi/fastapi | no | `master`, 2026-10-26 | 0.162.0 | 2026-10-23 |
| litestar-org/litestar | no | `main`, 2026-10-28 | v2.25.0 | 2026-10-18 |
| alecthomas/injector | no | `master`, 2026-10-23 | 0.24.0 | 2026-01-09 |
| reagento/dishka | no | `develop`, 2026-10-15 | 1.10.1 | 2026-04-25 |
| meadsteve/lagom | no | `master`, 2026-05-24 | GitHub release 2.7.6; PyPI 2.7.7 from tag `2.7.7` | 2025-07-16 |
| hynek/svcs | no | `main`, 2026-10-19 | 26.2.0 | 2026-08-24 |
| maldoinc/wireup | no | `master`, 2026-10-26 | v2.13.0 | 2026-10-18 |
| ets-labs/python-dependency-injector | no | `master`, 2026-06-18 | 4.49.1 | 2026-06-18 |
| Neoteroi/rodi | no | `main`, 2026-03-08 | v2.1.0 | 2026-03-08 |
| bobthemighty/punq | no | `main`, 2026-10-26 | v0.9.0 | 2026-09-08 |
| modern-python/that-depends | no | `main`, 2026-10-24 | 4.2.1 | 2026-10-22 |

## Appendix D. Raw output

Excerpts, copied from the run files.

### D.1 The scenario's counts

Each of these printed `COUNTS {'db_built': 2, 'tx_opened': 4, 'tx_rolled_back': 4}` after two tests:
PT-M1, PC-M1, LF-M1, WD-M1, FA-M1, FA-M7, LS-M1, IJ-M1 (both forms), DK-M1 (both forms), LG-M1, SV-M1, SV-M2, WU-M1, WU-M2, WU-M8, DI-M1, RD-M1, PQ-M1.
DI-M1 and RD-M1 ran two more tests in another form, and printed `8, 8` for the transactions.

### D.2 No role given

```
PT-M3a  NO-ROLE a is b: True Tx(on=Database('only', id=1)) Tx(on=Database('only', id=1))
PT-M3b  E   SyntaxError: duplicate argument 'database' in function definition
WD-M2   NO-ROLE a is b: True … / USING a is b: True
FA-M2   /cached 200 {'a is b': True, …}   /nocache 200 {'a is b': False, …}
LS-M2   (a) REFUSED: ImproperlyConfiguredException 500: Provider for key b is already defined under the different key a. …
IJ-M2   bound with no scope: a is b: False   bound as singleton: a is b: True
DK-M2   provided at REQUEST, cache=True: a is b: True   cache=False: a is b: False
LG-M2   defined as a plain lambda: a is b: False   defined as Singleton: a is b: True
SV-M3   get(Database, Database): a is b = True   autowire(a: Database, b: Database): a is b = True
WU-M3   lifetime=singleton: a is b = True   scoped: True   transient: False
DI-M2   no role, Singleton: a is b = True   no role, Factory: a is b = False
RD-M2   singleton: a is b = True   scoped: True   transient: False
PQ-M2   no role, Database transient: a … b … a is b = True   transient cache=False: a is b = False
```

### D.3 A second provider for one key

```
IJ-M2   bind Database twice in one module: got: Database('second', id=7)
        two modules, First then Second: Database('second-module', id=8)
DK-M2   default validation, First then Second: Database('second', id=7)
        STRICT_VALIDATION refused at make_container: ImplicitOverrideDetectedError Detected multiple factories for (<class 'subject.Database'>, component='') while `override` flag is not set.
LG-M2   refused at definition: DuplicateDefinition DuplicateDefinition() - The type has already been defined somewhere else
SV-M3   registered twice, no error; get(Database) -> Database('second', id=3)
WU-M3   create_sync_container: wireup.errors.DuplicateServiceRegistrationError: Cannot register type <class 'subject.Database'> as it already exists.
RD-M2   duplicate add_singleton(Database) x2: OverridingServiceException: A service with key 'Database' is already registered and would be overridden by value <rodi.DynamicResolver object at 0x…>.
PQ-M2   duplicate: resolve -> Database('second', id=7)
        duplicate: resolve_all -> [Database('first', id=8), Database('second', id=9)]
PT-M5   OTHER MODULE got Database('from-conftest', id=1)
        OVERRIDE got Database('from-module-second', id=2)
LS-M2   (d) key db on controller and handler: {"db":"Database('from-handler-layer', id=4)"}
```

### D.4 The generic adapter

```
PT-M4   test_generic.py::test_factory PASSED / test_stamped PASSED
        INDIRECT got Tx(on=Database('source', id=1)) / INDIRECT got Tx(on=Database('target', id=2))
FA-M7   COUNTS {'db_built': 2, 'tx_opened': 4, 'tx_rolled_back': 4}
LS-M5   REFUSED: ImproperlyConfiguredException 500: Provider for key dst is already defined under the different key src. …
IJ-M4   refused: UnknownProvider couldn't determine provider for __main__.GTx[__main__.SourceDb] to None
DK-M4   --- NewType roles: refused: TypeError issubclass() arg 1 must be a class
        --- subclass roles: passed twice; COUNTS {'db_built': 2, 'tx_opened': 4, 'tx_rolled_back': 4}
DK-M5   --- NewType roles (TypeVar with no bound): passed twice; COUNTS {'db_built': 2, 'tx_opened': 4, 'tx_rolled_back': 4}
LG-M4   generic definition accepted
        call refused: UnresolvableType Unable to construct dependency of type __main__.GTx[__main__.SourceDb] …
WU-M3   TypeVar factory: wireup.errors.WireupError: Parameter 'db' of <class 'subject.Tx'> has an unknown dependency on ~D.
```

### D.5 Silent fills

```
PT-M2   test_wrong_annotation ANNOTATION Tx, GOT Database Database('source', id=1)  PASSED
WD-M2   ANNOTATION Database, GOT Tx   PASS
FA-M3   /annotated 200 {'type': 'Database', …}   /str 200 {'type': 'Database', …}
FA-M4b  /shortcut?name=chosen-by-client -> {'a': "Database('chosen-by-client', id=1)"}
LS-M4   (b) app1 session 1: {'db': "Database('db', id=2)"} / app2, same Provide object: {'db': "Database('db', id=2)"}
IJ-M5   unbound NewType role: Database('db', id=4) Database('db', id=5) a is b: False   (Database bound as a singleton 'bound-plain')
LG-M2   LG-M3b Annotated markers as roles: a = Database('db', id=8)  b = Database('db', id=9)  a is b: False
LG-M5   direct c[Annotated[Database,'source']]: Database('source', id=1)
        reflect(t): (Database)
SV-M6   plain Database with default -> None
DI-M2   Provide[Database] (a type, not a provider): a=Provide[<class 'subject.Database'>]
DI-M4   FAIL: target not clean at start: ['a', 'b']
        COUNTS after two default-form tests {'db_built': 2, 'tx_opened': 2, 'tx_rolled_back': 0}
RD-M2   annotated: source_db=Database('db', id=9) target_db=Database('db', id=9)
RD-M5   function exec: user_db -> UserDb / constructor: AssertionError Configured aliases cannot be ambiguous
RD-M6   function exec: user_db -> UserDb / constructor: built
```

### D.6 FastAPI exit order, FA-M6

```
function-scoped: enter
request-scoped: enter
handler: body
function-scoped: exit
middleware: response object returned
request-scoped: exit
client: got response
reverse nesting REFUSED: DependencyScopeError The dependency "outer_request" has a scope of "request", it cannot depend on dependencies with scope "function".
```

### D.7 pytest-cases on pytest 9.2.0, PC-M0

```
Error while loading plugin "pytest11".
…/pytest_cases/common_pytest.py:644: in <module>
    _idval = IdMaker([], [], None, None, None, None, None)._idval
E   TypeError: IdMaker.__init__() takes 7 positional arguments but 8 were given
EXIT=3
```
