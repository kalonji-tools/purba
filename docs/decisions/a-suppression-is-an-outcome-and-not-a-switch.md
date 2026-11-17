# A suppression is an outcome and not a switch

## Context and Problem Statement

A [strict check](../../CONTEXT.md#strict-check) can give a false positive.
With no way past it, the false positive blocks the [tester](../../CONTEXT.md#tester) until a fix ships.
[purba is always strict](purba-is-always-strict.md#decision-outcome) refuses every mode, flag and opt-out that turns strictness off.
A way past one verdict must therefore show in the report.

| fact | source |
|---|---|
| clippy guarantees no stability under `#[deny]`, because a user can add `#[allow()]` to their own crate | [RFC 2476](https://rust-lang.github.io/rfcs/2476-clippy-uno.html#stability-guarantees) |
| staticcheck's `//lint:ignore` requires a reason | [staticcheck configuration](https://staticcheck.dev/docs/configuration/) |
| staticcheck exits non-zero for an ignore directive that suppresses nothing | [staticcheck 2020.2](https://staticcheck.dev/changes/2020.2/) |
| Rust's `#[expect]` raises `unfulfilled_lint_expectations` when its lint does not fire | [the Rust Reference](https://doc.rust-lang.org/reference/attributes/diagnostics.html) |
| 50.8% of 1,873 suppressions in 10 Python projects affect no warning | [An Empirical Study of Suppressed Static Analysis Warnings](https://conf.researchr.org/details/fse-2025/fse-2025-research-papers/42/An-Empirical-Study-of-Suppressed-Static-Analysis-Warnings) |
| false positives are a minor share of suppressions in 1,425 Java projects | [Quieting the Static](https://arxiv.org/abs/2311.07482) |
| developers rewrite code slightly to suppress a false positive | [Tailoring Programs for Static Analysis via Program Transformation](https://2020.icse-conferences.org/details/icse-2020-papers/72/Tailoring-Programs-for-Static-Analysis-via-Program-Transformation) |
| coding agents exploit the checks they face | [ImpossibleBench](https://arxiv.org/abs/2510.20270), [EvilGenie](https://arxiv.org/abs/2511.21654) |

## Considered Options

- **No suppression.** The tester rewrites the test until the check stops, and no report shows that rewrite.
- **An opt-out that the report does not show.** It is the switch that [purba is always strict](purba-is-always-strict.md#considered-options) refuses, at a smaller scale.
- **A suppression at a class.** A reader of one method does not see a comment on its class.
- **One bracket after `project error` that holds every name of the line.** A refused line can give several reasons, and some have no name. The tester would then match each name to its words by their order.
- **`def-under-block`, `name-without-def` and `returned-value` in the practice tier.** Execution reads an expectation only in a `def` at the top of the module, and only in its last asserts. A suppressed one of the three still reaches no outcome.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/LANDSCAPE.md` `/CONTEXT.md`

**Area:** strictness

**A suppression is an outcome that the report shows, and not a switch.**

- **A [suppression](../../CONTEXT.md#suppression) names one strict check and gives a reason, in one comment.** A comment keeps [purba's public surface](purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome) empty.
- **A suppression acts at a test or a module, and not at a class.** A test's suppression is `# purba: suppress[<check>] <reason>`, on its own line directly above the `def` or above its first decorator. No blank line separates them. Several suppressions can stack, each with one name. A module's suppression is `# purba: suppress-module[<check>] <reason>`, above its first statement other than its docstring. purba refuses a path pattern in a file of configuration, because nobody reads it beside the test.
- **Each strict check has a name: its breach, in the glossary's words, in kebab case.** A refused line puts the name in brackets before the words of each check, such as `[computed-expected] line 7 computes obj, and an expected value is a literal`. A reason that says purba cannot call the test has no name and no bracket.
- **Each check sits in one tier.**

  | tier | checks |
  |---|---|
  | the [practice tier](../../CONTEXT.md#practice-tier): a suppression turns its violation into a warning, and the test runs | `foreign-decorator` `foreign-framework` `name-bound-again` `not-one-comparison` `computed-expected` `second-path` `assert-before-expectation` `swallowed-raise` `patched-name` `mock-without-spec` `iterator-read` `type-alone` |
  | never suppressed: a suppressed one would let an outcome stand on what purba did not check | `no-expectation` `readings-differ` `assert-in-reading` `hook-replaced` `no-item` |
  | never suppressed: purba reads no expectation of such a test, so a suppression cannot help | `def-under-block` `name-without-def` `returned-value` |
  | no name: purba cannot call the test, so no suppression reaches it | an `async` test, a generator, a parameter with no default, a test in a class |

- **A tester suppresses a check that execution sees as one that selection sees.** The case then gets the outcome that it would get without that check.
- **The report shows each suppressed verdict as a warning under the line of its case, and its reason on the line below.** The warning reads `⚠ suppressed [<check>] <words>`. It shows under a case that passed or failed, or that an error blocked. A suppressed verdict is never the outcome of its case.
- **One check of one test gives one suppressed verdict.** A check of selection or of the paths that one test breaks on two lines gives one warning, which joins both clauses. For an assert before the first expectation, the warning names the first such assert.
- **The count line counts each suppressed verdict after the errors, such as `; 5 suppressed`.** A suppressed verdict changes no exit code.
- **Each fault of a suppression is a project error, and a suppression with a fault does not count.** A fault is a `# purba:` comment that does not read as a suppression, or a suppression with no reason. A site that names one check twice has a fault. A name that no check has is a fault. A suppression of a check outside the practice tier is a fault. A `suppress[…]` above no test is a fault. A `suppress-module[…]` below the module's first statement is a fault. A fault directly above a test refuses that test. Any other fault is the module's, and blocks each test of the module.
- **purba counts a suppression as used only when it turns a violation of a selected test into a warning.** A verdict of `unknown` or `clean` passes through it unchanged.
- **A suppression that purba judged and that suppresses nothing is a project error, and fails the run.** If purba only warned about such a suppression, the suppression would stay. The Python study above finds suppressions that suppress nothing common.
- **purba judges a suppression of a check that selection sees at each test that purba can call, whether selection refuses the test or not.**
- **purba judges a suppression of a check that execution sees only in a case where execution read the paths of the act.** A case that raised leaves such a suppression unjudged. A case that a project error blocked before execution read its paths, and a test that did not run, leave it unjudged too.
- **A suppression above a test that purba cannot call stays unjudged.** A module's suppression that reaches such a test stays unjudged too.
- **purba judges a module's suppression only when it judged that suppression at each test that the suppression reaches.** A module's suppression of a check reaches no test when each test carries its own suppression of that check, so it suppresses nothing. When the module defines one test twice, with a suppression of one check above each `def`, the second suppression reaches no test.
- **An unused suppression changes no outcome and no warning of a case.** Its line is a project error of the module, and [a tester runs purba on one module, and reads a line for each case](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome) places it. The line reads `line <n> suppresses <check>, which <test> does not break`, or `line <n> suppresses <check>, which no test of the module breaks` for a module's suppression. It names no check in brackets, since an unused suppression is no strict check of a test.
- **An [integrator](../../CONTEXT.md#integrator) caps the suppressed verdicts with a configuration key and a flag.** The cap bounds how many violations a tester suppresses, whether a person or an agent occupies the tester. Where the integrator sets both, the lower value applies. The integrator can refuse every suppression with the cap.
- **The cap counts verdicts and not sites.** A count of sites would charge a module suppression once, however many verdicts it suppresses. A run over the cap fails. The report still lists every suppressed verdict.
- **The cap only tightens strictness, so it is not a switch.**

**Downside:**

- **A suppression hides a true violation as well as a false positive.** purba cannot judge a reason. The Java study above finds false positives a minor share of suppressions.
- **A module suppression covers tests that the tester did not read.**
- **A better check fails a run whose suppression it empties.** The tester must then remove the suppression.
- **purba reports an unused suppression above a test whose act raises only in a run in which the act returns.**
- **A run with a failed case exits 1, even when a suppression is unused.** Its exit code alone does not show an unused suppression.
- **The rule for configuration admits a value that states no fact about the project.** [A concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) calls the line between a fact and a switch thin. The cap moves that line.
- **Selection and execution each write the names of the checks that execution sees.** Selection checks the name of each suppression, and execution names each fault that it sees. Neither part reaches the other. The tests quote each name.
- **A tester cannot suppress a `def` under a block, a test name with no `def`, or a `return` of a value.** The tester rewrites such a test instead.
- **A suppression of a test in a class does not count.** No suppression reaches a test that purba cannot call.

## Confirmation

`mise run test:rust` checks in `crates/selection/tests/select.rs` each site, each tier and each fault of a suppression, and which suppressions selection judges and counts as used.
It checks in `crates/execution/tests/run.rs` each of the four checks that execution sees and a suppression reaches.
It checks there in which cases execution judges such a suppression.

`mise run test:wheel` checks in `tests/suppress.bats` the warning under a case that passed and under one that failed, and the count line.
It checks five faults of a suppression above a test, and a fault of the module.
It checks the line of each unused suppression after the last case, and a suppression that a raise left unjudged.
It checks that a module's suppression that one test used, and a suppression above a test that purba cannot call, print nothing.

No command checks the cap yet.
[A flag joins the public surface only with a change that purba needs to work as a command](purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome), so the cap waits for one.
