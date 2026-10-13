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

- **No suppression.** Rejected. The tester rewrites the test until the check stops, and no report shows that rewrite.
- **An opt-out that the report does not show.** Rejected. It is the switch that [purba is always strict](purba-is-always-strict.md#considered-options) refuses, at a smaller scale.
- **A suppression that the report shows as a warning, by name and with a reason, under a cap that the integrator sets.** Chosen. The report shows every suppressed verdict, so a suppression is no switch. The cap bounds how many violations a tester suppresses, whether a person or an agent occupies the tester.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/CONTEXT.md`

**A suppression is an outcome that the report shows, and not a switch.**

- **A [suppression](../../CONTEXT.md#suppression) names one strict check and gives a reason.** purba refuses a suppression with no name or no reason.
- **A suppression acts at a test, a class or a module.** purba refuses a path pattern in a file of configuration, because nobody reads it beside the test.
- **The report shows each suppressed verdict as a warning on its own line, with its reason.** It counts each one. It never shows one as passed.
- **purba counts a suppression as used only when it turns a violation of a selected test into a warning.** A verdict of `unknown` or `clean` passes through it unchanged.
- **A suppression that suppresses nothing fails the run.** If purba only warned about such a suppression, the suppression would stay. The Python study above finds such suppressions common.
- **purba does not judge a suppression whose tests the run did not select.**
- **An [integrator](../../CONTEXT.md#integrator) caps the suppressed verdicts with a configuration key and a flag.** Where the integrator sets both, the lower value applies. The integrator can refuse every suppression with the cap.
- **The cap counts verdicts and not sites.** A count of sites would charge a module suppression once, however many verdicts it suppresses. A run over the cap fails. The report still lists every suppressed verdict.
- **The cap only tightens strictness, so it is not a switch.**

**Downside:**

- **A suppression hides a true violation as well as a false positive.** purba cannot judge a reason. The Java study above finds false positives a minor share of suppressions.
- **A module suppression covers tests that the tester did not read.**
- **A better check fails a run whose suppression it empties.** The tester must then remove the suppression.
- **The rule for configuration admits a value that states no fact about the project.** [A concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) calls the line between a fact and a switch thin. The cap moves that line.

## Confirmation

No command checks it, because purba holds no product code.
[The landscape](../../LANDSCAPE.md#the-order-of-design) ranks the design in depth of strictness.
That design names each check and the syntax of a suppression.
