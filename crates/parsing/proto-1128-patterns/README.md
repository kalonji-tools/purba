# PROTOTYPE: the shapes of an expectation as tree patterns

Throwaway, for purba issue 1128, "How do the shapes of an expectation look as
tree patterns?". This branch, `proto/1128-patterns`, is never merged.

## Run it

```sh
# from the root of the clone, with a purba wheel installed in target/wheel-venv
bash crates/parsing/proto-1128-patterns/compare.sh "$PWD/target/wheel-venv/bin/purba" \
  $(git ls-files 'tests/*.py')
bash crates/parsing/proto-1128-patterns/compare.sh "$PWD/target/wheel-venv/bin/purba" \
  crates/parsing/proto-1128-patterns/probes/*.py
```

`probes/` is split from `probes.txt` by
`awk '/^### /{f="probes/" $2; next} {print > f}' probes.txt`.

## What stands for what

| Rust on `94611a47`                     | rule                         |
|----------------------------------------|------------------------------|
| selection, a `def test…` at the top    | `utils/test-def.yml`         |
| parsing `ending`                       | `utils/ending.yml`, `utils/at-the-end.yml` |
| parsing `check`, `Check::Compare`      | `utils/check.yml`, `utils/compare.yml` |
| parsing `read`, `Reads`                | `utils/read.yml`             |
| parsing `each`, `Shape::Each`          | `utils/each.yml`, `utils/all-or-any.yml`, `utils/one-generator.yml` |
| parsing `is_literal`                   | `utils/literal.yml`          |
| parsing `blocks`                       | none: `stopBy: end` walks every block, and the bodies of a `def` and a `class` too |
| selection `[not-one-comparison]`       | `rules/not-one-comparison.yml` |
| selection `[computed-expected]`        | `rules/computed-expected.yml` |

The rules hold 220 lines that are not a comment or blank. The Rust of
L473-L728 holds 200.

## Results

The 79 modules under `tests/`: 14 findings in both, none in one alone.

The first run found 22 in the patterns alone: a negative `has` sees `[`, `,`
and `]`, so no list was a literal. `utils/literal.yml` now lets each of them
through by its text. That run is the control: the comparison sees a wrong rule.

The 21 probes under `probes/`: 16 in both, 7 in one alone.

| probe | patterns | purba | why |
|---|---|---|---|
| `test_syntax_error.py` | computed-expected | `SyntaxError`, 0 cases | tree-sitter recovers `def test_sum(:` with no error node |
| `test_comment_in_list.py` | computed-expected | passed | a comment is an item of tree-sitter's list |
| `test_comment_in_comparison.py` | not-one-comparison | passed | a comment is a third operand of tree-sitter's comparison |
| `test_backslash_in_comparison.py` | not-one-comparison | passed | a `line_continuation` is a third operand |
| `test_double_parens_generator.py` | not-one-comparison | passed | tree-sitter keeps `(…)` around the generator; ruff does not |
| `test_class.py` | nothing | computed-expected | the rule names no method a test; selection does |
| `test_bound_twice.py` | computed-expected | name-bound-again | selection judges the name, not each `def` |
