# How a test names two inputs of one type: prototype

A throwaway branch for [#690](https://github.com/kalonji-tools/purba/issues/690). Never merge it.

## What ran

- `subject.py`: a `copy` from one database to another, and a planted bug, `copy_nothing`.
- `sketch.py`: a toy resolver that models the rules of #689: one value for each key in a test, and a refusal of two parameters with one key. It is a model, not a measurement. Its `strict` mode adds the two refusals that this prototype tests.
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

## Not varied

- A role declared in one module and used in another.
- A generic type, such as `Database[Row]`.
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
```
