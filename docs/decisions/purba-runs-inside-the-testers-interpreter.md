# purba runs inside the tester's interpreter

## Context and Problem Statement

Where purba ends depends on where CPython sits.
The scaffold took the shape of an extension module before any decision chose it.
A Rust host could start first and load the tester's interpreter, or ship an interpreter of its own.
Each arrangement decides which interpreter runs the tester's code.

| fact | source |
|---|---|
| on Linux and macOS, a Rust host built with PyO3 links the `libpython` of one version, such as `python3.12`, even with `abi3` | [PyO3 build configuration](https://github.com/PyO3/pyo3/blob/v0.29.3/pyo3-build-config/src/impl_.rs#L562-L577), [its Unix library name](https://github.com/PyO3/pyo3/blob/v0.29.3/pyo3-build-config/src/impl_.rs#L2446-L2475) |
| CPython finds a virtual environment from the path of its executable, through `pyvenv.cfg` | [CPython path configuration](https://github.com/python/cpython/blob/v3.15.0/Modules/getpath.py#L340-L380) |
| an interpreter skips a compiled module whose suffix names another version or threading variant | [What does each way of hosting CPython give a Python test runner written in Rust, and what does it cost?](https://github.com/kalonji-tools/purba/issues/481#issuecomment-6107174085) |
| pip refuses an `abi3` wheel for a free-threaded interpreter | [Which way of hosting CPython holds up when purba runs one test in a tester's virtual environment?](https://github.com/kalonji-tools/purba/issues/482#issuecomment-6107327309) |
| a Rust host ran the test in the tester's virtual environment only when a launcher found `libpython` and that environment came first on `PATH` | [Which way of hosting CPython holds up when purba runs one test in a tester's virtual environment?](https://github.com/kalonji-tools/purba/issues/482#issuecomment-6107327309) |
| a Rust host built for 3.12, with a 3.14t environment first on `PATH`, ran the test on 3.12 and reported that environment's `python3` as `sys.executable` | [Which way of hosting CPython holds up when purba runs one test in a tester's virtual environment?](https://github.com/kalonji-tools/purba/issues/482#issuecomment-6107327309) |

## Considered Options

- **A Rust host that loads the tester's `libpython`.** Rejected. The choice of interpreter becomes purba's work. A wrong choice reaches the tester as a test failure, or as a report of an interpreter that did not run the test.
- **A Rust host that ships its own CPython.** Rejected. A compiled package in the tester's environment that is not `abi3` loads only when its version and threading variant match the bundled interpreter.
- **An extension module that the tester's interpreter loads.** Chosen. The interpreter that the tester chose is the one that runs the test.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/Cargo.toml` `/pyproject.toml` `.github/scripts/require-abi3.sh` `src/**`

**purba runs inside the tester's interpreter, as an extension module.**
The tester's CPython loads purba, so each test runs on the interpreter and the packages that the tester chose.
CPython is outside purba's boundary.
[The landscape](../../LANDSCAPE.md#the-context) draws each neighbour.

| neighbour | what crosses the boundary |
|---|---|
| [tester](../../CONTEXT.md#tester), a person or an agent | the tester runs purba, and purba returns a report as text and as data |
| [integrator](../../CONTEXT.md#integrator), such as CI or an IDE | the integrator runs purba, and purba returns an exit code and data |
| the tester's project | purba reads the tests as text, then imports the selected tests |
| CPython, the tester's interpreter | it loads purba as an extension module |

**What stays off the diagram:**

- **`conftest.py`.** purba does not read it, because a test names what affects it. [purba ranks an honest outcome first](purba-ranks-an-honest-outcome-first.md) rejects a pytest suite run unchanged.
- **pytest.** It runs the control run of a pinned suite. purba does not call it.
- **The packager.** It builds purba before purba runs. No data crosses while purba runs.
- **The library author.** It joins the diagram when the public surface holds something that a test imports.
- **A plugin.** [purba's public surface starts empty and has no plugin API](purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome) refuses a plugin API.
- **How purba is developed.** The landscape draws the product.

**Downside:**

- **pip refuses the `abi3` wheel on a free-threaded interpreter.** A free-threaded interpreter needs a wheel of its own, as [purba supports a Python with two years left](purba-supports-a-python-with-two-years-left.md) states.
- **Selection starts after the interpreter starts.** purba reads the tests as text before any test import. It cannot read them before CPython starts.

## Confirmation

`.github/scripts/require-abi3.sh` refuses to upload a wheel unless its tag is `abi3` at the floor.
A Rust host builds a wheel with no `abi3` tag, so the upload refuses it.
No check reads the diagram, because [a standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md).
The walking skeleton is the first to run one test through this boundary.
