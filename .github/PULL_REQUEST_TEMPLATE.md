<!-- How to write it, whichever section you are in.

     what you owe:  docs/decisions/a-change-is-specified-before-it-is-built.md
     the decision:  docs/decisions/an-artifact-holds-the-minimum-that-conveys-its-point.md

     Name the issue this answers on the first line, as Closes #<n>. -->

## What you must decide

<!-- Each decision that is the reviewer's, in bold, then what follows from it.

     Example. One pull request runs through every section of this template.

       **Whether `Unicode-3.0` joins the allowlist.** One crate needs it, and
       an exception scoped to that crate is the other answer. -->

## Implementation plan

<!-- The steps in order, and the files each one writes.

     Example, continued.

       1. Write `deny.toml` with an empty allowlist.
       2. Run `cargo deny check licenses`, and add each licence it names,
          with a comment naming the crate that forced it.
       3. Add the check to `quality`. -->
