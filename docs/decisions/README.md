# Decision records by kind

PROTOTYPE for #801. A script writes this page from the **Topic:** line in each record.
A record of two kinds is listed under both, marked *also*.

## Product design, whole (14)

- [A concern is what no part can own](a-concern-is-what-no-part-can-own.md)
- [A design in depth waits for what it assumes](a-design-in-depth-waits-for-what-it-assumes.md)
- [A gate refuses each invariant a tool can see, from the first code it reads](a-gate-refuses-each-invariant-a-tool-can-see.md) *also*
- [An invariant needs no design of its own](an-invariant-needs-no-design-of-its-own.md)
- [A pinned candidate decides when a milestone is done](a-pinned-candidate-decides-when-a-milestone-is-done.md)
- [Each part is a crate](each-part-is-a-crate.md)
- [Every crate lives under crates/](every-crate-lives-under-crates.md) *also*
- [Every part of the skeleton is Rust, inside the extension module](every-part-of-the-skeleton-is-rust-inside-the-extension-module.md)
- [A script calculates the order of design, and only a changed input moves it](only-a-changed-input-moves-the-order-of-design.md)
- [purba divides by responsibility and not by language](purba-divides-by-responsibility-and-not-by-language.md)
- [purba ranks an honest outcome first](purba-ranks-an-honest-outcome-first.md)
- [purba runs inside the tester's interpreter](purba-runs-inside-the-testers-interpreter.md)
- [purba's public surface starts empty and has no plugin API](purbas-public-surface-starts-empty-and-has-no-plugin-api.md)
- [purba supports a Python that has two years left](purba-supports-a-python-with-two-years-left.md)

## One section of the product design (35)

### assertion

- [A loaded module joins sys.modules under its file's name, without .py](a-loaded-module-joins-sys-modules-under-its-files-name.md)
- [A module that binds a name purba adds is a project error](a-module-that-binds-a-name-purba-adds-is-a-project-error.md)
- [A module whose load fails leaves sys.modules as it found it](a-module-whose-load-fails-leaves-sys-modules-as-it-found-it.md)
- [Assertion's rewrite crosses each part whole, and only assertion opens it](assertions-rewrite-crosses-each-part-whole-and-only-assertion-opens-it.md)

### command

- [A command test checks only command's rules](a-command-test-checks-only-commands-rules.md)
- [A tester runs purba on one module, and reads one line for each test](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md)
- [Command lends one parse to selection and assertion](command-lends-one-parse-to-selection-and-assertion.md)

### dispatch

- [A worker loads each module once, and command hands each module once](a-worker-loads-each-module-once.md)
- [Each test runs in a process of its own](each-test-runs-in-a-process-of-its-own.md)

### errors

- [A blocked test has no outcome](a-blocked-test-has-no-outcome.md)
- [A case of each kind checks that no failure prints as another kind](a-case-of-each-kind-checks-that-no-failure-prints-as-another-kind.md)
- [A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md)
- [A line names its kind in the glossary's words](a-line-names-its-kind-in-the-glossarys-words.md)
- [An error says its kind, and reporting takes four ways in](an-error-says-its-kind-and-reporting-takes-four-ways-in.md)
- [An error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md)
- [A panic inside a test gives that test an internal error](a-panic-inside-a-test-gives-that-test-an-internal-error.md)
- [A test causes an internal error through a switch that only a test build carries](a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md)

### execution

- [Execution calls a test by its function's name, and reporting prints the identity it is given](execution-calls-a-test-by-its-functions-name.md)

### fixtures

- [A test names the fixture of each input](a-test-names-the-fixture-of-each-input.md)
- [A test names two values of one type by two fixtures](a-test-names-two-values-of-one-type-by-two-fixtures.md)

### interrupt handling

- [A KeyboardInterrupt that no signal caused is the test's own raise](a-keyboardinterrupt-that-no-signal-caused-is-the-tests-own-raise.md)
- [An interrupted run prints its count, and dies by the signal](an-interrupted-run-prints-its-count-and-dies-by-the-signal.md)
- [An interrupt puts back no more than a normal end does](an-interrupt-puts-back-no-more-than-a-normal-end-does.md)
- [An interrupt stops the run, and a second one exits at once](an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md)

### parsing

- [Parsing hands on no tree that ruff recovered](parsing-hands-on-no-tree-that-ruff-recovered.md)
- [purba parses Python with ruff's parser](purba-parses-python-with-ruff.md)

### reporting

- [A run exits 0 only when it checked something and every test passed](a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md)
- [A test that selection refuses prints before any test runs](a-test-that-selection-refuses-prints-before-any-test-runs.md)
- [Reporting renders the exit code, and command returns it](reporting-renders-the-exit-code-and-command-returns-it.md)
- [Reporting writes each line as dispatch hands it the test](reporting-writes-each-line-as-dispatch-hands-it-the-test.md)

### selection

- [A test carries only purba's own decorators](a-test-carries-only-purbas-own-decorators.md)
- [A test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md)

### strictness

- [A suppression is an outcome and not a switch](a-suppression-is-an-outcome-and-not-a-switch.md)
- [purba is always strict](purba-is-always-strict.md)

### the Rust/Python seam

- [The skeleton crosses the seam in execution and assertion](the-skeleton-crosses-the-seam-in-execution-and-assertion.md)


## Pull-request workflow (12)

- [A change is specified before it is built](a-change-is-specified-before-it-is-built.md)
- [A commit message outlives its review](a-commit-outlives-its-review.md)
- [An artifact is rewritten until its direction is agreed](an-artifact-is-rewritten-until-its-direction-is-agreed.md)
- [An implementation follows test-driven development](an-implementation-follows-test-driven-development.md)
- [An outside contribution is applied, not merged](an-outside-contribution-is-applied-not-merged.md)
- [A standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md) *also*
- [Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md)
- [One pull request answers one issue](one-pull-request-answers-one-issue.md)
- [purba records delegation and does not prevent it](purba-records-delegation-and-does-not-prevent-it.md)
- [Architectural significance is declared, not detected](significance-is-declared-not-detected.md)
- [The agent that marks a pull request ready watches it](the-agent-that-marks-a-pull-request-ready-watches-it.md)
- [The person owns the Decision Outcome](the-person-owns-the-decision-outcome.md) *also*

## How a record is written (17)

- [A commit message outlives its review](a-commit-outlives-its-review.md) *also*
- [A figure in a record is a fence GitHub renders](a-figure-in-a-record-is-a-fence-github-renders.md)
- [A location inherits its readers](a-location-inherits-its-readers.md)
- [An actor is what it does, not what it is](an-actor-is-what-it-does-not-what-it-is.md)
- [An artifact holds the minimum that conveys its point](an-artifact-holds-the-minimum-that-conveys-its-point.md)
- [A new question gets a new record](a-new-question-gets-a-new-record.md)
- [An unpublished comment carries what no other location carries](an-unpublished-comment-carries-what-no-other-location-carries.md)
- [A part-of-speech tagger decides the tense rule](a-part-of-speech-tagger-decides-the-tense-rule.md)
- [A decision record is rewritten, not amended](a-record-is-rewritten-not-amended.md)
- [A record names every location where it applies](a-record-names-every-location-where-it-applies.md)
- [A refusal states the rule, why it holds, and where it was found](a-refusal-states-the-rule-why-it-holds-and-where-it-was-found.md) *also*
- [A register belongs to one location](a-register-belongs-to-one-location.md)
- [A term belongs to the glossary](a-term-belongs-to-the-glossary.md)
- [purba borrows from Simplified Technical English rather than adopting it](purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md)
- [Architectural significance is declared, not detected](significance-is-declared-not-detected.md) *also*
- [The glossary stays open, and a gate rewrites each word it avoids](the-glossary-stays-open-and-a-gate-rewrites-each-word-it-avoids.md)
- [The person owns the Decision Outcome](the-person-owns-the-decision-outcome.md)

## Repository tooling (29)

- [A gate owns the mechanical standard](a-gate-owns-the-mechanical.md)
- [A gate reads its marks on the base branch](a-gate-reads-its-marks-on-the-base-branch.md)
- [A gate refuses each invariant a tool can see, from the first code it reads](a-gate-refuses-each-invariant-a-tool-can-see.md)
- [A normaliser's style is chosen for the human who reads the code](a-normalisers-style-is-chosen-for-the-human-who-reads-the-code.md)
- [A part-of-speech tagger decides the tense rule](a-part-of-speech-tagger-decides-the-tense-rule.md) *also*
- [A path not marked to skip starts the gate](a-path-not-marked-to-skip-starts-the-gate.md)
- [A refusal states the rule, why it holds, and where it was found](a-refusal-states-the-rule-why-it-holds-and-where-it-was-found.md)
- [A script that needs a Rust library is a cargo script](a-script-that-needs-a-rust-library-is-a-cargo-script.md)
- [A standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md)
- [A task is named for what it does to the tree](a-task-is-named-for-what-it-does-to-the-tree.md)
- [A test causes an internal error through a switch that only a test build carries](a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md) *also*
- [A workflow is granted an identity, not a power](a-workflow-is-granted-an-identity-not-a-power.md)
- [A workflow names each action by its commit](a-workflow-names-each-action-by-its-commit.md)
- [bats tests the installed wheel from outside purba](bats-tests-the-installed-wheel-from-outside-purba.md)
- [Configuration takes the first location its tool reads](configuration-takes-the-first-location-its-tool-reads.md)
- [Dependabot proposes each newer action](dependabot-proposes-each-newer-action.md)
- [Every crate lives under crates/](every-crate-lives-under-crates.md)
- [One file marks every tracked path for each gate that can skip](one-file-marks-every-tracked-path-for-each-gate-that-can-skip.md)
- [One manifest declares each package](one-manifest-declares-each-package.md)
- [A script calculates the order of design, and only a changed input moves it](only-a-changed-input-moves-the-order-of-design.md) *also*
- [Only GitHub runs what lives under `.github`](only-github-runs-what-lives-under-github.md)
- [purba carries no tool manager beside mise, and no compiler of its own](purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md)
- [purba meets the next trait solver before it stabilizes](purba-meets-the-next-trait-solver-before-it-stabilizes.md)
- [purba supports a Python that has two years left](purba-supports-a-python-with-two-years-left.md) *also*
- [purba writes its scripts in bash until purba can test them](purba-writes-its-scripts-in-bash-until-purba-can-test-them.md)
- [The crate carries an rlib](the-crate-carries-an-rlib.md)
- [The glossary stays open, and a gate rewrites each word it avoids](the-glossary-stays-open-and-a-gate-rewrites-each-word-it-avoids.md) *also*
- [The nightly is named by a date](the-nightly-is-named-by-a-date.md)
- [The zip alerts of check-prose are dismissed as unused](the-zip-alerts-of-check-prose-are-dismissed-as-unused.md)
