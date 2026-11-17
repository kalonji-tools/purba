# Ways a module gets imported, and means to refuse one

Python code can reach a module in far more ways than the `import` statement
shows, and no single means sees all of them. A test module can import
`pytest` by a statement, by a call that takes the name as data (the probes
cover fifteen standard callables, and any library can add more), by running
one of pytest's files under another name, through a dependency, or by binding a
module that the environment loaded before the test. Reading the source sees
only what the text spells out. A hook inside the interpreter sees what passes
through that hook, and each hook has its own gaps: `importlib.import_module`
passes through neither `builtins.__import__` nor the `import` audit event, and
of the hooks only a replaced `builtins.__import__` sees an import statement
for a module already in `sys.modules`. Only a watch on the files that the
process reads flagged every way that ran pytest's code in the probes: an `open`
audit hook inside the test's process, and `strace -f` outside it, which also
saw the child process. Neither sees a module that was loaded before the test.
Each means also costs something different: false refusals, a dependency that
breaks, a silent refusal that a `try`/`except ImportError` swallows, or a
change to the interpreter's state that the test can undo. The rest of this
file lists the ways, the means, and a matrix of which means caught which way
on CPython 3.12.14, 3.13.13 and 3.14.4.

## Scope and evidence

The target is the name `pytest` or `_pytest`, or any submodule of either.
pytest's own `pytest/__init__.py` imports `_pytest` first, so a means that
watches only one of the two names often still catches the other one.

Each fact is marked one of two ways:

- **probed**: a script in [`unwanted-import/`](unwanted-import/) shows it. The
  results are committed beside the scripts.
- **sourced**: a document or a source file says it, and no probe here shows it.

The probes ran against purba built from `main` at `1dbae44`, CPython 3.12.14
with pytest 9.2.0 in one virtualenv, and CPython 3.13.13 and 3.14.4 reading
that same `site-packages` through `PYTHONPATH`.

| file | what it shows |
|---|---|
| [`ways.py`](unwanted-import/ways.py) | each way, as a test module and its helper modules |
| [`child.py`](unwanted-import/child.py) | loads one way as purba does (compile, then exec in a new module) under one in-process means |
| [`matrix.py`](unwanted-import/matrix.py) | every way under every means; the static means and the out-of-process means live here |
| [`results/3.12.14.md`](unwanted-import/results/3.12.14.md), [`3.13.13.md`](unwanted-import/results/3.13.13.md), [`3.14.4.md`](unwanted-import/results/3.14.4.md) | the matrix for each version, and what each way left behind with no means |
| [`results/*.json`](unwanted-import/results/) | the raw hits, evidence and errors of each cell |
| [`facts.py`](unwanted-import/facts.py), [`results/facts.txt`](unwanted-import/results/facts.txt) | ten single facts about the import machinery, on all three versions |
| [`purba_process.py`](unwanted-import/purba_process.py), [`results/purba-process.md`](unwanted-import/results/purba-process.md) | what the process holds when purba loads a test module |
| [`cost.py`](unwanted-import/cost.py), [`results/cost-3.12.14.txt`](unwanted-import/results/cost-3.12.14.txt) | the time each means adds |

To run them again:

```console
$ v/bin/python research/unwanted-import/matrix.py v/bin/python v/lib/python3.12/site-packages <work dir> research/unwanted-import/results
$ v/bin/python research/unwanted-import/facts.py
$ v/bin/python research/unwanted-import/purba_process.py v/bin/purba <work dir>
$ v/bin/python research/unwanted-import/cost.py v/bin/python v/lib/python3.12/site-packages <work dir>
```

### What purba's process looks like today

These facts bound what a means can rely on inside purba.

- purba reads the test module's text with ruff's parser, and selection already
  maps each `import` and `from ... import` to the module it names
  ([`origins`](../crates/selection/src/lib.rs) in `crates/selection/src/lib.rs`).
  **sourced**
- purba loads a test module by compiling its text and running it with `exec`
  in a new module object that it puts in `sys.modules` under the file's stem
  ([`load`](../crates/assertion/src/lib.rs) in `crates/assertion/src/lib.rs`).
  The test module therefore has no `__package__` and no `__spec__`, and
  `from . import x` raises `ImportError: attempted relative import with no
  known parent package`. **probed** (`purba-process.md`, `relative`)
- `sys.path[0]` is the directory of the `purba` script (`v/bin`), not the test
  module's directory. A module next to the test module is not importable, so a
  helper reaches a test only through an installed package, `PYTHONPATH` or a
  `.pth` file. **probed** (`purba-process.md`, `sibling`)
- At load, the process holds 61 modules, none of them `pytest` or `_pytest`.
  `sys.meta_path` holds the three default finders, and `__import__` is the
  built-in. **probed** (`purba-process.md`, `state`)
- A `sitecustomize` module on `PYTHONPATH` that imports pytest puts pytest and
  `_pytest` in `sys.modules` before purba loads the test module (292 modules at
  load). **probed** (`purba-process.md`, `preloaded`)
- Each test runs in its own forked process. An import in one test's body is not
  in `sys.modules` in the next test's process. **probed** (`per_test`)
- While a test runs, purba holds `sys.monitoring` tool 3 (or 4 if 3 is taken),
  with local events on the test module's code objects only
  ([`tool`](../crates/execution/src/paths.rs) in
  `crates/execution/src/paths.rs`). **probed** (`per_test`) and **sourced**
- purba today runs a test module that imports pytest, and the test passes.
  **probed** (`purba-process.md`, `imports`)

## Part 1: the ways

The ways fall into six groups by what the test module's text shows. The
identifier of each way is the name of its probe in `ways.py`. A way marked
"no-import" names the target but imports nothing; it is there to show the
false refusals of a means.

### Group A: the import statement names the target

The language reference defines the statement and its forms
([the import statement](https://docs.python.org/3.12/reference/simple_stmts.html#the-import-statement)).
Each form calls `__import__` and binds a name.

| way | example | does | evidence |
|---|---|---|---|
| `import-stmt` | `import pytest` | imports | probed |
| `import-as` | `import pytest as pt` | imports | probed |
| `from-import` | `from pytest import raises` binds `raises`, not the module | imports | probed |
| `star-import` | `from pytest import *` | imports | probed |
| `private-submodule` | `import _pytest.outcomes` imports `_pytest` first, then binds `_pytest` | imports | probed |
| `from-private` | `from _pytest.outcomes import fail` | imports | probed |
| `in-test-body` | the statement inside a test function runs at call time, not at load | imports | probed |
| `under-type-checking` | `if TYPE_CHECKING: import pytest` never runs, since [`typing.TYPE_CHECKING`](https://docs.python.org/3.12/library/typing.html#typing.TYPE_CHECKING) is false at run time | no-import | probed |
| `try-optional` | `try: import pytest` / `except ImportError: pytest = None` | imports | probed |
| `relative-from` | `from . import pytest` resolves against `__package__` ([package relative imports](https://docs.python.org/3.12/reference/import.html#package-relative-imports)); under purba it raises `ImportError` | no-import | probed |

Python 3.15 adds `lazy import pytest` and `lazy from pytest import raises`
([PEP 810](https://peps.python.org/pep-0810/#abstract)). The module then loads
when the name is first used, and reification "uses the state of the import
system ... at reification time"
([PEP 810, reification](https://peps.python.org/pep-0810/#reification)). A
module-level `__lazy_modules__` list, `-X lazy_imports` and
`sys.set_lazy_imports()` make plain statements lazy too. Outside the 3.12 to
3.14 range of this file. **sourced**

### Group B: a call imports a module whose name it takes as data

Each of these imports by a string. Where the string is a literal, the text
shows the name; where it is computed, the text does not.

| way | example | how it imports | evidence |
|---|---|---|---|
| `dunder-import-literal` | `__import__("pytest")` | [`__import__`](https://docs.python.org/3.12/library/functions.html#import__) | probed |
| `dunder-import-computed` | `__import__("py" + "test")` | same | probed |
| `import-module` | `importlib.import_module("pytest")` | calls `_bootstrap._gcd_import` directly ([`importlib/__init__.py` L90](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/importlib/__init__.py#L90)), never `builtins.__import__` | probed |
| `exec-string` | `exec("import pytest")` | [`exec`](https://docs.python.org/3.12/library/functions.html#exec) runs a statement from a string | probed |
| `eval-string` | `eval("__import__('pytest')")` | [`eval`](https://docs.python.org/3.12/library/functions.html#eval) | probed |
| `find-spec-submodule` | `importlib.util.find_spec("_pytest.outcomes")` | "If name is for a submodule (contains a dot), the parent module is automatically imported" ([`find_spec`](https://docs.python.org/3.12/library/importlib.html#importlib.util.find_spec)) | probed |
| `find-spec-top` | `importlib.util.find_spec("pytest")` asks the finders but runs no module | no-import | probed |
| `resolve-name` | `pkgutil.resolve_name("pytest:raises")` | `importlib.import_module` ([`pkgutil.py` L505](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/pkgutil.py#L505), [docs](https://docs.python.org/3.12/library/pkgutil.html#pkgutil.resolve_name)) | probed |
| `mock-patch-target` | `mock.patch("pytest.raises")` | "The target is imported when the decorated function is executed" ([`patch`](https://docs.python.org/3.12/library/unittest.mock.html#unittest.mock.patch)), through `pkgutil.resolve_name` ([`mock.py` L1647](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/unittest/mock.py#L1647)) | probed |
| `pickle-global` | `pickle.loads(b"cpytest\nraises\n.")` | the unpickler imports the module that a global names ([`find_class`](https://docs.python.org/3.12/library/pickle.html#pickle.Unpickler.find_class)), through `PyImport_Import` ([`_pickle.c` L7128](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Modules/_pickle.c#L7128)) | probed |
| `resources-files` | `importlib.resources.files("pytest")` | `importlib.import_module` on a string anchor ([`_common.py` L82](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/importlib/resources/_common.py#L82)) | probed |
| `entry-point-load` | `EntryPoint.load()` of pytest's `console_scripts` entry | `import_module` of the entry's value ([`metadata/__init__.py` L205](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/importlib/metadata/__init__.py#L205)). pytest 9.2.0's value is `_pytest.config:_console_main`, so the module imported is never `pytest` | probed |
| `runpy-submodule` | `runpy.run_module("_pytest.outcomes")` | imports the parent package first ([`runpy.py` L112](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/runpy.py#L112)) | probed |
| `c-api-import-module` | `PyImport_ImportModule(b"pytest")` through `ctypes`, as an extension module would call it | calls `PyImport_Import`, which calls the `__import__` of the current globals' builtins ([`import.c` L3014](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Python/import.c#L3014)) | probed |
| `c-api-import-level` | `PyImport_ImportModuleLevel(b"pytest", ...)` through `ctypes` | goes straight to the import machinery, not through `__import__` ([`import.c` L2818](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Python/import.c#L2818)) | probed |
| `metadata-version` | `importlib.metadata.version("pytest")` reads the distribution's metadata | no-import | probed |
| `walk-packages` | `pkgutil.walk_packages([site])` | "this function must import all *packages* ... on the given *path*" ([`walk_packages`](https://docs.python.org/3.12/library/pkgutil.html#pkgutil.walk_packages)) | probed |
| `logging-config-factory` | `logging.config.dictConfig` with `"()": "pytest.MonkeyPatch"` | [user-defined objects](https://docs.python.org/3.12/library/logging.config.html#user-defined-objects) resolve through `__import__` ([`config.py` L404](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/logging/config.py#L404)) | probed |

The list is open. Any library can call `importlib.import_module` with a name it
read from data, as the four standard modules above do. Two more standard ones
read the name from the environment rather than from the test:
[`breakpoint()`](https://docs.python.org/3.12/library/functions.html#breakpoint)
imports the hook that `PYTHONBREAKPOINT` names, and a `-W` option's category
is imported by its module name
([`warnings.py` L263](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/warnings.py#L263)).
**sourced**

`importlib.reload(module)` runs a loaded module's code again
([`reload`](https://docs.python.org/3.12/library/importlib.html#importlib.reload)),
and [`importlib.util.LazyLoader`](https://docs.python.org/3.12/library/importlib.html#importlib.util.LazyLoader)
defers the run of a module's code to its first attribute access. **sourced**

### Group C: code runs a file of the target without importing its name

| way | example | evidence |
|---|---|---|
| `spec-from-file-package` | `spec_from_file_location("not_the_target", ".../pytest/__init__.py")`, then `exec_module`. The module runs as `not_the_target`, and its own statements then import `_pytest` by name | probed |
| `spec-from-file-leaf` | the same on `_pytest/_version.py`, which imports nothing. No module named `pytest` or `_pytest` exists afterwards | probed |
| `exec-file-text` | `exec(compile(open(path).read(), "<a copy>", "exec"), ns)` on `_pytest/_version.py`. No module object exists, and the code's file name is not pytest's | probed |

[Importing a source file directly](https://docs.python.org/3.12/library/importlib.html#importing-a-source-file-directly)
is a documented recipe. `facts.py` shows that it asks no meta path finder and
puts nothing in `sys.modules`. **probed** (`facts.txt`). Copying pytest's code
into the test's own tree is the limit of this group; nothing then ties the code
to pytest.

### Group D: the test's text does not show the target

| way | example | does | evidence |
|---|---|---|---|
| `transitive-stmt` | `import helper`, where `helper` runs `import pytest` | imports | probed |
| `transitive-dynamic-two-hops` | `helper_chain` imports `helper_dyn`, which calls `import_module("py" + "test")` | imports | probed |
| `star-reexport` | `from helper import *` binds `pytest` when `helper` has no `__all__` and imported it ([the import statement](https://docs.python.org/3.12/reference/simple_stmts.html#the-import-statement): without `__all__`, every name not starting with `_`) | imports | probed |
| `lazy-module-getattr` | `helper.thing` calls the module's `__getattr__` ([PEP 562](https://peps.python.org/pep-0562/)), which imports pytest when the test runs | imports | probed |
| `dependency-call` | a test calls `helper.make()`, whose body imports pytest | imports | probed |
| `preloaded-import-stmt` | `import pytest`, with pytest already in `sys.modules` before the test loads (a `.pth` file, `sitecustomize`, a dependency) | binds | probed |
| `preloaded-sys-modules` | `sys.modules["py" + "test"]`, with pytest already loaded: a binding with no import at all | binds | probed |
| `shadow-same-name` | a file `pytest.py` earlier on `sys.path` than pytest; `import pytest` loads it | a module named `pytest` that is not pytest | probed |
| `atexit-import` | `atexit.register(__import__, "pytest")`: the import runs after the last test | imports at exit | probed |
| `other-process` | `subprocess.run([sys.executable, "-c", "import pytest"])` | imports in a child process only | probed |
| `annotation-expression` | `def f(x: __import__("pytest").MonkeyPatch)` | imports at `def` on 3.12 and 3.13; imports nothing on 3.14, where annotations are lazy ([3.14 annotations](https://docs.python.org/3.14/reference/compound_stmts.html#annotations), [PEP 649](https://peps.python.org/pep-0649/)) | probed |
| `annotation-read` | the same, then `f.__annotations__`, which evaluates it on 3.14 | imports | probed |
| `future-annotations-name` | `from __future__ import annotations` and `x: pytest.MonkeyPatch` with no import: the annotation is a string | no-import | probed |

The environment can load the target before any test code runs:
[a `.pth` file](https://docs.python.org/3.12/library/site.html) runs each line
that starts with `import` ([`site.py` L205](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/site.py#L205)),
and `site` imports [`sitecustomize`](https://docs.python.org/3.12/library/site.html#module-sitecustomize)
and `usercustomize`. **sourced**; the `sitecustomize` case under purba is
**probed** (`preloaded`). A process that
[`multiprocessing`](https://docs.python.org/3.12/library/multiprocessing.html#contexts-and-start-methods)
starts with `spawn` or `forkserver` imports modules again in the child.
**sourced**

### Group E: the test takes apart the means

`undo-then-import` deletes each `None` from `sys.modules`, removes each meta
path finder that is not one of the default ones, sets `builtins.__import__` to
`importlib.__import__`, turns off the events of every `sys.monitoring` tool,
imports pytest through `import_module("py" + "test")`, then deletes every
`pytest` and `_pytest` key from `sys.modules` while it keeps a reference.
**probed**

Two facts make this possible. `builtins.__import__` "can be replaced ... in
order to change semantics of the import statement"
([`__import__`](https://docs.python.org/3.12/library/functions.html#import__)),
and `sys.modules` "is writable"
([the module cache](https://docs.python.org/3.12/reference/import.html#the-module-cache)).
An audit hook cannot be removed from Python
([`sys.addaudithook`](https://docs.python.org/3.12/library/sys.html#sys.addaudithook),
probed in `facts.txt`), but the same docs say that hooks "are not suitable for
implementing a sandbox" and that "malicious code can trivially disable or
bypass hooks". **sourced**

## Part 2: the means

Each means is named for what it does, with the code it adds. "Catches" and
"misses" name the ways of part 1 and come from the matrix below unless marked
otherwise.

### Reading the source before it runs

**`stmt-scan`: walk the syntax tree for `import` and `from ... import` of the
target.** This is the shape of selection's existing name resolution.

```python
for node in ast.walk(tree):
    if isinstance(node, ast.Import): ...      # alias.name starts with pytest/_pytest
    elif isinstance(node, ast.ImportFrom) and node.level == 0: ...
```

- Catches: all of group A that runs, at any depth, inside functions and `try`
  blocks; `preloaded-import-stmt`, since it reads text and not state.
- Misses: every way of groups B to E that imports or binds the target, except
  `preloaded-import-stmt`.
- False refusals: `under-type-checking`.
- Cost: no change to the interpreter. The probes use Python's `ast`, which has
  the same `Import` and `ImportFrom` nodes, with `level`, as ruff's AST that
  purba reads. **probed** with `ast`; ruff **sourced** from purba's code.

**`known-call-scan`: `stmt-scan`, plus calls of a fixed list of importing
functions whose first argument is a literal naming the target.** The probe's
list is `__import__`, `import_module`, `find_spec`, `resolve_name`,
`run_module`, `files` and `patch`, matched on the called name alone.

- Catches beyond `stmt-scan`: `dunder-import-literal`, `import-module`,
  `find-spec-submodule`, `resolve-name`, `mock-patch-target`,
  `resources-files`, `runpy-submodule`, `annotation-expression`,
  `annotation-read`.
- Misses: any computed name; any importing call not on the list
  (`pickle-global`, `entry-point-load`, `logging-config-factory`,
  `walk-packages`, both C-API ways, `exec-string`, `eval-string`); groups C to
  E.
- False refusals: `find-spec-top`, `under-type-checking`, and
  `annotation-expression` on 3.14.
- Cost: the list never closes, since any library may import by a string.

**`name-mention-scan`: refuse any identifier, alias or string literal whose
text is the target's name.**

- Catches beyond `known-call-scan`: `exec-string`, `eval-string`, both C-API
  ways, `entry-point-load` (its `name="pytest"` keyword), group C (the paths
  name `pytest/` or `_pytest/`), `atexit-import`, `undo-then-import` (it names
  `_pytest` in its cleanup).
- Misses: `dunder-import-computed`, `pickle-global` (the opcode `c` runs into
  the name, `cpytest`),
  `walk-packages`, the transitive ways of group D, `preloaded-sys-modules`.
- False refusals: `relative-from`, `find-spec-top`, `metadata-version`,
  `future-annotations-name`, `under-type-checking`; `other-process` counts
  here as "seen".

**`modulefinder`: follow import statements through every module the test
reaches, without running them.** The standard
[`modulefinder`](https://docs.python.org/3.12/library/modulefinder.html) scans
each code object's bytecode for imports
([`modulefinder.py` L387](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/modulefinder.py#L387))
and finds modules with `PathFinder` alone
([L54](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/modulefinder.py#L54)).

- Catches beyond `stmt-scan`: `transitive-stmt`, `star-reexport`,
  `lazy-module-getattr`, `dependency-call` (it reads the statement inside the
  helper's function, whether or not it runs).
- Misses: every call of group B, group C, `transitive-dynamic-two-hops`,
  `preloaded-sys-modules`, `atexit-import`, `undo-then-import`.
- False refusals: `under-type-checking`. It raised `ImportError: relative
  importpath too deep` on `relative-from`.
- Cost: 0.9 s and 321 modules walked for a test module that imports only
  `json`; 1.3 s and 450 modules when it imports pytest. **probed**
  (`cost-3.12.14.txt`). A finder installed on `sys.meta_path` (an editable
  install's, for one) is not consulted, since it uses `PathFinder` only.
  **sourced**

### Watching or refusing at run time, in the same process

Every means here changes the interpreter of the test, and test code can see
and change it (group E).

**`modules-diff`: compare the keys of `sys.modules` before the load with
those after the load and after each test.**

```python
before = set(sys.modules)
... load, call ...
new = {k for k in set(sys.modules) - before if k.split(".")[0] in TARGET}
```

- Catches: every way that leaves a target name in `sys.modules` during the load
  or a call, whatever the route: all of A and B that import,
  `spec-from-file-package` (through `_pytest`), the transitive ways of D,
  `annotation-read`, and `annotation-expression` on 3.12 and 3.13.
- Misses: `spec-from-file-leaf`, `exec-file-text`, both `preloaded-*` ways (the
  name was there before), `atexit-import` (after the last check),
  `undo-then-import` (it deleted the keys).
- False refusals: `shadow-same-name` (a name, not pytest).
- Cost: reads only; it detects and does not prevent, so the imported code has
  already run.

**`audit-open`: an audit hook on the `open` event for paths inside the
target's directories.**

```python
sys.addaudithook(lambda e, a: e == "open" and str(a[0]).startswith(DIRS) and hits.append(a[0]))
```

The import system reads each file through `_io.open_code`
([`_bootstrap_external.py` L1190](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/importlib/_bootstrap_external.py#L1190)),
and the probe saw the event for each `.pyc` that it read.

- Catches: everything that reads a target file in this process: all importing
  ways of A to D, group C including `exec-file-text`, `atexit-import` (the hook
  outlives the tests), `undo-then-import`.
- Misses: both `preloaded-*` ways (no file is read again), `other-process`.
- Passes: `shadow-same-name` (its file is elsewhere).
- Cost: about 1 ms on an import of pytest (`cost-3.12.14.txt`). The hook stays
  for the life of the interpreter. Any read of a target file counts, so a
  traceback or `inspect.getsource` that reads one would count too. **sourced**,
  not probed.

**`monitoring-code`: a `sys.monitoring` tool whose `PY_START` callback records
code objects from the target's files, and returns `DISABLE` for every code
object, so each one costs one call.**

- Catches: each way that runs a target code object whose file name is a target
  file.
- Misses: `exec-file-text` (its file name is `<a copy>`), both `preloaded-*`
  ways (binding runs no code; calling a pytest function would run code and be
  seen), `atexit-import` (the probe turned the events off after the tests),
  `undo-then-import` (it turned the events off).
- Cost: about 4 ms on an import of pytest. It needs a tool id: ids run "0 to 5"
  ([`use_tool_id`](https://docs.python.org/3.12/library/sys.monitoring.html#sys.monitoring.use_tool_id)),
  purba takes 3 or 4 during a test, and 0, 1, 2 and 5 are named for a
  debugger, coverage, a profiler and an optimizer
  ([tool identifiers](https://docs.python.org/3.12/library/sys.monitoring.html#tool-identifiers)).
  **sourced**

**`modules-none`: set `sys.modules["pytest"]` and `sys.modules["_pytest"]` to
`None` before the load.** "The key can also be assigned to `None`, forcing the
next import of the module to result in a `ModuleNotFoundError`"
([the module cache](https://docs.python.org/3.12/reference/import.html#the-module-cache)).
A submodule import fails too, since its parent does.

- Refuses: every import by name through the machinery, `import_module` and the
  C API included; `preloaded-import-stmt` (the entry is replaced).
- Refuses late: `spec-from-file-package`; the first lines of
  `pytest/__init__.py` ran before `from _pytest import ...` failed.
- Silent: `try-optional` and `walk-packages` swallow the error, and the means
  records nothing, so the tool does not learn that the test tried;
  `atexit-import` fails after the tests, where no test owns the error;
  `preloaded-sys-modules` gets `None` instead of the module.
- Misses: `spec-from-file-leaf`, `exec-file-text`, `undo-then-import`.
- Side effects: `importlib.util.find_spec("pytest")` returns `None`, as if pytest
  were not installed ([`util.py` L103](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/importlib/util.py#L103),
  probed in `facts.txt`). `import_module("_pytest.config")` failed with
  `'_pytest' is not a package` rather than the `halted` message. A dependency
  that needs pytest at import fails; one that catches the error behaves as if
  pytest were absent.

**`meta-path-finder`: a finder first on `sys.meta_path` that records and
raises `ImportError` for a target name.** Finders are asked only when the name
is not in `sys.modules`
([the meta path](https://docs.python.org/3.12/reference/import.html#the-meta-path),
[`_bootstrap.py` L1255](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/importlib/_bootstrap.py#L1255);
probed in `facts.txt`).

- Refuses and records: every import by name of a module not yet loaded,
  through every route of groups A, B and D, with the name as asked; this
  includes `import-module`, `pickle-global` and both C-API ways.
- Refuses late: `spec-from-file-package`.
- Misses: both `preloaded-*` ways, `spec-from-file-leaf`, `exec-file-text`,
  `undo-then-import` (it removed the finder).
- False refusals: `find-spec-top`; `find_spec` raises though it would run
  nothing.
- Side effects: a new entry on `sys.meta_path`, which purba and pytest leave as
  it is. A `try`/`except ImportError` in a dependency hides the refusal from
  the dependency's caller, but the finder has already recorded it.

**`builtins-import`: replace `builtins.__import__` with a function that records
and refuses a target name, after it resolves a relative name against the
importer's `__package__`.** The import statement calls the frame's builtins'
`__import__` and takes a fast path only when it is the default
([`ceval.c` L2489](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Python/ceval.c#L2489)).

- Refuses: each import statement and `__import__` call, even for a module
  already loaded, so it refuses `preloaded-import-stmt`; library code that
  calls `__import__` (`walk-packages`, `runpy-submodule`,
  `logging-config-factory`); `PyImport_Import` callers (`pickle-global`,
  `c-api-import-module`).
- A library that saved `__import__` before the replacement keeps the original:
  `logging.config` stores it as a class attribute when it is first imported
  ([`config.py` L390](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/logging/config.py#L390)).
  The probe imported `logging.config` after the replacement. **sourced**
- Never sees the module that `importlib.import_module`,
  `importlib.__import__` or `PyImport_ImportModuleLevel` loads by name
  ([`_bootstrap.py` L1375](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Lib/importlib/_bootstrap.py#L1375);
  probed in `facts.txt`). In the matrix it still refused `import-module`,
  `resolve-name`, `mock-patch-target`, `resources-files`,
  `transitive-dynamic-two-hops` and `c-api-import-level`, but the hit was
  `_pytest`, which `pytest/__init__.py` then imported by a statement. A target
  whose code imports nothing by statement would pass.
- Refuses late: `entry-point-load`. `import_module` loaded `_pytest` unseen.
  Its first refusal, of `_pytest._version`, was swallowed by the
  `try`/`except ImportError` in `_pytest/__init__.py`; the second, of
  `_pytest.config.exceptions`, stopped the load.
- Misses: `preloaded-sys-modules`, `spec-from-file-leaf`, `exec-file-text`,
  `undo-then-import`.
- Side effects: every import in the process now calls a Python function. The
  docs call the replacement "**strongly** discouraged"
  ([`__import__`](https://docs.python.org/3.12/library/functions.html#import__)).

**`audit-import`: an audit hook that records and raises on the `import` event
for a target name.** The import statement raises the event
([the import statement](https://docs.python.org/3.12/reference/simple_stmts.html#the-import-statement)),
from `import_find_and_load`, which runs only for a module missing from
`sys.modules` or set to `None`
([`import.c` L2757](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Python/import.c#L2757),
[L2869](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Python/import.c#L2869)).

- Refuses: the import statement, `__import__`, the C API including
  `PyImport_ImportModuleLevel`, pickle, transitive imports.
- Never sees the module that `importlib.import_module` loads by name (probed in
  `facts.txt`): as with `builtins-import`, the matrix's hit for the
  `import_module` routes was `_pytest`.
- Refuses late: `entry-point-load`. Its first refusal, of `_pytest._version`,
  was swallowed by pytest's own `try`/`except ImportError` in
  `_pytest/__init__.py`.
- Misses: both `preloaded-*` ways, `spec-from-file-leaf`, `exec-file-text`,
  `undo-then-import` (`importlib.__import__` in place of the built-in never
  reaches the C path, so no event fires).
- Side effects: the hook cannot be removed and runs for every audit event of
  the interpreter's life. pickle also raises its own event,
  `pickle.find_class` ([`_pickle.c` L7064](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Modules/_pickle.c#L7064)).
  **sourced**

**`path-without-target`: load the test in an environment where pytest is not
importable.** The probe replaces `site-packages` on `sys.path` with a copy that
lacks `pytest`, `_pytest`, `py.py` and pytest's `dist-info`.

- Refuses: every import by name, through every route, including
  `undo-then-import` (it does not touch `sys.path`; a test that adds the path
  back would undo this means too, **sourced** reasoning, not probed).
- Refuses late: `spec-from-file-package`.
- Silent: `try-optional`, `walk-packages`, `atexit-import`.
- Misses: `spec-from-file-leaf`, `exec-file-text` (the files are still on disk at
  a path the test can compute); both `preloaded-*` ways, because the probe
  loaded pytest before it changed the path. In an environment that has no
  pytest at all, nothing can load it first. **sourced** reasoning.
- False refusals: `metadata-version`, since the metadata is gone too.
- Side effects: a dependency that imports pytest fails to import. The 25 ms
  that `cost.py` shows comes from the probe's copy of `site-packages`, not from
  the means; a real environment was not measured.

### Watching from another process

**`importtime`: run the load under `-X importtime` and read the module names
it prints to stderr after a mark.** The interpreter prints a line from
`import_find_and_load` ([`import.c` L2779](https://github.com/python/cpython/blob/2abcf904b8dac8c999d2b3aac76681abb333798a/Python/import.c#L2779);
[`-X importtime`](https://docs.python.org/3.12/using/cmdline.html#cmdoption-X)).

- Catches: the same routes as `audit-import`, as names only; the top name of
  an `import_module` route gets no line (probed in `facts.txt`), only the
  modules that it then imports by statement.
- Misses: both `preloaded-*` ways, `spec-from-file-leaf`, `exec-file-text`,
  `undo-then-import`.
- Cost: under 1 ms. The docs warn that "its output may be broken in
  multi-threaded application".

**`strace-open`: trace the process's `openat` calls and match paths in the
target's directories after a mark.**

- Catches: every way that reads a target file after the mark, in this process
  or a child (`strace -f`): all importing ways, group C, `atexit-import`,
  `undo-then-import`, `other-process`.
- Misses: both `preloaded-*` ways: the file was read before the mark.
- Passes: `shadow-same-name`.
- Cost: about 10 ms on a load that imports `json`, and 40 ms on one that
  imports pytest. It needs `ptrace` (Linux); the test's interpreter is
  unchanged. A tracer attached to a process forked after the zygote's load
  sees none of the load's reads. **sourced** reasoning from purba's zygote
  ([`zygote.rs`](../crates/execution/src/zygote.rs)), not probed.

## The matrix

CPython 3.12.14, from [`results/3.12.14.md`](unwanted-import/results/3.12.14.md).
CPython 3.13.13 gives the same matrix. CPython 3.14.4 differs in one row,
`annotation-expression`, which imports nothing there: the runtime means pass
it, and `known-call-scan` and `name-mention-scan` refuse it falsely.

Columns, left to right: the four static means (`stmt`, `call`, `mention`,
`mfind`), three in-process watches (`diff`, `open`, `mon`), five in-process
refusals (`none`, `meta`, `bimp`, `aimp`, `path`), two watches from another
process (`itime`, `strace`).

Cells: `catch` the means flagged a way that imports; `refuse` it stopped one
before any of pytest's code ran and recorded it, or the error reached the tool;
`late` it stopped one after some of pytest's code ran; `silent` it stopped one
but nothing reached the tool; **MISS** pytest's code ran or its module was
bound and the means did not flag it; **FALSE** it flagged a way that imports
nothing; `pass` it did not flag a way that imports nothing; `name` it flagged a
module that only has pytest's name; `crash` the means itself failed; `seen` and
`-` for the way that imports in a child process.

| way | does | stmt | call | mention | mfind | diff | open | mon | none | meta | bimp | aimp | path | itime | strace |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| A `import-stmt` | imports | catch | catch | catch | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| A `import-as` | imports | catch | catch | catch | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| A `from-import` | imports | catch | catch | catch | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| A `star-import` | imports | catch | catch | catch | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| A `private-submodule` | imports | catch | catch | catch | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| A `from-private` | imports | catch | catch | catch | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| A `in-test-body` | imports | catch | catch | catch | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| A `under-type-checking` | no-import | **FALSE** | **FALSE** | **FALSE** | **FALSE** | pass | pass | pass | pass | pass | pass | pass | pass | pass | pass |
| A `try-optional` | imports | catch | catch | catch | catch | catch | catch | catch | silent | refuse | refuse | refuse | silent | catch | catch |
| A `relative-from` | no-import | pass | pass | **FALSE** | crash | pass | pass | pass | pass | pass | pass | pass | pass | pass | pass |
| B `dunder-import-literal` | imports | **MISS** | catch | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `dunder-import-computed` | imports | **MISS** | **MISS** | **MISS** | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `import-module` | imports | **MISS** | catch | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `exec-string` | imports | **MISS** | **MISS** | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `eval-string` | imports | **MISS** | **MISS** | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `find-spec-submodule` | imports | **MISS** | catch | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `find-spec-top` | no-import | pass | **FALSE** | **FALSE** | pass | pass | pass | pass | pass | **FALSE** | pass | pass | pass | pass | pass |
| B `resolve-name` | imports | **MISS** | catch | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `mock-patch-target` | imports | **MISS** | catch | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `pickle-global` | imports | **MISS** | **MISS** | **MISS** | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `resources-files` | imports | **MISS** | catch | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `entry-point-load` | imports | **MISS** | **MISS** | catch | **MISS** | catch | catch | catch | refuse | refuse | late | late | refuse | catch | catch |
| B `runpy-submodule` | imports | **MISS** | catch | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `c-api-import-module` | imports | **MISS** | **MISS** | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `c-api-import-level` | imports | **MISS** | **MISS** | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| B `metadata-version` | no-import | pass | pass | **FALSE** | pass | pass | pass | pass | pass | pass | pass | pass | **FALSE** | pass | pass |
| B `walk-packages` | imports | **MISS** | **MISS** | **MISS** | **MISS** | catch | catch | catch | silent | refuse | refuse | refuse | silent | catch | catch |
| B `logging-config-factory` | imports | **MISS** | **MISS** | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| C `spec-from-file-package` | imports | **MISS** | **MISS** | catch | **MISS** | catch | catch | catch | late | late | late | late | late | catch | catch |
| C `spec-from-file-leaf` | imports | **MISS** | **MISS** | catch | **MISS** | **MISS** | catch | catch | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** | catch |
| C `exec-file-text` | imports | **MISS** | **MISS** | catch | **MISS** | **MISS** | catch | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** | catch |
| D `transitive-stmt` | imports | **MISS** | **MISS** | **MISS** | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| D `transitive-dynamic-two-hops` | imports | **MISS** | **MISS** | **MISS** | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| D `star-reexport` | imports | **MISS** | **MISS** | **MISS** | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| D `lazy-module-getattr` | imports | **MISS** | **MISS** | **MISS** | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| D `dependency-call` | imports | **MISS** | **MISS** | **MISS** | catch | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| D `preloaded-import-stmt` | binds | catch | catch | catch | catch | **MISS** | **MISS** | **MISS** | refuse | **MISS** | refuse | **MISS** | **MISS** | **MISS** | **MISS** |
| D `preloaded-sys-modules` | binds | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** | silent | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** |
| D `shadow-same-name` | shadow | name | name | name | name | name | pass | pass | name | name | name | name | pass | name | pass |
| D `atexit-import` | imports at exit | **MISS** | **MISS** | catch | **MISS** | **MISS** | catch | **MISS** | silent | refuse | refuse | refuse | silent | catch | catch |
| D `other-process` | other process | - | - | seen | - | - | - | - | - | - | - | - | - | - | seen |
| D `annotation-expression` | imports (3.12, 3.13) | **MISS** | catch | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| D `annotation-read` | imports | **MISS** | catch | catch | **MISS** | catch | catch | catch | refuse | refuse | refuse | refuse | refuse | catch | catch |
| D `future-annotations-name` | no-import | pass | pass | **FALSE** | pass | pass | pass | pass | pass | pass | pass | pass | pass | pass | pass |
| E `undo-then-import` | imports | **MISS** | **MISS** | catch | **MISS** | **MISS** | catch | **MISS** | **MISS** | **MISS** | **MISS** | **MISS** | refuse | **MISS** | catch |

Three things the matrix does not show by itself:

- A `catch` or `refuse` on an `import_module` route (`import-module`,
  `resolve-name`, `mock-patch-target`, `resources-files`,
  `transitive-dynamic-two-hops`) under `bimp`, `aimp` and `itime` came from
  `_pytest`, which pytest's own code imports by statement, not from the name
  `pytest` (`top_level_hits` in the JSON).
- The `mention` column's catches of `undo-then-import` and group C depend on
  the probe writing `_pytest` or a path in the text; the computed form of
  `dunder-import-computed` shows what it does when the text does not.
- `strace` sees every file read after the mark, so it counts any read of a
  target file, imported or not.

## What could not be settled

- **ruff's AST.** The static probes use Python's `ast`. That ruff's
  `Stmt::Import`, `Stmt::ImportFrom` and `level` match for every form here is
  read from purba's code, not run.
- **Python 3.15 lazy imports.** PEP 810 changes when a statement imports and
  adds `sys.lazy_modules`; nothing here ran on 3.15.
- **`.pth` and `usercustomize`.** Only `sitecustomize` was probed under purba.
- **False detections of `audit-open`** from a traceback, `linecache` or
  `inspect` reading a target file without importing it: not probed.
- **Threads.** An import from a thread that outlives a test's check was not
  probed. purba's zygote treats a live thread after the load specially
  ([`zygote.rs`](../crates/execution/src/zygote.rs)).
- **Platforms.** `strace` is Linux-only. macOS, free-threaded builds and
  subinterpreters were not probed.
- **`-X importtime=2`** of 3.14, which also reports cached imports, was not
  probed.
- **A real environment without pytest.** `path-without-target` simulates one
  with a filtered copy of `site-packages`.
- **Probe side effect.** An early run of `walk-packages` wrote three `.pyc`
  files into the probe virtualenv's `packaging` package. They were removed, and
  every later run sets `PYTHONDONTWRITEBYTECODE=1`.
