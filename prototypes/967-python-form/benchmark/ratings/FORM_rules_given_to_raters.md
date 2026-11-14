# The rules of this form

A prototype test form, run by its own small runner (`run.py`, `engine.py`, `form_experiments.py`).

- **An experiment is a class whose fields are its inputs.** It subclasses `Experiment[P]` or `TypedExperiment[P]`, where `P` is the type of the product. A field with a default is a value every case shares; a case may override it. A fixture is a field whose default is `use(fixture)`.
- **`act` is ONE call, and its return value is the product.** A nested call, a literal argument, `if`/`for`/`try`, or a function of the test file as the act is refused before the run.
- **Every other method is an assay.** It reads `self.product` or one field, in one read, and hands back a literal reading (str, int, float, bool, bytes, None, or a list/tuple/set/dict of them).
- **A case is an instance, named by its variable:** `plain = SlugOf(text="Hello World").expect(product="hello-world")`. `.raises(Type, "message part")` expects a raise; `.warns(...)` expects a warning; an unexpected warning fails the case.
- **Two levels.** At the small level (`Experiment`) a test writes no `expect` declaration; the runner checks each expected value against its assay's return type before the run. At the typed level (`TypedExperiment`) the experiment declares `def expect(self, *, <assay>: <type> = UNREAD, …) -> Self: return self`, so a type checker also sees each expected value. A test grows from one level to the other by changing its base and adding `expect`.
- **A multi-step process is a chain.** A later case takes an earlier case's `step.product` or `step.<field>` as an input; the runner replays the earlier step's act and assays inside the later case. A replay that agrees prints nothing; one that differs blocks the case. A failed step blocks the steps after it.
- **Each case runs in its own forked process,** and each assay but the last in a fork of its own, so no case or assay can change what another reads. A reading is compared in the assay's process, by type first, then by value (`True` is not `1`).
- **The runner also refuses, before the run:** a misspelled input or assay, an expected value that does not fit its assay's type, a case name bound twice in a file, cases built by a loop or a list, a subclass of an experiment, a field and an assay with one name, and a case or experiment that checks nothing. A refusal applies to the whole experiment.
