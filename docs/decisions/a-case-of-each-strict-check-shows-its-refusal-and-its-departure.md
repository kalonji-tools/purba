# A case of each strict check shows its refusal and its departure

## Context and Problem Statement

[purba is always strict](purba-is-always-strict.md#decision-outcome) lets a tester turn a strict check into a warning only with a suppression and its reason.
[A suppression is an outcome and not a switch](a-suppression-is-an-outcome-and-not-a-switch.md#decision-outcome) gives each check a tier.
A tester can suppress a check of the practice tier, and cannot suppress a check of the other two tiers.
A case that a test sends to the installed purba shows a tester's screen for its check, and for the suppression of that check.

| fact | source |
|---|---|
| selection holds 12 checks in the practice tier, 5 that keep an outcome honest and 3 that read no expectation | [`suppression.rs` L11-L47](https://github.com/kalonji-tools/purba/blob/fb2dbee2/crates/selection/src/suppression.rs?plain=1#L11-L47) |
| on `main` at `fb2dbee2`, 13 files of `tests/` send a case of 19 of the 20 checks, and no file sends `foreign-decorator` | [the probe](https://github.com/kalonji-tools/purba/issues/1067#issuecomment-6961283743), with the command in item 7 of [its corrections](https://github.com/kalonji-tools/purba/issues/1067#issuecomment-6962491831) |
| the wheel refuses a case of each of the 20 checks, and prints the warning or the fault of its suppression | [the probe](https://github.com/kalonji-tools/purba/issues/1067#issuecomment-6961283743) |
| a suppression of `name-without-def` above the binding is above no test | [the probe](https://github.com/kalonji-tools/purba/issues/1067#issuecomment-6961283743) |

## Considered Options

- **The cases stay in the files that send them, with no gate.** A check can have no case, and nothing fails. `foreign-decorator` had none on `main` at `fb2dbee2`.

## Decision Outcome

**Reach:** `tests/**` `crates/selection/src/suppression.rs`

**Area:** strictness

**`tests/strictness.bats` sends a case of each strict check to the installed purba.**
A Rust test of selection fails when a check has no case in the file.

- **One test for each check, with two runs.** The first run sends the refused case. The second run sends the twin of that case. The twin carries a suppression and its reason. The test matches the exit code, the whole stdout and an empty stderr of each run.
- **The twin of a check of the practice tier passes.** purba prints the warning and the reason under the line of the test. The run exits 0.
- **For the twin of a check of the other two tiers, purba prints the fault of the suppression.** The run exits 4. The twin of `name-without-def` puts its suppression at the top of the module, because a suppression above the binding is above no test.
- **A refusal with no name, the read of a private name and a test that checks through a helper have one run each.** The refusals with no name are an `async` test, a generator, a parameter with no default, and a test in a class.
- **The Rust test searches `tests/strictness.bats` for `@test "[name]`, the start of a title.** It fails when no line holds that text for a check of the three tiers. Its message names each such check.
- **The text `@test "[name]` decides whether a check has a case, and a line of a screen does not.** The case of a test that checks through a helper also prints `[no-expectation]` and `[assert-before-expectation]`.
- **The files that already send a case of a check keep it.**
- **The warning of `-O` stays in `tests/optimize.bats`.** Its cases run purba with `PYTHONOPTIMIZE` at 1, 256 and 2.

**Downside:**

- **The Rust test does not see a lost run.** It searches only for the start of a title. The body of a test can lose its twin, or both runs, while its title stays.
- **A comment that holds `@test "[name]` makes the Rust test pass too.** The test searches the whole file.
- **No gate checks that the refusals with no name and the read of a private name keep a case.** Their lines carry no `[name]`.
- **A check can have a case in two files.** The author who changes the words of its line edits both files.

## Confirmation

`mise run test:rust` runs `each_strict_check_has_a_case_in_the_file_of_strictness` in `crates/selection/src/suppression.rs`.
`mise run test:wheel` and the `wheel` job of `ci.yml` run `tests/strictness.bats` on Linux and macOS.
A mutant takes the test of `no-expectation` out of the file.
The Rust test fails under the mutant.
The case of a test that checks through a helper still prints `[no-expectation]`.

```
sed -i '/^@test "\[no-expectation\]/,/^}/d' tests/strictness.bats
mise run test:rust
```

```
panicked at crates/selection/src/suppression.rs:477:9:
…/crates/selection/../../tests/strictness.bats holds no case of ["no-expectation"]
```

[The probe](https://github.com/kalonji-tools/purba/issues/1067#issuecomment-6961283743) measured the cases on `main` at `fb2dbee2`.
