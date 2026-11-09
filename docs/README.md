# The decision records, by area

`mise run fmt:index` writes this page from the `**Area:**` line of each record.

## Product design

- [A concern is what no part can own](decisions/a-concern-is-what-no-part-can-own.md)
- [A design in depth waits for what it assumes](decisions/a-design-in-depth-waits-for-what-it-assumes.md)
- [A gate refuses each invariant a tool can see, from the first code it reads](decisions/a-gate-refuses-each-invariant-a-tool-can-see.md)
- [A pinned candidate decides when a milestone is done](decisions/a-pinned-candidate-decides-when-a-milestone-is-done.md)
- [An invariant needs no design of its own](decisions/an-invariant-needs-no-design-of-its-own.md)
- [Each part is a crate](decisions/each-part-is-a-crate.md)
- [Every crate lives under crates/](decisions/every-crate-lives-under-crates.md)
- [Every part of the skeleton is Rust, inside the extension module](decisions/every-part-of-the-skeleton-is-rust-inside-the-extension-module.md)
- [A script calculates the order of design, and only a changed input moves it](decisions/only-a-changed-input-moves-the-order-of-design.md)
- [purba divides by responsibility and not by language](decisions/purba-divides-by-responsibility-and-not-by-language.md)
- [purba ranks an honest outcome first](decisions/purba-ranks-an-honest-outcome-first.md)
- [purba runs inside the tester's interpreter](decisions/purba-runs-inside-the-testers-interpreter.md)
- [purba runs on Linux and macOS, and a Windows user runs it under WSL](decisions/purba-runs-on-linux-and-macos.md)
- [purba supports a Python that has two years left](decisions/purba-supports-a-python-with-two-years-left.md)
- [purba's public surface starts empty and has no plugin API](decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md)

## One part or concern

### command

- [A command test checks only command's rules](decisions/a-command-test-checks-only-commands-rules.md)
- [A tester runs purba on one module, and reads one line for each test](decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md)
- [Command lends one parse to selection and assertion](decisions/command-lends-one-parse-to-selection-and-assertion.md)

### selection

- [A test carries only purba's own decorators](decisions/a-test-carries-only-purbas-own-decorators.md)
- [A test returns nothing, and the first part that sees a breach of a test refuses it](decisions/a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md)

### parsing

- [A failed line quotes the last assert that did not hold, without its comments](decisions/a-failed-line-quotes-the-last-assert-that-did-not-hold-without-its-comments.md)
- [Parsing hands on no tree that ruff recovered](decisions/parsing-hands-on-no-tree-that-ruff-recovered.md)
- [purba parses Python with ruff's parser](decisions/purba-parses-python-with-ruff.md)

### dispatch

- [A worker loads each module once, and command hands each module once](decisions/a-worker-loads-each-module-once.md)
- [Each test runs in a process of its own](decisions/each-test-runs-in-a-process-of-its-own.md)

### execution

- [A failed line quotes the last assert that did not hold, without its comments](decisions/a-failed-line-quotes-the-last-assert-that-did-not-hold-without-its-comments.md)
- [Execution calls a test by its function's name, and reporting prints the identity it is given](decisions/execution-calls-a-test-by-its-functions-name.md)

### fixtures

- [A test names the fixture of each input](decisions/a-test-names-the-fixture-of-each-input.md)
- [A test names two values of one type by two fixtures](decisions/a-test-names-two-values-of-one-type-by-two-fixtures.md)

### assertion

- [A failed line quotes the last assert that did not hold, without its comments](decisions/a-failed-line-quotes-the-last-assert-that-did-not-hold-without-its-comments.md)
- [A loaded module joins sys.modules under its file's name, without .py](decisions/a-loaded-module-joins-sys-modules-under-its-files-name.md)
- [A module that binds a name purba adds is a project error](decisions/a-module-that-binds-a-name-purba-adds-is-a-project-error.md)
- [A module whose load fails leaves sys.modules as it found it](decisions/a-module-whose-load-fails-leaves-sys-modules-as-it-found-it.md)
- [A tuple assert keeps CPython's warning and counts as no assert](decisions/a-tuple-assert-keeps-cpythons-warning-and-counts-as-no-assert.md)
- [Assertion's rewrite crosses each part whole, and only assertion opens it](decisions/assertions-rewrite-crosses-each-part-whole-and-only-assertion-opens-it.md)

### reporting

- [A run exits 0 only when it checked something and every test passed](decisions/a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md)
- [A test that selection refuses prints before any test runs](decisions/a-test-that-selection-refuses-prints-before-any-test-runs.md)
- [Command hands reporting each line from before dispatch takes the module](decisions/command-hands-reporting-each-line-from-before-dispatch-takes-the-module.md)
- [Reporting renders the exit code, and command returns it](decisions/reporting-renders-the-exit-code-and-command-returns-it.md)
- [Reporting writes each line as dispatch hands it the test](decisions/reporting-writes-each-line-as-dispatch-hands-it-the-test.md)
- [The report names only the free-threaded flag](decisions/the-report-names-only-the-free-threaded-flag.md)

### the Rust/Python seam

- [The skeleton crosses the seam in execution and assertion](decisions/the-skeleton-crosses-the-seam-in-execution-and-assertion.md)

### strictness

- [A suppression is an outcome and not a switch](decisions/a-suppression-is-an-outcome-and-not-a-switch.md)
- [purba is always strict](decisions/purba-is-always-strict.md)

### interrupt handling

- [A KeyboardInterrupt that no signal caused is the test's own raise](decisions/a-keyboardinterrupt-that-no-signal-caused-is-the-tests-own-raise.md)
- [A process that a test forks has the signal handlers it has without purba](decisions/a-process-that-a-test-forks-has-the-signal-handlers-it-has-without-purba.md)
- [An interrupt puts back no more than a normal end does](decisions/an-interrupt-puts-back-no-more-than-a-normal-end-does.md)
- [An interrupt stops the run, and a second one exits at once](decisions/an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md)
- [An interrupted run prints its count, and dies by the signal](decisions/an-interrupted-run-prints-its-count-and-dies-by-the-signal.md)

### errors

- [A blocked test has no outcome](decisions/a-blocked-test-has-no-outcome.md)
- [A case of each kind checks that no failure prints as another kind](decisions/a-case-of-each-kind-checks-that-no-failure-prints-as-another-kind.md)
- [A failure's kind is what the part that first sees it observes](decisions/a-failures-kind-is-what-the-part-that-first-sees-it-observes.md)
- [A line names its kind in the glossary's words](decisions/a-line-names-its-kind-in-the-glossarys-words.md)
- [A panic inside a test gives that test an internal error](decisions/a-panic-inside-a-test-gives-that-test-an-internal-error.md)
- [A test causes an internal error through a switch that only a test build carries](decisions/a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md)
- [An error says its kind, and reporting takes four ways in](decisions/an-error-says-its-kind-and-reporting-takes-four-ways-in.md)
- [An error stops only the work that depends on it](decisions/an-error-stops-only-the-work-that-depends-on-it.md)
- [Of the exceptions CPython ignores, only a thread's is a project error](decisions/of-the-exceptions-cpython-ignores-only-a-threads-is-a-project-error.md)

## The pull-request workflow

- [A change is specified before it is built](decisions/a-change-is-specified-before-it-is-built.md)
- [A commit message outlives its review](decisions/a-commit-outlives-its-review.md)
- [A required check of `main` counts only from GitHub Actions](decisions/a-required-check-of-main-counts-only-from-github-actions.md)
- [A standard a gate cannot decide does not become a gate](decisions/a-standard-a-gate-cannot-decide-does-not-become-a-gate.md)
- [An artifact is rewritten until its direction is agreed](decisions/an-artifact-is-rewritten-until-its-direction-is-agreed.md)
- [An implementation follows test-driven development](decisions/an-implementation-follows-test-driven-development.md)
- [An outside contribution is applied, not merged](decisions/an-outside-contribution-is-applied-not-merged.md)
- [Liability is recorded from the act that makes it true](decisions/liability-is-recorded-from-the-act-that-makes-it-true.md)
- [One pull request answers one issue](decisions/one-pull-request-answers-one-issue.md)
- [purba records delegation and does not prevent it](decisions/purba-records-delegation-and-does-not-prevent-it.md)
- [Architectural significance is declared, not detected](decisions/significance-is-declared-not-detected.md)
- [The agent that marks a pull request ready watches it](decisions/the-agent-that-marks-a-pull-request-ready-watches-it.md)
- [The person owns the Decision Outcome](decisions/the-person-owns-the-decision-outcome.md)

## How a record is written

- [A commit message outlives its review](decisions/a-commit-outlives-its-review.md)
- [A figure in a record is a fence GitHub renders](decisions/a-figure-in-a-record-is-a-fence-github-renders.md)
- [A location inherits its readers](decisions/a-location-inherits-its-readers.md)
- [A new question gets a new record](decisions/a-new-question-gets-a-new-record.md)
- [A part-of-speech tagger decides the tense rule](decisions/a-part-of-speech-tagger-decides-the-tense-rule.md)
- [A decision record is rewritten, not amended](decisions/a-record-is-rewritten-not-amended.md)
- [A record names every location where it applies](decisions/a-record-names-every-location-where-it-applies.md)
- [A record names its area, and docs/README.md lists the records by area](decisions/a-record-names-its-area-and-docs-readme-lists-the-records-by-area.md)
- [A record names the role, not the kind of entity, unless the kind is what it decides](decisions/a-record-names-the-role-not-the-kind-of-entity.md)
- [A refusal states the rule, why it holds, and where it was found](decisions/a-refusal-states-the-rule-why-it-holds-and-where-it-was-found.md)
- [A register belongs to one location](decisions/a-register-belongs-to-one-location.md)
- [A term belongs to the glossary](decisions/a-term-belongs-to-the-glossary.md)
- [An actor is what it does, not what it is](decisions/an-actor-is-what-it-does-not-what-it-is.md)
- [An artifact holds the minimum that conveys its point](decisions/an-artifact-holds-the-minimum-that-conveys-its-point.md)
- [An unpublished comment carries what no other location carries](decisions/an-unpublished-comment-carries-what-no-other-location-carries.md)
- [purba borrows from Simplified Technical English rather than adopting it](decisions/purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md)
- [Architectural significance is declared, not detected](decisions/significance-is-declared-not-detected.md)
- [The decision sentence is the winner, and Considered Options holds only what lost](decisions/the-decision-sentence-is-the-winner-and-considered-options-holds-only-what-lost.md)
- [The glossary stays open, and a gate rewrites each word it avoids](decisions/the-glossary-stays-open-and-a-gate-rewrites-each-word-it-avoids.md)
- [The person owns the Decision Outcome](decisions/the-person-owns-the-decision-outcome.md)

## Repository tooling

- [A failure report comments only on an issue that `GITHUB_TOKEN` opened](decisions/a-failure-report-comments-only-on-an-issue-that-github-token-opened.md)
- [A gate owns the mechanical standard](decisions/a-gate-owns-the-mechanical.md)
- [A gate reads its marks on the base branch](decisions/a-gate-reads-its-marks-on-the-base-branch.md)
- [A gate refuses each invariant a tool can see, from the first code it reads](decisions/a-gate-refuses-each-invariant-a-tool-can-see.md)
- [A normaliser's style is chosen for the reader of the code](decisions/a-normalisers-style-is-chosen-for-the-human-who-reads-the-code.md)
- [A part-of-speech tagger decides the tense rule](decisions/a-part-of-speech-tagger-decides-the-tense-rule.md)
- [A path not marked to skip starts the gate](decisions/a-path-not-marked-to-skip-starts-the-gate.md)
- [A record names its area, and docs/README.md lists the records by area](decisions/a-record-names-its-area-and-docs-readme-lists-the-records-by-area.md)
- [A refusal states the rule, why it holds, and where it was found](decisions/a-refusal-states-the-rule-why-it-holds-and-where-it-was-found.md)
- [A script that needs a Rust library is a cargo script](decisions/a-script-that-needs-a-rust-library-is-a-cargo-script.md)
- [A standard a gate cannot decide does not become a gate](decisions/a-standard-a-gate-cannot-decide-does-not-become-a-gate.md)
- [A task is named for what it does to the tree](decisions/a-task-is-named-for-what-it-does-to-the-tree.md)
- [A test causes an internal error through a switch that only a test build carries](decisions/a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md)
- [A workflow is granted an identity, not a power](decisions/a-workflow-is-granted-an-identity-not-a-power.md)
- [A workflow names each action by its commit](decisions/a-workflow-names-each-action-by-its-commit.md)
- [bats tests the installed wheel from outside purba](decisions/bats-tests-the-installed-wheel-from-outside-purba.md)
- [Configuration takes the first location its tool reads](decisions/configuration-takes-the-first-location-its-tool-reads.md)
- [Dependabot proposes each newer action](decisions/dependabot-proposes-each-newer-action.md)
- [Every crate lives under crates/](decisions/every-crate-lives-under-crates.md)
- [One file marks every tracked path for each gate that can skip](decisions/one-file-marks-every-tracked-path-for-each-gate-that-can-skip.md)
- [One manifest declares each package](decisions/one-manifest-declares-each-package.md)
- [A script calculates the order of design, and only a changed input moves it](decisions/only-a-changed-input-moves-the-order-of-design.md)
- [Only GitHub runs what lives under `.github`](decisions/only-github-runs-what-lives-under-github.md)
- [purba carries no tool manager beside mise, and no compiler of its own](decisions/purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md)
- [purba meets the next trait solver before it stabilizes](decisions/purba-meets-the-next-trait-solver-before-it-stabilizes.md)
- [purba runs on Linux and macOS, and a Windows user runs it under WSL](decisions/purba-runs-on-linux-and-macos.md)
- [purba supports a Python that has two years left](decisions/purba-supports-a-python-with-two-years-left.md)
- [purba writes its scripts in bash until purba can test them](decisions/purba-writes-its-scripts-in-bash-until-purba-can-test-them.md)
- [The crate carries an rlib](decisions/the-crate-carries-an-rlib.md)
- [The decision sentence is the winner, and Considered Options holds only what lost](decisions/the-decision-sentence-is-the-winner-and-considered-options-holds-only-what-lost.md)
- [The glossary stays open, and a gate rewrites each word it avoids](decisions/the-glossary-stays-open-and-a-gate-rewrites-each-word-it-avoids.md)
- [The nightly is named by a date](decisions/the-nightly-is-named-by-a-date.md)
- [The zip alerts of check-prose are dismissed as unused](decisions/the-zip-alerts-of-check-prose-are-dismissed-as-unused.md)
