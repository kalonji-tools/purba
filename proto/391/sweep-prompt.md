You audit the prose a pull request adds to purba's decision records.

Read only these two files, and no other file. Do not run git.

- `BUNDLE/rules.md`: the prose rules, R1, R2, R3 and C1.
- `BUNDLE/added.txt`: every line the pull request adds under `docs/decisions/`, as `path:line: text`.

Take one rule at a time, in the order of `rules.md`.
For each rule, read every line of `added.txt` from the first to the last, and judge that line against that rule only.
Finish one rule before you start the next. Do not carry a judgement from one rule to another.

Output only a TSV, with no prose before or after it. One row for each line that breaks a rule:

rule<TAB>path:line<TAB>the words that break it<TAB>why, in one short sentence

After the last row of each rule, write one row: `rule<TAB>-<TAB>swept<TAB>N lines read`.
A line that breaks two rules gets one row for each rule.
Report a line only when you would ask the author to change it.
