# How a test names an input and its role: prototype

A throwaway branch for [#700](https://github.com/kalonji-tools/purba/issues/700). Never merge it.

## What ran

- `ways/subject.py`: a stock `Database`, a reagent `Tx[D]`, and `copy`.
- `ways/sketch.py`: a stub of a surface purba does not have. Nothing resolves. Each helper only records what a resolver would read.
- `ways/w*.py`: one module for each way that [#699](https://github.com/kalonji-tools/purba/blob/909f5de/research/test-signature.md#the-distinct-ways-found) found, in purba's terms. Each is the whole file a tester writes for the test of a copy. Two modules are not from #699: `w02` (the #690 syntax, with each parameter named after its role) and `w11` (a typed default).
- `run.sh` runs `probe.py`, which prints what `test_copy`'s signature hands a resolver at runtime. It ran on Python 3.12.15 and 3.14.4, as written and with `from __future__ import annotations`. The four runs printed the same lines (`runs/probe.txt`).
- `check.sh`: mypy 2.4.0 `--strict`, pyright 1.1.414 and ty 0.0.93, on Python 3.12 (`runs/checkers.txt`).
- `lie.sh`: each way whose line names a provider, with the source's provider swapped for `count`, which returns an `int`. A type checker that reports it can see a line that lies.

## Results

| way | the test's line | the line shows | mypy, pyright, ty | a provider of the wrong type | what it changes in the records |
|---|---|---|---|---|---|
| `w01` #690 | `src: Tx[SourceDb], dst: Tx[TargetDb]` | the role, as a type | pass | cannot be written: the type is the key | nothing |
| `w02` #690, named after the role | `source: Tx[SourceDb], target: Tx[TargetDb]` | the role, twice | pass | cannot be written | nothing |
| `w03` alias of the reagent | `src: SourceTx, dst: TargetTx`, with `SourceTx = Tx[SourceDb]` | the role, as a type | pass | cannot be written | nothing; at runtime purba sees `Tx[SourceDb]` |
| `w04` `type` alias of the reagent | as `w03`, with `type SourceTx = Tx[SourceDb]` | the role, as a type | pass | cannot be written | nothing; at runtime purba sees `SourceTx`, and #690 reads it as the type itself |
| `w05` `NewType` of the reagent | `src: SourceTx, dst: TargetTx` | the role, as a type | pass | cannot be written | each role needs its own reagent fixture; #690 has one generic one |
| `w06` parameter name | `source: Tx[Database], target: Tx[Database]` | the role, by name | pass | **silent** in all three | a test names its inputs by name, not by type (#689) |
| `w07` no annotation | `source_tx, target_tx` | the role, by name | **mypy refuses**; pyright, ty pass | **silent** in all three | selection already refuses a parameter with no type (#689) |
| `w08` provider in a decorator | `@using(src=source, dst=target)` over `src: Tx[Database], …` | the role and the provider, above the line | pass | **silent** in all three | an input is named by a decorator, not by type |
| `w09` provider as the default | `src: Tx[Database] = source` | the role and the provider | **all three refuse** | — | a parameter with a default becomes an input (#689 says it is not) |
| `w10` provider in a default typed `Any` | `src: Tx[Database] = use_any(source)` | the role and the provider | pass | **silent** in all three | as `w09` |
| `w11` provider in a typed default | `src: Tx[Database] = use(tx, source)` | the role, the stock fixture and the reagent fixture | pass | **all three report it** | as `w09` |
| `w12` provider in `Annotated` | `src: Annotated[Tx[Database], From(source)]` | the role and the provider | pass | **silent** in all three | an input is named by its provider, not by type |
| `w13` alias that hides the provider | `src: SourceTx`, with `SourceTx = Annotated[Tx[Database], From(source)]` | the role, as a name | pass | **silent** in all three | as `w12` |
| `w14` container attribute | `src: Tx[Database] = Provide[Stocks.source]` | the role and the provider | pass | **silent** in all three | as `w09` |
| `w15` marker string | `src: Annotated[Tx[Database], "source"]` | the role, as a string | pass | cannot be written: the type and string are the key | a role is a string, not a `NewType` (#690) |
| `w16` marker object | `src: Annotated[Tx[Database], Role("source")]` | the role, as a string | pass | cannot be written | as `w15` |
| `w17` tuple position | `pair: CopyPair`, then `src, dst = pair` | nothing; the role is a position in the body | pass | cannot be written | #690 rejected a type that holds both values |
| `w18` locator | `fx: Fixtures`, then `fx.get(Tx[SourceDb])` | nothing; the role is in the body | pass | cannot be written | a test names one locator, not its inputs |

ty also reported `sketch.py:20`, the stub's own `__purba_using__` attribute. That line is in the stub, not in a way.

### What the runs show

1. **Only one way that names a provider on the line lets a type checker see a lie: the typed default, `w11`.** Each of mypy, pyright and ty reported `use(tx, count)`. They passed the same swap in the decorator, in `Annotated`, in a default typed `Any`, and in a container attribute.
2. **The bare provider as a default, Ward's way, is refused by every type checker.** The annotation says `Tx`, and the default is a function.
3. **Three ways change no rule:** `w01`, `w02` and `w03`. `w04` changes none either, because #690 already reads a `type` alias as the type itself.
4. **A plain alias is invisible at runtime.** `SourceTx = Tx[SourceDb]` hands purba `Tx[SourceDb]`, so a message from purba names `Tx[SourceDb]`, not `SourceTx`. A `type` alias hands purba `SourceTx`.
5. **Each way that names the provider on the line overturns a line of #689:** a test names its inputs by type, and a parameter with a default is not an input.
6. **The shortest lines hide the role in the body** (`w17`, `w18`), or drop the type (`w07`).

### Length of the test's line, in characters

| | way |
|---|---|
| 37–45 | `w18` locator, `w17` tuple, `w07` no annotation |
| 53 | `w03`, `w04`, `w05`, `w13`: an alias |
| 61–67 | `w01` #690, `w02`, `w06`, `w08` |
| 79–97 | `w09`, `w10`, `w11`: a default |
| 103–115 | `w12`, `w14`, `w15`, `w16`: a marker on each parameter |

## What did not vary

- No resolver ran. Whether purba can build each way is not measured here; #690's prototype measured the generic reagent.
- No editor ran. That go-to-definition on `From(source)` reaches the fixture, and on `SourceDb` reaches the `NewType`, is inferred from how Python binds the name.
- The type checkers ran on 3.12 only.
- The lie of a typo in a role string (`w15`, `w16`) was not run.
- Readability itself is not measured. It is the person's call.
