# How a test names two inputs of one type: prototype

A throwaway branch for [#690](https://github.com/kalonji-tools/purba/issues/690). Never merge it.

## What ran

- `subject.py`: a `copy` from one database to another, and a planted bug, `copy_nothing`.
- `sketch.py`: a toy resolver that models the rules of #689: one value for each key in a test, and a refusal of two parameters with one key. It is a model, not a measurement. Its `strict` mode adds the refusals that this prototype tests.
- `ways/`: one module for each way. Each runs the same test of a copy. `run.py` runs the test with `copy`, then with `copy_nothing`. A way works only when the planted bug fails the test.
- `stray.sh`: the same modules, with a builder that names plain `Database` instead of the source.
- `probe.py`: the runtime facts that each way depends on.
- `check.sh`: mypy 2.4.0 `--strict`, pyright 1.1.414 and ty 0.0.93 on every module.

Python 3.12.14 and 3.14.4, each as written and with `from __future__ import annotations`. The four runs printed the same lines (`runs/ways.txt`).

## Results

| way | what the test writes | planted bug | both databases through the database builder | a builder that names plain `Database` | that builder, strict |
|---|---|---|---|---|---|
| one value (#689 refuses) | `src: Database, dst: Database` | **passed** | yes | n/a | refused by #689 |
| by name | `src: Database, dst: Database` | failed | yes | **bug passed** | refused |
| `NewType` | `src: SourceDb, dst: TargetDb` | failed | yes | **bug passed** | refused |
| subclass | `src: SourceDb, dst: TargetDb` | failed | **no** | **bug passed** | refused |
| `Annotated` | `src: SourceDb, dst: TargetDb` | failed | yes | **bug passed** | refused |
| `type` alias | `src: SourceDb, dst: TargetDb` | failed | yes | **bug passed** | refused |
| factory | `new_db: Callable[[], Database]` | failed | yes | no builder can seed the source | n/a |
| tuple | `dbs: tuple[Database, Database]` | failed | yes | **bug passed** | refused |
| holder (refusal kept) | `bench: CopyBench` | failed | **no** | **bug passed** | **not refused** |
| holder whose builder names both | `bench: CopyBench` | **passed** | yes | n/a | refused |

All three type checkers passed every module, including each builder that names plain `Database`.

### What the runs show

1. Each way that gives the test two objects fails the planted bug. One value for both does not. This is the reason #689 refuses the pair.
2. **In every way, a builder that names plain `Database` gets a third database.** The test then seeds nothing, and the planted bug passes. No type checker sees it. The strict sketch refuses that builder in every way except the holder, because the holder's fields are hidden from purba.
3. The #689 refusal must also cover a builder. A builder that names two `Database` parameters gets one object twice, and the planted bug passes.
4. Subclass and holder lose the database builder's etiquette. Their builders construct each database directly, because a builder that returns a `Database` cannot return a `SourceDb`.
5. A factory test seeds the source itself, because no builder can reach a value made inside the test. A tuple makes the source a position, by convention.

### Runtime facts (`runs/probe.txt`, both Pythons)

| fact | result |
|---|---|
| two `NewType("SourceDb", Database)` | two keys |
| `isinstance(x, SourceDb)` for a `NewType` | `TypeError` |
| two `Annotated[Database, "source"]`, spelled apart | one key |
| `get_type_hints()` without `include_extras` | the marker is gone, both are `Database` |
| `Db = Database` | `Database` itself, one key |
| `type Db = Database` | a new key, unequal to `Database` and to another alias of it |
| a subclass of `bool` | `TypeError` |
| a subclass of `sqlite3.Connection` | only through `connect(factory=...)` |

## The generic adapter: `Tx[SourceDb]`

The roles sit on the resource, and one generic adapter serves every role. The sketch adds a resource, which lives for the session, and an adapter, which lives for one test and rolls back at its end.

```python
SourceDb = NewType("SourceDb", Database)   # a role of the resource
TargetDb = NewType("TargetDb", Database)

@resource
def database() -> Database: ...            # the only resource builder

@fixture
def tx[D: Database](db: D) -> Tx[D]:      # one adapter builder, for every role
    return Tx(db)

def test_copy(src: Tx[SourceDb], dst: Tx[TargetDb], seeded: Seeded) -> None: ...
```

`generic.sh` runs `test_copy` and then `test_starts_clean` in one session, with the real copy and with the planted bug, and with the adapter's rollback on and off. Three modules:

- `generic_derived`: one `Database` builder, and each role derives from it.
- `generic_per_role`: a builder for each role, and none for `Database`.
- `generic_typevar`: as `generic_derived`, with `D = TypeVar("D", bound=Database)`.

The three modules printed the same results on 3.12.14 and 3.14.4, as written and with `from __future__ import annotations`. All four runs printed the same lines.

| | result |
|---|---|
| `test_copy`, real copy | passed |
| `test_copy`, planted bug | failed |
| `src` and `dst` | two objects |
| each resource across the two tests | one object, built once (2 builds in the session) |
| `test_starts_clean`, rollback on | passed |
| `test_starts_clean`, rollback off | failed: the adapter's etiquette is what keeps the next test clean |

### Each variant (`runs/generic.txt`, 3.12.14)

| variant | without strict | strict |
|---|---|---|
| a builder names plain `Database`, roles derived | **bug passed** (a third database) | refused |
| a builder names plain `Database`, a builder for each role | refused: no builder for `Database` | refused |
| a builder names `Tx[Database]` | **bug passed** | refused |
| a builder names the resource `SourceDb` directly | bug failed: it gets the test's source | bug failed |
| a builder names `SourceDb` twice | bug failed: one shared resource, both names mean the source | refused |
| a test names `Tx[SourceDb]` twice | refused | refused |
| a test names the resource `SourceDb` | runs | refused: only a builder takes a resource |
| a test names `Tx[Seeded]` | refused: `Seeded` is not a `Database` | refused |

The three type checkers passed every module and every variant except `Tx[Seeded]`, which all three flagged as outside the bound (`runs/checkers_generic.txt`).

### Runtime facts (`runs/probe_generic.txt`, both Pythons)

| fact | result |
|---|---|
| `Tx[SourceDb] == Tx[TargetDb]` | `False` |
| `Tx[SourceDb]` spelled twice | one key |
| `get_type_hints(tx)["return"]` | `Tx[D]`, and `D`'s bound is `Database` |
| `isinstance(value, Tx[SourceDb])` | `TypeError` |
| a value built by `Tx(db)` records its role | no: purba trusts the builder's annotation |

### What the generic runs show

1. The role is written once, on the resource. One adapter builder serves every role, and a new role costs one line.
2. A builder for each role, and none for plain `Database`, refuses the stray builder with no extra rule. One `Database` builder with derived roles needs the strict rule.
3. The rollback is what keeps the shared resource clean between tests. purba cannot check it, as the #689 record says.

## Not varied

- A role declared in one module and used in another.
- A generic resource, such as `Database[Row]`.
- An adapter with two type parameters, or one generic over another.
- A catalyst. #689 holds that two catalysts of one type are harmless.
- What a type checker says about a mistake inside a builder, such as a `NewType` builder that returns a plain `Database`.
- purba's own resolver: none exists yet.

## Rerun

```sh
export PY=<python3.12> PY312=<python3.12> PY314=<python3.14>
./run.sh <scratch> > runs/ways.txt
./stray.sh <scratch> > runs/stray.txt
./check.sh <scratch> > runs/checkers.txt   # after stray.sh
$PY -I -B -c "import sys; sys.path.insert(0, '.'); exec(open('probe.py').read())" > runs/probe.txt
./generic.sh <scratch> > runs/generic.txt
./check_generic.sh <scratch> > runs/checkers_generic.txt   # after generic.sh
$PY -I -B -c "import sys; sys.path.insert(0, '.'); exec(open('probe_generic.py').read())" > runs/probe_generic.txt
```

## A `type` alias, to the type checkers

`alias_check.py` passes a `TargetDb`, and a plain `Database`, where a `SourceDb` is asked, with `type SourceDb = Database` and `type TargetDb = Database`. mypy, pyright and ty passed both (`runs/checkers_alias.txt`). To the type checkers, a `type` alias is the type itself, while at runtime it is a new key.
