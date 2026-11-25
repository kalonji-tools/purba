# How purba finds the root

The root is the folder that each test's id starts from. `purba` with no path runs every test file under it, and a path you name starts there.

## `[tool.purba]`

purba walks up from the working folder to the first `pyproject.toml` that holds `[tool.purba]`. The folder of that file is the root. An empty table is enough:

```toml
[tool.purba]
```

The walk stops at the folder that holds `.git`. purba prints no line about a root it found this way.

## No `[tool.purba]`

When the walk finds no such file, purba refuses the run, and exits 3. The error names where to add the table: the `pyproject.toml` with `[project]` that the walk met, or else the folder that holds `.git`, or else the working folder.

```text
purba: invocation error: no pyproject.toml holds [tool.purba]; add it to pyproject.toml
purba: invocation error: no pyproject.toml holds [tool.purba]; add one with it to .
```

The path in the error starts at the working folder. From `tests/`, the first error names `../pyproject.toml`.

## `PURBA_IMPLICIT_ROOT`

Set `PURBA_IMPLICIT_ROOT=1` to run without `[tool.purba]`. Unset, empty or `0` gives the refusal above. Any other value is refused, and purba exits 3:

```text
purba: invocation error: PURBA_IMPLICIT_ROOT is yes; set 1 or 0
```

With `1`, purba takes the first of these steps that applies, and the run goes on as with `[tool.purba]`:

1. The folder of the nearest `pyproject.toml` with `[project]` that the walk met.
2. Else, the folder that holds `.git`.
3. Else, the working folder.

The first line of the output names the root and the step. `~` is your home folder. The line changes no count and no exit code.

Step 1, from `~/with-project` or from `~/with-project/tests`:

```text
⚠ root ~/with-project is the folder of the pyproject.toml that holds [project], because PURBA_IMPLICIT_ROOT is set and no pyproject.toml holds [tool.purba]
tests/test_a.py::test_add passed
1 case: 1 passed
```

Step 2, from `~/only-git` or from `~/only-git/tests`:

```text
⚠ root ~/only-git is the folder that holds .git, because PURBA_IMPLICIT_ROOT is set and no pyproject.toml holds [tool.purba]
tests/test_a.py::test_add passed
1 case: 1 passed
```

Step 3, from `~/neither`:

```text
⚠ root ~/neither is the working folder, because PURBA_IMPLICIT_ROOT is set and no pyproject.toml holds [tool.purba]; the id of each test starts here, and moves with the working folder
tests/test_a.py::test_add passed
1 case: 1 passed
```

Step 3, from `~/neither/tests`. The root moves with the working folder, and so does each id:

```text
⚠ root ~/neither/tests is the working folder, because PURBA_IMPLICIT_ROOT is set and no pyproject.toml holds [tool.purba]; the id of each test starts here, and moves with the working folder
test_a.py::test_add passed
1 case: 1 passed
```
