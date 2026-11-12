# Which practices the sources hold for a good automated test, and how strong the evidence is

This file answers [#954](https://github.com/kalonji-tools/purba/issues/954).
It is never merged.
Every source was fetched on 2026-11-12, unless its row in the [source register](#source-register) says otherwise.

## Answer

The sources agree on little, and what they agree on concerns a test's verdict, not its shape.
Nearly every source wants a test that decides pass or fail by itself, gives the same result each time, and does not depend on other tests.
These three practices also have the strongest evidence.
Field studies count how often flaky and order-dependent tests occur, what causes them, and what they cost.
Even so, Meta's engineers write that every real test is flaky to some degree, so Meta measures flakiness and does not try to ban it.

Most other practices concern how a test is written: one condition per test, no logic in a test, a message on each assertion, repetition in place of abstraction, and the arrange-act-assert layout.
These practices rest on argument and on the authors' experience, and the sources contradict each other on several of them.
Go's own guidance runs a loop over a table of cases and tells a test to keep going after a failure.
Meszaros, Google and Microsoft argue against logic in a test and for one condition per test.
Google wants literal values in a test, while Microsoft and the ISTQB test-automation syllabus want constants or data files.
Studies of "test smells" find that tests with smells change more and are more defect-prone (Spadini 2018).
Other studies find that developers judge the detection rules too strict (Spadini 2020), and that the tools misclassify most smells in generated tests (Panichella 2020, 2022).

The deepest split is about mocks.
Google, Fowler and Meszaros prefer real collaborators and checks on state.
The inventors of mock objects argue for mocks as a design tool.
Microsoft's engineering playbook asks a unit test to remove all external dependencies.
On both sides the evidence is experience, plus one study of how developers actually mock (Spadini 2017).

Whether a suite is adequate is contested with real data.
NASA requires 100% MC/DC coverage for safety-critical software.
Inozemtseva and Holmes found only a low to moderate correlation between coverage and fault detection once suite size is controlled.
The number of assertions (Zhang and Mesbah) and killed mutants (Just 2014; Petrović 2021) track fault detection better.

Test-first is stated as a benefit by SWEBOK, ISTQB and Meszaros.
Where it was measured, writing the test first "had no important influence"; the fine-grained, steady steps did (Fucci 2017).

Some practices were dropped or softened.
These include capture-and-playback tests, JUnit's `@Test(expected=…)` and its `ExpectedException` rule, an assertion message on every assertion, strict one-assertion-per-test, and heavy use of mocking frameworks at Google.

From a test's code alone, a tool can decide only syntax with no false positive: a skip marker, a direct `sleep` call, an `if` in a test body, a count of known assertion calls, or a malformed test name.
No tool decides the practice behind that syntax.
Assertion helpers, soft assertions and expected-exception annotations cause false positives in "has an assertion" checkers (three PMD issues; an eslint-plugin-jest option exists to work around them).
Whether the assertions actually run needs execution, and even then DrTest reports false positives.
Finding order-dependent tests is NP-complete in a useful special case (Zhang 2014).

| practice | what it asks of a test | strongest evidence | contested? | decidable from code alone, no false positive? |
|---|---|---|---|---|
| [P1 self-checking](#p1-the-test-decides-its-own-verdict) | decide pass or fail with no human, and actually check something | empirical: assertions correlate with fault detection; rotten green tests exist | little; the edge is tests whose oracle is "no exception" | no: helpers and annotations give false positives |
| [P2 deterministic](#p2-same-code-same-result) | give the same result on the same code | empirical: causes, rates, costs measured | yes: Meta calls all tests flaky to a degree; reruns tolerated at low rates | no; a direct `sleep` call is decidable syntax only |
| [P3 independent](#p3-no-test-depends-on-another) | pass or fail regardless of other tests and their order | empirical: 96 dependent tests studied; order dependence a top-3 flaky cause | yes, at the margin: shared fixtures for cost; ordered integration tests | no: NP-complete special case; shuffling is dynamic and incomplete |
| [P4 one condition](#p4-one-behaviour-or-condition-per-test) | check one behaviour, so a failure points at one cause | mixed: smells linked to change-proneness; detectors misclassify | strongly | assertion count yes; "one condition" no |
| [P5 clear failure output](#p5-a-failure-explains-itself) | say what was expected, what happened, with which input | opinion and argument | yes: message per assertion vs rich assertion libraries vs no assertion libraries | presence of a message argument yes; quality no |
| [P6 no logic](#p6-no-logic-in-a-test) | be straight-line code a reader can check by eye | argument; practitioner survey | yes: Go table tests, parameterised and property tests loop | syntax yes; the practice no |
| [P7 repetition over abstraction](#p7-literal-and-visible-over-shared-and-dry) | keep values and setup visible in the test | argument | yes: direct contradictions between Google, Microsoft, ISTQB, tsDetect | no |
| [P8 phased layout](#p8-a-phased-layout) | set up, act, check (and tear down) in that order | opinion | mildly: alternating steps allowed by Google, not by Meszaros or Microsoft | no |
| [P9 public interface](#p9-test-through-the-public-interface) | test behaviour through the interface users use | argument; org experience | partly: per-class test layouts; mockist design | no |
| [P10 state over interaction](#p10-real-collaborators-and-state-over-mocks-and-interactions) | prefer real collaborators and checks on state over mocks | one empirical study of practice; org experience | the most contested | mock usage countable; "type you own" needs a project boundary |
| [P11 fast and small](#p11-fast-small-hermetic) | run in milliseconds, in one process, with no sleep, disk or network | empirical for sleep as a flaky cause; argument for speed | yes: speed trades against predictiveness | direct banned calls yes; speed no |
| [P12 adequacy](#p12-adequacy-coverage-assertions-mutants) | find faults; coverage, assertions and mutants as measures | empirical, and in conflict | strongly for coverage as a target | not a property of one test's code |
| [P13 test-first](#p13-write-the-test-first) | write the test before the code | empirical against the active ingredient | yes | no: needs history, not code |
| [P14 never leave a test switched off](#p14-do-not-leave-tests-switched-off) | no skipped, quarantined or lost tests left behind | opinion and argument | mildly: a skip with a reason is endorsed | skip markers yes |

"Contested" means at least one source in this survey argues against the practice or bounds it.
P12 and P13 concern a suite or a process, not one test.
I keep them because the sources argue about them in the same breath as the per-test practices.

## 1. The sources, and why I chose them

I chose sources in five kinds, three or more of each, as the issue asks.
Inside each kind I chose sources that are cited widely, that are free to fetch, and that differ from each other, so that disagreement could appear.
Full links and fetch commands are in the [source register](#source-register).

| kind | sources | why these |
|---|---|---|
| standard or body of knowledge | SWEBOK v4 (S1); ISTQB Foundation syllabus v4.0.1 (S2); ISTQB Test Automation Engineering syllabus v2.0 (S3); NASA NPR 7150.2D, SWE-219 (S4); ISO/IEC/IEEE 29119 (S5, **[C] not fetched**) | SWEBOK and ISTQB are the bodies of knowledge most courses and certifications use. NASA is a binding standard that takes a hard position on coverage. ISO 29119 is the testing standard, but its pages refused the fetch. |
| book on the design of tests | Meszaros, *xUnit Test Patterns* (B1); Winters, Manshreck and Wright, *Software Engineering at Google* (B2); Khorikov, *Unit Testing Principles, Practices, and Patterns* (B3, headings only); plus primary essays by book authors: Beck's *Test Desiderata* (B4), Freeman et al. *Mock Roles, not Objects* and Mackinnon et al. *Endo-Testing* (B5), Fowler (B6), Feathers (B7) | Meszaros is the catalogue the test-smell research cites. The Google book is free online and states one large organisation's rules. Khorikov is recent and widely cited, but paywalled. The essays are the authors' own statements of the ideas in their books (TDD, mock objects, legacy code), which I could not fetch as books. |
| peer-reviewed empirical study | Luo 2014 (E1); Zhang 2014 (E2); Inozemtseva 2014 (E3); Zhang and Mesbah 2015 (E4); Just 2014 (E5); Ivanković 2019 (E6); Petrović 2021 (E7); Fucci 2017 (E8); Kochhar 2019 (E9); Spadini 2018 (E10); Panichella 2022 (E11); Panichella 2020 (E12); Spadini 2020 (E13); Delplanque 2019 (E14); Parry 2022 (E15); Spadini 2017 (E16); Peruma 2020 (E17); Claessen and Hughes 2000 (E18) | One or more studies for each practice that a study can measure: flakiness, independence, coverage, assertions, mutants, test-first, smells, mocks, and practitioners' views. Two (E11, E12) test the detectors themselves, which item 5 needs. |
| organisation that tests at large scale | Google (B2, O1, O2, E6, E7); Microsoft (O3, O4); Meta (O5); Spotify (O6); NASA (S4) | Google publishes the most. Microsoft, Meta and Spotify each publish a position that differs from Google's on one point. |
| test runner or test linter documentation, three or more languages | Java: JUnit 5/6 user guide and JUnit 4 wiki (T1), PMD (T2), tsDetect (T6); C++: GoogleTest (T3); Go: `go test`, Go wiki, `go vet` (T4); JavaScript: eslint-plugin-jest (T5) | Four languages. Each runner encodes a position: fresh fixture per test, shuffling, keep-going assertions. Each linter encodes a decidable rule, so its documentation and issue tracker show the false positives. |

I left Python and Rust tools out on purpose: [#955](https://github.com/kalonji-tools/purba/issues/955) and [#956](https://github.com/kalonji-tools/purba/issues/956) cover them.

### What the bodies of knowledge cover, and what they do not

SWEBOK's testing chapter and the ISTQB Foundation syllabus are mostly about *which inputs to test* (equivalence partitions, boundary values, coverage criteria) and about the test process.
Neither says much about how a single automated test should be written.
The words "flaky" and "test smell" do not appear in SWEBOK v4 at all ([A], `grep -iE 'flak|test smell'` on the extracted text: no match).
The ISTQB automation syllabus (S3) does speak to automated test code, but by importing general clean-code rules and design patterns.
So the per-test practices below come mostly from books, organisations, tools and empirical studies, not from the standards.
This is a finding in itself: the standards do not settle the questions this issue asks.

## 2–5. The practices

Each practice below names what it asks (item 2), who holds it and on what evidence (item 2), who contests it (item 3), what it costs (item 4), and whether a tool can decide it (item 5).
Evidence is marked as **empirical** (measured on real projects), **argument** (reasoned, with examples but no measurement), or **opinion** (stated as a preference or from experience).

### Group A. The verdict: what a pass or a fail means

This grouping follows Meszaros's goals "Fully Automated Test", "Self-Checking Test" and "Repeatable Test" (B1), Beck's "Automated", "Deterministic" and "Isolated" (B4), and Microsoft's "Self-Checking", "Repeatable" and "Isolated" (O3).

#### P1. The test decides its own verdict

**Asks.** The test checks an expected outcome itself, with no human reading the output, and the check actually runs.

**Holds it.**
- SWEBOK: "a test is meaningful only if it is possible to decide its observed outcome." (S1, §1.2.7 [A])
- Meszaros: "A Self-Checking Test is one that has encoded within it everything that it needs to verify that the expected outcome is correct." (B1 [A])
- Microsoft: "Self-Checking: The test should automatically detect if it passed or failed without any human interaction." (O3 [A])
- Beck: "Automated — tests should run without human intervention." (B4 [A])
- ISTQB automation syllabus: "Verify if all the assertions are in place. Missing assertions may result in inconclusive test results." (S3, §7.1.3 [A])
- Practitioners: "Test assertions can help detect subtle errors that might otherwise go undetected" scored 4.51 of 5, among the highest of 29 statements (E9 [A]).

**Evidence.** Empirical.
Zhang and Mesbah built 6,700 suites from 24,000 assertions in five Java projects and found that "the number of assertions in a test suite strongly correlates with its effectiveness" (E4 [A]).
Delplanque et al. found 294 "rotten green" tests among 19,905 Pharo tests: tests that pass but "contain assertions that are never executed" (E14 [A]).
"Some rotten tests have been “sleeping” in Pharo for at least 5 years." (E14 [A])

**Contested.** Hardly at all.
The edge cases are tests whose oracle is "no exception was raised", and tests whose check sits in a helper or an annotation.
Fowler calls coverage-driven tests with no assertions the absurd end of coverage targets ("AssertionFreeTesting", B6 [A]).

**Cost.** Writing an oracle can be hard. SWEBOK: "The automation of oracles can be difficult and expensive." (S1 [A])

**Mechanics.** Not decidable from code alone without false positives.
- PMD `UnitTestShouldIncludeAssert` (Java) has closed issues titled as false positives: soft assertions in a JUnit 4 rule ([pmd#6681](https://github.com/pmd/pmd/issues/6681)), soft assertions on a parent class ([pmd#6431](https://github.com/pmd/pmd/issues/6431)), and TestNG's `expectedException` ([pmd#6188](https://github.com/pmd/pmd/issues/6188)) (T2 [A], `gh search issues`).
- eslint-plugin-jest `expect-expect` needs an `assertFunctionNames` option to name the helper functions that assert (T5 [A]). Its issue [#1475](https://github.com/jest-community/eslint-plugin-jest/issues/1475) reports that `additionalTestBlockFunctions` does not recognise `test.each`-like functions (T5 [A], title only).
- tsDetect's "Unknown Test" smell detects "A test method that does not contain a single assertion statement and @Test(expected" (T6 [A]); the annotation is the escape hatch.
- Whether the assertion *executes* is a runtime fact. DrTest combines static and dynamic analysis and "reports no false negatives, yet it still reports some false positives due to conditional use or multiple test contexts" (E14 [A]).

#### P2. Same code, same result

**Asks.** The test gives the same result each time it runs on unchanged code.

**Holds it.**
- Meszaros: "A Repeatable Test is one that can be run many times in a row with exactly the same results without any human intervention between runs." (B1 [A])
- Beck: "Deterministic — if nothing changes, the test result shouldn’t change." (B4 [A])
- Microsoft: "Repeatable: Running a unit test should be consistent with its results." (O3 [A]); the Microsoft playbook asks for "100% reliable so failures indicate a bug in the code" (O4 [A]).
- Google: "the most important qualities we want from our test suite are speed and determinism" (B2 ch. 11 [A]).
- Practitioners: 4.07 of 5 for "a good test case should be designed such that its results are deterministic" (E9 [A]).

**Evidence.** Empirical, the strongest in the survey.
- Luo et al. studied 201 commits that likely fix flaky tests in 51 projects (E1 [A]). "The top three categories of flaky tests are Async Wait, Concurrency, and Test Order Dependency." "Most flaky tests (78%) are flaky the first time they are written." (E1 [A])
- Google, 2016: "about 1.5% of all test runs reporting a "flaky" result", and "about 84% of the transitions we observe from pass to fail involve a flaky test" (O1 [A]).
- Google, 2020: "as you approach 1% flakiness, the tests begin to lose value. At Google, our flaky rate hovers around 0.15%" (B2 ch. 11 [A]). The 2016 and 2020 figures measure different things at different times; I do not reconcile them.
- Parry et al. survey 76 papers on flaky tests and cite a developer survey in which "59% claimed to deal with flaky tests on a monthly, weekly, or daily basis" (E15 [A], abstract).

**Contested.**
- Meta: "all real-world tests are flaky to some extent, even if they are implemented following best engineering principles. Thus, the right question to ask is not whether a particular test is flaky, but how flaky it is." (O5 [A]) Meta found no end-to-end test without some flakiness (O5 [A]).
- Google tolerates reruns at low rates: "At low levels of flakiness, this trade-off makes sense." (B2 ch. 11 [A])
- Fowler advises quarantine, with a limit: "Place any non-deterministic test in a quarantined area. (But fix quarantined tests quickly.)" (B6 [A])
- Random inputs. Meszaros says "Nondeterministic Tests are caused by using different values each time a test is run" (B1 [A]). Property-based testing does exactly that on purpose: QuickCheck properties "can be automatically tested on random input" (E18 [A]). The two sources disagree on whether random input is a defect or a technique. (Reproducing a failure from a seed is how property tools usually reconcile the two; I did not fetch a source for that, so it is [B].)

**Cost.** Google restricts small tests to one process, often one thread, with no sleep and no I/O (B2 ch. 11 [A]).
Larger tests pay more: "If a large test is nonhermetic, it is almost impossible to guarantee determinism" (B2 ch. 14 [A]).
Following P2 strictly can push a suite away from realistic tests; Beck names the tension directly: "Making tests more predictive of production behavior makes them slower." (B4 [A])

**Mechanics.** Not decidable from code alone.
Flakiness is a property of runs, not of text; Google, Meta and the flaky-test literature all detect it by rerunning or by modelling run history (O1, O5, E15 [A]).
A proxy is decidable as syntax: tsDetect's "Sleepy Test" is "A test method that invokes the Thread.sleep() method" (T6 [A]).
That proxy misses a sleep inside a helper, and it flags a test whose purpose is to exercise real time.
Luo found that only about a third (34%) of Async Wait flaky tests "use a simple method call with time delays" (E1 [A]), so the syntactic proxy covers a minority of even that one cause.

#### P3. No test depends on another

**Asks.** A test passes or fails the same whether it runs alone, first, last, or alongside others.

**Holds it.**
- Meszaros: "An Independent Test can be run by itself." (B1 [A])
- Beck: "Isolated — tests should return the same results regardless of the order in which they are run." (B4 [A])
- Google: "All tests should strive to be hermetic … Tests should assume as little as possible about the outside environment, such as the order in which the tests are run." (B2 ch. 11 [A])
- JUnit creates a new test-class instance per method "to allow individual test methods to be executed in isolation" (T1 [A]); its default order is "deterministic but intentionally nonobvious" (T1 [A]).
- GoogleTest "creates a new test fixture object for each test in order to make tests independent and easier to debug", and offers `--gtest_shuffle`, which "helps to reveal bad dependencies between tests" (T3 [A]).
- `go test -shuffle` will "Randomize the execution order of tests and benchmarks. It is off by default." (T4 [A])

**Evidence.** Empirical.
Zhang et al. studied "96 real-world dependent tests from 5 issue tracking systems" and found that dependence "can cause non-trivial consequences, such as masking program faults and leading to spurious bug reports" (E2 [A]).
94 of the 96 caused false alarms (E2 [A]).
Luo ranks test order dependency among the top three flaky causes, and "Most Test Order Dependency flaky tests (74%) are fixed by cleaning the shared state between test runs." (E1 [A])

**Contested.** At the margin, by cost.
- Practitioners agree less than with most statements: 3.95 of 5, with 48 neutral, 19 disagreeing and 6 strongly disagreeing; those who disagree "often highly value reuse over simplicity" (E9 [A]).
- JUnit documents ordered methods: "Although true unit tests typically should not rely on the order in which they are executed, there are times when it is necessary to enforce a specific test method execution order" (T1 [A]).
- GoogleTest documents sharing an expensive resource across a suite: "If the tests don’t change the resource, there’s no harm in their sharing a single resource copy." (T3 [A])
- Meszaros keeps Shared Fixture as a pattern. On his Slow Tests page he warns that reaching for one "almost always results in other problems including Erratic Tests" (B1 [A]).

**Cost.** A fresh fixture per test costs setup time and setup code. That cost is why shared fixtures exist (B1 Slow Tests [A]).

**Mechanics.** Not decidable from code alone.
Zhang et al. "prove that a useful special case is NP-complete", and their four detection algorithms are "sound but incomplete" (E2 [A]).
Shuffling (T3, T4) is dynamic and samples orders; it finds dependence only when a sampled order exposes it.

### Group B. The diagnosis: what a failure tells the reader

#### P4. One behaviour or condition per test

**Asks.** A test checks one behaviour, so that a failure points at one cause.

**Holds it.**
- Meszaros, "Verify One Condition per Test": combining conditions "is not recommended because when one assertion fails, the rest of the test is not executed." (B1 [A])
- Google: "Each test should cover only a single behavior, and the vast majority of unit tests require only one "when" and one "then" block." (B2 ch. 12 [A])
- Microsoft: "try to include only one Act task per test" (O3 [A]); the playbook: "Unit tests should be short and test only one thing." (O4 [A])
- Beck: "Specific — if a test fails, the cause of the failure should be obvious." (B4 [A])
- PMD `UnitTestContainsTooManyAsserts` defaults to `maximumAsserts` = 1 (T2 [A]); eslint-plugin-jest `max-expects` defaults to 5 (T5 [A]).
- Practitioners: 3.93 of 5 for "one test case should be testing one aspect of a requirement" (E9 [A]).

**Evidence.** Mixed, and partly against the strict form.
Spadini et al. (221 releases, ten systems, over a million test cases) found "tests with smells are more change- and defect-prone", and that "‘Indirect Testing’, ‘Eager Test’, and ‘Assertion Roulette’ are the most significant smells for change-proneness" (E10 [A]).
Panichella et al. found that Assertion Roulette and Eager Test "rarely indicated issues, either because of advances in testing frameworks (esp. “Assertion Roulette”) or because they encode a matter of preference (e.g., “Eager Test”)" (E11 [A]).

**Contested.** Strongly, and on its premise.
- Meszaros's reason is that a failed assertion stops the test. Several runners removed that premise. GoogleTest: "Usually EXPECT_* are preferred, as they allow more than one failure to be reported in a test." (T3 [A]) Go: "Even after your test cases encounter a failure, they should keep going for as long as possible" (T4 [A]). JUnit's `assertAll` runs a group of assertions and reports "all failures … together" (T1 [A]).
- Meszaros himself does not insist on one assertion: "I wouldn't be too dogmatic about insisting on a single assertion." (B1 [A])
- Google allows "alternating sequences of when/then blocks" for a multistep process (B2 ch. 12 [A]); Meszaros rules that out ("We do not have a series of such alternating calls", B1 [A]).
- Practitioners who disagree argue from cost: "in cases where tests take longer to run, testing multiple things in one test case may be a more efficient alternative" (E9 [A]).

**Cost.** More test methods and more repeated setup (B1 [A]). Panichella et al. suggest developers may not split tests "for fear of introducing clones" (E11 [A]).

**Mechanics.** The assertion count is decidable as syntax for known assertion names; PMD and eslint-plugin-jest count it (T2, T5 [A]).
The count is not the practice: one behaviour can need several assertions, and one assertion can check a compound object.
For the related smell, Panichella et al. found "test methods with only a single assertion are common among the false positives" of the Assertion Roulette detector used by earlier work, on tests generated by EvoSuite; its F-measure was 0.36 (E11 [A]).

#### P5. A failure explains itself

**Asks.** A failing test says what was expected, what happened, and with which input; its name says which behaviour broke.

**Holds it.**
- Google: "A good failure message contains much the same information as the test’s name: it should clearly express the desired outcome, the actual outcome, and any relevant parameters." (B2 ch. 12 [A]) "A test’s name should summarize the behavior it is testing." (B2 ch. 12 [A])
- Go wiki: "Got before Want", "Identify the Function", "Identify the Input" are section headings (T4 [A]).
- Meszaros, Assertion Message: people with several assertions per test "should strongly consider including a message that at least distinguishes which assertion failed." (B1 [A])
- PMD `UnitTestAssertionsShouldIncludeMessage` (since PMD 1.04): "Unit assertions should include an informative message" (T2 [A]).

**Evidence.** Opinion and argument. I found no measurement of failure-message quality against diagnosis time.

**Contested.** Yes, three ways.
- Messages on every assertion versus assertion libraries that print the subject: Google's Truth example yields "<[red, green, blue]> should have contained <orange>" with no hand-written message (B2 ch. 12 [A]).
- Go discourages assertion libraries entirely: "Avoid the use of ‘assert' libraries to help your tests." (T4 [A]) Google's Java chapter recommends one (Truth). These are the same organisation's guidance in two languages.
- Panichella et al.: in JUnit 4 "an assert failure is explained by the JUnit runtime, and the very notion of the lack of documentation as a “smell” is thus rather doubtful." (E11 [A])
- Meszaros records "two schools of thought on this subject" (B1 [A]).

**Cost.** A hand-written message must be kept true as the test changes; a library adds a dependency and, in Go's view, "a whole new sub-language" (T4 [A]).

**Mechanics.** Whether an assertion call has a message argument is decidable (PMD does it, T2 [A]). Whether the message or the name is informative is not.

### Group C. Reading: can a reader trust the test by eye?

#### P6. No logic in a test

**Asks.** A test is straight-line code with no loops, branches or computed expected values.

**Holds it.**
- Google: "Clear tests are trivially correct upon inspection", and "in test code, stick to straight-line code over clever logic" (B2 ch. 12 [A]).
- Microsoft: "avoid manual string concatenation, logical conditions, such as if, while, for, and switch" (O3 [A]).
- Meszaros, Conditional Test Logic: "Test Methods (page X) must be simple enough to not need tests." "Any control structures within a Test Method should be viewed with extreme suspicion!" (B1 [A])
- eslint-plugin-jest `no-conditional-in-test` reports "any use of a conditional statement such as `if`, `switch`, and ternary expressions"; `no-conditional-expect` (in the recommended set) because conditional `expect` calls can make tests "end up passing but not actually test anything" (T5 [A]).
- Practitioners: 4.04 for "Increased complexity in a test case can lead to bugs in the test code itself", 4.20 for "Test cases should be simpler than the code being tested" (E9 [A]).

**Evidence.** Argument, with one worked example: Google shows a string concatenation that hides a double slash (B2 ch. 12 [A]). Practitioner opinion (E9). I found no study that measures defects in tests with and without logic.

**Contested.** Yes, by whole testing styles.
- Go's table-driven tests loop over cases: "the actual test simply iterates through all table entries" (T4 [A]). Go's wiki then bounds it: when "every entry in a table has to be subjected to multiple kinds of conditional logic", write separate functions (T4 [A]).
- Meszaros's own Parameterized Test pattern factors the loop out (B1 [A]).
- Property-based tests compute expected relations by design (E18 [A]).
So the sources split on loops over data, and largely agree against branches that pick the check.

**Cost.** Repetition (see P7).

**Mechanics.** The syntax is decidable: `no-conditional-in-test` reports every `if`, `switch` and ternary (T5 [A]); tsDetect has a "Conditional Test Logic" smell (T6 [A]).
The practice is not: a loop over a table is endorsed by Go and reported by these rules.

#### P7. Literal and visible over shared and DRY

**Asks.** Keep the values and the setup that matter inside the test, even at the cost of repetition.

**Holds it.**
- Google, "DAMP, Not DRY": "A little bit of duplication is OK in tests so long as that duplication makes the test simpler and clearer." (B2 ch. 12 [A]) On shared constants: "it can be difficult to understand why a particular value was chosen for a test" (B2 ch. 12 [A]).
- Microsoft prefers helper methods to shared Setup and Teardown, so that "Each test shows the setup it requires." (O3 [A])

**Evidence.** Argument only.

**Contested.** Directly, on literal values.
- Microsoft, same page: "Rather than using magic strings, assign any hard-coded values to constants." (O3 [A])
- ISTQB automation syllabus: "Hardcoding reduces development time, but it is not recommended", to be avoided "by using data-driven testing" (S3 [A]).
- tsDetect's "Magic Number Test" smell: numeric literals in assertions "should be replaced with constants or variables" (T6 [A]).
- Meszaros lists Test Code Duplication as a smell (B1 [A]); Go's wiki says copy and paste is a cue to refactor into a table or a helper (T4 [A]).

Google and Microsoft agree that setup should be visible, and disagree on whether a literal in a test is a defect.

**Cost.** Repetition, and edits in many places when a shared fact changes. Abstraction costs the reader a jump (B2 ch. 12 [A]).

**Mechanics.** No. A literal is detectable, but whether it should be a constant is the disputed question itself.

#### P8. A phased layout

**Asks.** Set up, exercise, verify, and (where needed) tear down, in that order.

**Holds it.** Meszaros's Four-Phase Test: "fixture setup, exercise SUT, result verification and fixture teardown" (B1 [A]). Microsoft's Arrange, Act, Assert (O3, O4 [A]). Google's given/when/then (B2 ch. 12 [A]). ISTQB names Given/When/Then as the BDD format (S2 [A]).

**Evidence.** Opinion.

**Contested.** Mildly: Google permits alternating when/then blocks (B2 ch. 12 [A]); Meszaros and Microsoft do not (B1, O3 [A]).

**Mechanics.** No. The phases are usually marked, if at all, by comments or blank lines.

### Group D. Coupling: what makes a test break

Beck states this group as a pair of properties: "Behavioral — tests should be sensitive to changes in the behavior of the code under test" and "Structure-insensitive — tests should not change their result if the structure of the code changes" (B4 [A]).
Google states it as a goal: "the ideal test is unchanging: after it’s written, it never needs to change unless the requirements of the system under test change." (B2 ch. 12 [A])

#### P9. Test through the public interface

**Asks.** Call the code the way its users do, and test behaviours, not methods.

**Holds it.**
- Google: "By far the most important way to ensure this is to write tests that invoke the system being tested in the same way its users would" (B2 ch. 12 [A]); and "Test Behaviors, Not Methods" (B2 ch. 12 [A]).
- Microsoft: "In most cases, you don't need to test a private method in your code." (O3 [A])
- Meszaros, "Use the Front Door First": "we test an object through it's public interface and use State Verification" (B1 [A]).
- Go: do not compare error strings, which can turn a test into "a change detector" (T4 [A]).

**Evidence.** Argument and organisational experience.

**Contested.** Partly. Meszaros's Testcase Class per Class (one test class per production class) is "a good starting point", to be split later (B1 [A]); Google warns that one test per production method "can be convenient at first, but over time it leads to problems" (B2 ch. 12 [A]).

**Cost.** Some internal behaviour is hard to reach from outside; Microsoft's page covers the case with "seams" for static references (O3 [A], heading only).

**Mechanics.** No. Calls to private members through reflection are detectable in some languages, but "behaviour, not method" is not a property of syntax.

#### P10. Real collaborators and state over mocks and interactions

**Asks.** Prefer real implementations or fakes, and check the resulting state, over mocks that check calls.

**Holds it (the "classicist" side).**
- Google: "Interaction tests tend to be more brittle than state tests" (B2 ch. 12 [A]); "Prefer Realism Over Isolation" (B2 ch. 13 [A]).
- Google's history: mocking frameworks "seemed like a hammer fit for every nail"; such tests "required constant effort to maintain while rarely finding bugs. The pendulum at Google has now begun swinging in the other direction" (B2 ch. 13 [A]).
- Fowler: "I don't see any compelling benefits for mockist TDD, and am concerned about the consequences of coupling tests to implementation." He adds that he has not tried mockist TDD "on anything more than toys" (B6 [A]).
- Meszaros: "Overuse of Behavior Verification (page X) and Mock Objects (page X) can result in Overspecified Software" (B1 [A]).

**Contested (the "mockist" side, and the isolation side).**
- The inventors of mock objects: "Using Mock Objects for unit testing improves both domain code and test suites." (B5, Endo-Testing [A]) Mock objects are "less interesting as a technique for isolating tests from third-party libraries than is widely thought"; they are a design technique (B5, Mock Roles [A]).
- They reply to the brittleness charge: "Our experience is that Mock Object-based test failures are more focussed and more self-explanatory" (B5 [A]).
- Microsoft's playbook asks for isolation: "Isolated - removing all external dependencies ensures reliability and speed" (O4 [A]).
- Feathers: "A test is not a unit test if: It talks to the database; It communicates across the network; It touches the file system …", while adding "Tests that do these things aren't bad." (B7 [A])
- Google itself names the dispute: "Even though some people in the software industry practice mockist testing (including the creators of the first mocking frameworks), at Google, we have found that this style of testing is difficult to scale." (B2 ch. 13 [A])

**A narrower rule both sides share.** "Only Mock Types You Own" (B5 [A]): do not mock types "defined by the runtime or external libraries"; wrap them. "Specify as little as possible in a test" (B5 [A]).

**Evidence.** One empirical study of practice: Spadini et al. analysed over 2,000 test dependencies in four systems and surveyed over 100 developers (E16 [A], summary via the TU Delft portal). Developers "frequently mock dependencies that make testing difficult", prefer not to mock domain classes, and report that "mocking increases the coupling between the test and the production code" (E16 [A], short quotes). The rest is organisational experience (Google) and the authors' experience (Fowler, Freeman et al.). No study I fetched compares fault detection or maintenance cost of the two styles.

**Cost.** Real collaborators make tests slower and failures less local (B5 §5.5 [A]); mocks make tests duplicate the implementation (B5 §5.4 [A]) and break on refactoring (B2 ch. 13 [A]).

**Mechanics.** The number of mocks and interaction checks is countable. "Type you own" needs a project boundary as input, which a tool can be given; with it, a check is mechanical, but I found no tool that implements it ([C]). Whether a mock is appropriate is not decidable.

### Group E. Cost to run

#### P11. Fast, small, hermetic

**Asks.** Run in milliseconds, in one process, with no sleep, no network and no disk.

**Holds it.**
- Google's sizes: small tests "must run in a single process" and "aren’t allowed to sleep, perform I/O operations, or make any other blocking calls" (B2 ch. 11 [A]).
- Microsoft: "Unit tests should take little time to run. Milliseconds." (O3 [A])
- Beck: "Fast — tests should run quickly." (B4 [A])
- Fowler: "I strongly urge you to never use bare sleeps like this." (B6 [A]) Google: "actively poll for a state transition" (B2 ch. 11 [A]).
- Meszaros, Slow Tests: slow suites stop developers running them (B1 [A]).
- Feathers separates fast unit tests from tests that touch the database, network or file system (B7 [A]).

**Evidence.** Empirical for sleep as a flaky cause (E1, see P2). Argument for speed itself.

**Contested.**
- Beck: "Making tests more predictive of production behavior makes them slower." (B4 [A])
- Practitioners: 3.59 for "Large test cases may be needed to detect difficult bugs" (E9 [A]).
- At suite level, the pyramid shape that follows from P11 is contested: ISTQB and Google endorse it (S2 [A]; O2 "70/20/10" [A]); Spotify says for microservices "it can be actively harmful" and favours integration tests (O6 [A]).

**Cost.** Test doubles for every slow dependency (B2 ch. 11 [A]), with the coupling costs of P10.

**Mechanics.** Speed is a runtime fact. A direct call to a banned API (a sleep, a socket) is decidable as syntax (tsDetect "Sleepy Test", T6 [A]); a call through a helper is missed. Google enforces sizes at run time: "our definitions of small, medium, and large are actually encoded as constraints the testing infrastructure can enforce on a test" (B2 ch. 11 [A]).

### Group F. Adequacy: does the suite find faults?

#### P12. Adequacy: coverage, assertions, mutants

**Asks.** Measure whether tests can find faults, and act on the measure.

**Holds coverage as a requirement.**
- NASA: "the project manager shall ensure that there is 100 percent code test coverage using the Modified Condition/Decision Coverage (MC/DC) criterion for all identified safety-critical software components." (S4 [A])
- ISTQB: "The coverage criteria can function as key performance indicators" (S2 [A]).

**Contests coverage as a target.**
- Inozemtseva and Holmes (31,000 suites, five systems up to 724,000 lines): "there is a low to moderate correlation between coverage and effectiveness when the number of test cases in the suite is controlled for", and coverage "should not be used as a quality target" (E3 [A]).
- Google's book: engineers treat an 80% bar "like a ceiling" (B2 ch. 11 [A]). Google "does not enforce any code coverage thresholds across the entire codebase" (E6 [A]); of 286 write-in survey answers (173 + 67 + 46), 60% were positive about coverage, 16% negative (E6 [A]).
- Microsoft: "A high code coverage percentage isn't an indicator of success" (O3 [A]).
- Fowler: "I would be suspicious of anything like 100%" (B6 [A]).
- Practitioners: 4.02 for "Higher coverage does not mean that a test suite can detect more bugs" (E9 [A]).

**Better-supported measures.**
- Assertions: see P1 (E4 [A]).
- Mutants: Just et al., 357 real faults: "a statistically significant correlation between mutant detection and real fault detection, independently of code coverage" (E5 [A]). Petrović et al., almost 15 million mutants at Google: "developers using mutation testing write more tests" and "mutants are indeed coupled with real faults" (E7 [A]). SWEBOK lists mutation score as a measure (S1 [A]).

**Also contested here.** Kochhar's practitioners gave 3.89 to "Test even the simplest things that cannot go wrong" (E9 [A]); Fowler warns that coverage targets produce "lots of tests looking for things that rarely go wrong" (B6 [A]). ISTQB's principle "Tests wear out" holds that repeated tests "become increasingly ineffective in detecting new defects", while allowing that "repeating the same tests can have a beneficial outcome, e.g., in automated regression testing" (S2 [A]).

**Cost.** MC/DC needs many cases; mutation analysis needs compute (E7 [A]).

**Mechanics.** None of these is a property of one test's code. Coverage and mutation score need execution.

### Group G. Process

#### P13. Write the test first

**Asks.** Write a failing test, then the code, then refactor.

**Holds it.**
- SWEBOK (Software Construction, §4.16): "Test-first programming can usually detect defects earlier and correct them more easily than traditional programming styles." (S1 [A])
- ISTQB describes TDD as an approach that implements early testing (S2 [A]).
- Meszaros: "Write the Tests First" (B1 [A]); the Microsoft playbook calls TDD "a technique for writing your code that will lead you to a testable design from the start" (O4 [A]).

**Evidence and contest.** Fucci et al., 82 data points from 39 professionals: "Sequencing, the order in which test and production code are written, had no important influence." "The claimed benefits of TDD may not be due to its distinctive test-first dynamic, but rather due to the fact that TDD-like processes encourage fine-grained, steady steps that improve focus and flow." (E8 [A]) A Microsoft/IBM case study reports fewer defects with TDD at a 15–35% time cost, but I could not fetch it (E19 [B]).

**Mechanics.** Not from a test's code; it needs the commit history.

#### P14. Do not leave tests switched off

**Asks.** No disabled, ignored or quarantined test stays that way unnoticed.

**Holds it.** eslint-plugin-jest `no-disabled-tests` warns in the recommended set (T5 [A]). Meszaros names the risk: removing an erratic test "would result in an (intentional) Lost Test" (B1 [A]). Fowler: quarantined tests are "useless as regression tests", so cap the quarantine, e.g. "only allow 8 tests in quarantine" (B6 [A]). tsDetect lists "Ignored Test" (T6 [A]).

**Contested.** Mildly. JUnit endorses disabling with a reason: "the JUnit team recommends that developers provide a short explanation for why a test class or test method has been disabled" (T1 [A]). Google allows marking a test flaky, at a cost: it "encourages developers to ignore flakiness in their own tests" (O1 [A]).

**Mechanics.** The closest to decidable in this survey. A skip marker is syntax, and `no-disabled-tests` lists the forms it detects, including `.skip`, an `x` prefix, and "a test with a name but no function body" (T5 [A]). Whether the skip is legitimate is a policy, not a fact in the code.

## 3b. Practices the field held and later dropped or softened

| practice | what replaced it | what moved it | sources |
|---|---|---|---|
| Capture-and-playback ("Recorded Test") for regression tests | scripted tests | Fragility under UI change and unreadable recordings: "particularly prone to fragility if the user interface of the SUT is evolving" (B1 [A]); "Hard to maintain, scale and evolve" (S3 [A]). Not gone: the ISTQB syllabus still teaches it, for "a small scope and an SUT that rarely changes" (S3 [A]). Google's record/replay of *traffic* to services is a different technique (B2 ch. 14 [A]). | B1, S3, B2 |
| JUnit `@Test(expected = …)`, then the `ExpectedException` rule | `assertThrows` (JUnit 4.13 and later) | `expected` "will pass if *any* code in the method throws"; the rule "has been deprecated in JUnit 4.13" (T1 [A]) | T1 |
| A message on every assertion | assertion libraries and runners that report the failing assertion | Runner advances: in JUnit 4 the notion of a missing message as a smell is "rather doubtful" (E11 [A]). Still encoded: PMD's rule dates from PMD 1.04 and still exists (T2 [A]). | B1, E11, T2 |
| Strict one assertion per test | one behaviour per test; keep-going assertions | Meszaros "wouldn't be too dogmatic" (B1 [A]); GoogleTest, Go and JUnit `assertAll` report several failures (T1, T3, T4 [A]); Eager Test called "a matter of preference" (E11 [A]). Still encoded: PMD's default limit is 1 (T2 [A]). | B1, T1–T4, E11 |
| Heavy use of mocking frameworks (at Google) | real implementations and fakes | Maintenance cost "while rarely finding bugs" (B2 ch. 13 [A]). This is one organisation's change, stated from experience, not measured. | B2 |
| Google's "Test Certified" levels | "Project Health (pH)" metrics | Google says pH replaced Test Certified (B2 ch. 11 [A]). The book gives no reason I could quote; [C] for why. | B2 |
| A fixed, relied-upon test order | default order that is "intentionally nonobvious" (JUnit); opt-in shuffling (GoogleTest, Go) | Tools now offer this (T1, T3, T4 [A]). I found no source that ties the change to the studies of order dependence (E1, E2); [C] for the cause. | T1, T3, T4 |
| The ISTQB principle now called "Tests wear out" | — | v4.0.1 uses this name (S2 [A]). That earlier syllabi called it the "pesticide paradox" is from memory; I could not fetch an older syllabus ([B]). | S2 |

Two practices are contested now but not dropped: the test pyramid (Spotify against, O6 [A]) and test-first as the active ingredient of TDD (E8 [A]).

## 5. What a tool can decide from a test's code alone

"No false positive" is a strong bar.
Every practice above passes it only for a piece of syntax that stands in for the practice.
The rows below separate the syntax from the practice.

| practice | syntax a tool can decide exactly | tool that checks it | why the practice itself is not decidable | known false positives |
|---|---|---|---|---|
| P1 self-checking | a call to a known assertion name in the test body | PMD `UnitTestShouldIncludeAssert`; eslint-plugin-jest `expect-expect`; tsDetect Unknown Test | assertions in helpers, soft-assertion rules, expected-exception annotations; whether the assertion runs is a runtime fact | pmd#6681, pmd#6431, pmd#6188; the `assertFunctionNames` option exists to avoid them; DrTest's false positives (E14) |
| P2 deterministic | a direct `sleep` call | tsDetect Sleepy Test | flakiness is a property of runs; most causes are not a sleep call (E1) | a test that exercises real time |
| P3 independent | none useful | — (dynamic: shuffling in GoogleTest, `go test`) | NP-complete special case; detection is sound but incomplete (E2) | — |
| P4 one condition | count of known assertion calls | PMD `UnitTestContainsTooManyAsserts` (default 1); eslint-plugin-jest `max-expects` (default 5) | one behaviour can need many assertions | single-assertion tests flagged as Assertion Roulette (E11) |
| P5 failure output | presence of a message argument | PMD `UnitTestAssertionsShouldIncludeMessage` | message quality; libraries that print the subject | flagged as "doubtful" under JUnit 4 (E11) |
| P6 no logic | `if`, `switch`, ternary, loop in a test body | eslint-plugin-jest `no-conditional-in-test`, `no-conditional-expect`; tsDetect Conditional Test Logic | loops over a table are endorsed by Go (T4) | table-driven and property tests |
| P7 literal over shared | a literal in an assertion | tsDetect Magic Number Test | the sources disagree on which is the defect | every literal Google would keep (B2) |
| P11 fast, hermetic | a direct call to a banned API | tsDetect Sleepy Test; Google enforces sizes at run time instead | speed is runtime; helpers hide calls | as P2 |
| P14 not switched off | a skip marker | eslint-plugin-jest `no-disabled-tests`; tsDetect Ignored Test | whether a skip is legitimate is policy | a skip with a reason, as JUnit recommends |
| well-formed test declaration | malformed names, wrong signatures | `go vet` `tests` analyzer: "checks for common mistaken usages of tests and examples" (T4 [A]) | this is the one row where syntax *is* the practice | none found ([C]: I did not search its tracker) |

The detectors' own evaluations disagree with outside ones.
tsDetect's authors report "an average precision score of 96% and an average recall score of 97%" on 65 files (E17 [A]).
Panichella et al., labelling tests by hand, found the older detector "misclassified over 70% of test smells" on generated tests (E12 [A]), and Spadini et al. found that "current detection rules for certain test smells are considered as too strict by the developers" (E13 [A]).
Panichella et al. warn about "self-evaluating the validity of heuristics" (E11 [A]).

## [C] rows and what I could not do

| item | why |
|---|---|
| ISO/IEC/IEEE 29119 (S5): what it holds on test code | iso.org and standards.ieee.org returned HTTP 403; the standard is paywalled. I cannot say what it holds. |
| The "Stop 29119" petition, a known dispute about the standard | ipetitions.com returned a Cloudflare challenge. |
| Khorikov's four pillars, beyond the chapter headings | Manning's liveBook text is paywalled. Headings fetched: "The third and fourth pillars: Fast feedback and maintainability". The names of the first two pillars are [B]. |
| Beck, *Test-Driven Development: By Example*; Freeman and Pryce, *Growing Object-Oriented Software*; Feathers, *Working Effectively with Legacy Code*; Cohn, *Succeeding with Agile*; Meszaros's printed book | Not fetchable. I used the authors' own papers, essays and pre-publication pages instead (B1, B4, B5, B7). xunitpatterns.com pages state that the published chapter "has likely changed substanstially" [sic]. |
| Nagappan et al. 2008, TDD at Microsoft and IBM (E19) | microsoft.com returned HTTP 403. Its figures above are [B], from search-result summaries. |
| Bavota et al. 2015, "Are test smells really harmful?"; Tufano et al. 2016; Rafique and Mišić 2013 (TDD meta-analysis) | Publisher abstracts are elided in the Semantic Scholar API, and the PDFs are closed or moved. Not used. |
| Full text of Spadini 2017 (E16), Parry 2022 (E15), Delplanque 2019 (E14), Aranega 2021 (rotten green tests in Java, Pharo and Python) | Repository downloads returned 403, 400 or an Anubis bot check. I used abstracts or a portal summary only. |
| A tool that checks "only mock types you own" | Not found. |
| Why Google replaced Test Certified; why runners added shuffling | Not stated in the fetched sources. |
| `go vet` `tests` analyzer false positives | Not searched. |

## Source register

Fetch commands: `curl -sSL -A 'Mozilla/5.0 (X11; Linux x86_64)' -o <file> <url>`, then `pdftotext` for PDFs (poppler-utils via `nix-shell`), or the WebFetch tool where marked.
Abstracts marked "S2 API" were fetched with `curl 'https://api.semanticscholar.org/graph/v1/paper/DOI:<doi>?fields=title,year,venue,authors,abstract'`; they are the publisher's abstract as mirrored there.
GitHub data was fetched with `gh search issues` or `gh api`.

| id | source | fetched from | tier |
|---|---|---|---|
| S1 | Washizaki (ed.), *SWEBOK Guide v4.0*, IEEE Computer Society, 2024; ch. 4 §4.16, ch. 5 | https://mycourses.aalto.fi/pluginfile.php/2367766/mod_resource/content/3/swebok-v4.pdf (a course mirror; computer.org gates the PDF behind a form) | [A] mirror; whether it is v4.0 or v4.0a not checked |
| S2 | ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1*, 2024-09-15 | https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf | [A] |
| S3 | ISTQB, *Certified Tester Advanced Level Test Automation Engineering v2.0*, 2024-05-03 | https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTAL-TAE_Syllabus_v2.0.pdf | [A] |
| S4 | NASA, NPR 7150.2D, SWE-219, via the NASA Software Engineering Handbook | WebFetch https://swehb.nasa.gov/display/SWEHBVD/SWE-219+-+Code+Coverage+for+Safety+Critical+Software | [A] |
| S5 | ISO/IEC/IEEE 29119-1:2022 | https://www.iso.org/standard/81291.html → 403 | [C] |
| B1 | Meszaros, *xUnit Test Patterns*, 2007; pre-publication pages: Goals, Principles, Assertion Roulette, Conditional Test Logic, Erratic Test, Fragile Test, Slow Tests, Recorded Test, Shared Fixture, Assertion Message, Four Phase Test, Parameterized Test, Testcase Class per Class | `curl http://xunitpatterns.com/<Page%20Name>.html` (HTTPS refused) | [A] for the pages; the printed book [B] |
| B2 | Winters, Manshreck, Wright, *Software Engineering at Google*, O'Reilly 2020, ch. 11–14 | https://abseil.io/resources/swe-book/html/ch11.html … ch14.html | [A] |
| B3 | Khorikov, *Unit Testing Principles, Practices, and Patterns*, Manning 2020, ch. 4 | https://livebook.manning.com/book/unit-testing/chapter-4/ | [A] headings only |
| B4 | Beck, *Test Desiderata* | https://testdesiderata.com/ | [A] |
| B5 | Freeman, Mackinnon, Pryce, Walnes, "Mock Roles, not Objects", OOPSLA 2004; Mackinnon, Freeman, Craig, "Endo-Testing: Unit Testing with Mock Objects", XP 2000 | http://jmock.org/oopsla2004.pdf; https://www2.ccs.neu.edu/research/demeter/related-work/extreme-programming/MockObjectsFinal.PDF | [A] |
| B6 | Fowler, "Mocks Aren't Stubs"; "Eradicating Non-Determinism in Tests"; "TestCoverage" | https://martinfowler.com/articles/mocksArentStubs.html; https://martinfowler.com/articles/nonDeterminism.html; https://martinfowler.com/bliki/TestCoverage.html | [A] |
| B7 | Feathers, "A Set of Unit Testing Rules", 2005-09-09 | https://www.artima.com/weblogs/viewpost.jsp?thread=126923 | [A] |
| E1 | Luo, Hariri, Eloussi, Marinov, "An Empirical Analysis of Flaky Tests", FSE 2014, doi:10.1145/2635868.2635920 | https://huang.isis.vanderbilt.edu/cs8395/paper/flakytest.pdf (author-formatted copy) | [A] |
| E2 | Zhang et al., "Empirically Revisiting the Test Independence Assumption", ISSTA 2014 | https://homes.cs.washington.edu/~mernst/pubs/test-independence-issta2014.pdf | [A] |
| E3 | Inozemtseva, Holmes, "Coverage Is Not Strongly Correlated with Test Suite Effectiveness", ICSE 2014 | https://cs.ubc.ca/~rtholmes/papers/icse_2014_inozemtseva.pdf | [A] |
| E4 | Zhang, Mesbah, "Assertions Are Strongly Correlated with Test Suite Effectiveness", ESEC/FSE 2015 | https://people.ece.ubc.ca/amesbah/resources/papers/fse15.pdf | [A] |
| E5 | Just et al., "Are Mutants a Valid Substitute for Real Faults in Software Testing?", FSE 2014 | https://homes.cs.washington.edu/~rjust/publ/mutants_real_faults_fse_2014.pdf | [A] |
| E6 | Ivanković, Petrović, Just, Fraser, "Code Coverage at Google", ESEC/FSE 2019 | https://homes.cs.washington.edu/~rjust/publ/google_coverage_fse_2019.pdf | [A] |
| E7 | Petrović, Ivanković, Fraser, Just, "Does Mutation Testing Improve Testing Practices?", ICSE 2021 | https://arxiv.org/pdf/2103.07189 | [A] |
| E8 | Fucci et al., "A Dissection of the Test-Driven Development Process", IEEE TSE 2017 | https://arxiv.org/pdf/1611.05994 | [A] |
| E9 | Kochhar, Xia, Lo, "Practitioners' Views on Good Software Testing Practices", ICSE-SEIP 2019 | https://soarsmu.github.io/papers/2019/Kochhar2019SoftwareTestingPractices.pdf | [A] |
| E10 | Spadini et al., "On the Relation of Test Smells to Software Code Quality", ICSME 2018 | https://zenodo.org/record/1689875/files/main.pdf (preprint) | [A] |
| E11 | Panichella et al., "Test Smells 20 Years Later: Detectability, Validity, and Reliability", EMSE 27(170), 2022 | https://pure.tudelft.nl/ws/portalfiles/portal/137994226/s10664_022_10207_5.pdf | [A] |
| E12 | Panichella et al., "Revisiting Test Smells in Automatically Generated Tests", ICSME 2020, doi:10.1109/ICSME46990.2020.00056 | S2 API | [A] abstract |
| E13 | Spadini et al., "Investigating Severity Thresholds for Test Smells", MSR 2020, doi:10.1145/3379597.3387453 | S2 API | [A] abstract |
| E14 | Delplanque, Ducasse, Polito, Black, Etien, "Rotten Green Tests", ICSE 2019, doi:10.1109/ICSE.2019.00062 | S2 API | [A] abstract |
| E15 | Parry, Kapfhammer, Hilton, McMinn, "A Survey of Flaky Tests", ACM TOSEM 2022, doi:10.1145/3476105 | S2 API | [A] abstract |
| E16 | Spadini, Aniche, Bruntink, Bacchelli, "To Mock or Not To Mock?", MSR 2017 | WebFetch https://research.tudelft.nl/en/publications/to-mock-or-not-to-mock-an-empirical-study-on-mocking-practices | [A] summary and short quotes, via WebFetch's model |
| E17 | Peruma et al., "tsDetect: An Open Source Test Smells Detection Tool", ESEC/FSE 2020 demo, doi:10.1145/3368089.3417921 | S2 API | [A] abstract |
| E18 | Claessen, Hughes, "QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs", ICFP 2000 | https://www.cs.tufts.edu/comp/150FP/archive/john-hughes/quick.pdf | [A] |
| E19 | Nagappan, Maximilien, Bhat, Williams, "Realizing Quality Improvement through Test Driven Development", EMSE 2008 | microsoft.com → 403 | [B] |
| O1 | Micco, "Flaky Tests at Google and How We Mitigate Them", Google Testing Blog, 2016-05 | https://testing.googleblog.com/2016/05/flaky-tests-at-google-and-how-we.html | [A] |
| O2 | "Just Say No to More End-to-End Tests", Google Testing Blog, 2015-04 | https://testing.googleblog.com/2015/04/just-say-no-to-more-end-to-end-tests.html | [A] |
| O3 | Microsoft, "Best practices for writing unit tests" (.NET), `ms.date` 2026-11-05 | https://learn.microsoft.com/en-us/dotnet/core/testing/unit-testing-best-practices | [A] |
| O4 | Microsoft, *Engineering Fundamentals Playbook*, "Unit Testing" | https://microsoft.github.io/code-with-engineering-playbook/automated-testing/unit-testing/ | [A] |
| O5 | Meta, "Probabilistic flakiness: How do you test your tests?", 2020-12-10 | https://engineering.fb.com/2020/12/10/developer-tools/probabilistic-flakiness/ | [A] |
| O6 | Spotify, "Testing of Microservices", 2018-01 | https://engineering.atspotify.com/2018/01/testing-of-microservices | [A] |
| T1 | JUnit user guide 6.1.3: Test Execution Order, Test Instance Lifecycle, Assertions, Disabling Tests; JUnit 4 wiki "Exception testing" | https://docs.junit.org/6.1.3/writing-tests/<page>.html; https://raw.githubusercontent.com/wiki/junit-team/junit4/Exception-testing.md | [A] |
| T2 | PMD Java "Best Practices" rules; PMD issues #6681, #6431, #6188 | https://docs.pmd-code.org/latest/pmd_rules_java_bestpractices.html; `gh search issues --repo pmd/pmd 'UnitTestShouldIncludeAssert false positive'` | [A]; issues by title only |
| T3 | GoogleTest Primer and Advanced guide | https://google.github.io/googletest/primer.html; https://google.github.io/googletest/advanced.html | [A] |
| T4 | Go: `go help testflag` text on pkg.go.dev; Go wiki TestComments and TableDrivenTests; `go vet` `tests` analyzer | https://pkg.go.dev/cmd/go; https://go.dev/wiki/TestComments; https://go.dev/wiki/TableDrivenTests; https://pkg.go.dev/golang.org/x/tools/go/analysis/passes/tests | [A] |
| T5 | eslint-plugin-jest rule docs (`expect-expect`, `no-conditional-expect`, `no-conditional-in-test`, `max-expects`, `no-disabled-tests`) at commit `412426e`; issue #1475 | https://raw.githubusercontent.com/jest-community/eslint-plugin-jest/main/docs/rules/<rule>.md; `gh api search/issues` | [A]; issue by title only |
| T6 | tsDetect test smell catalogue | https://testsmells.org/pages/testsmells.html | [A] |
