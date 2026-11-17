"""What the process that purba loads a test module in holds, and what purba does
today with a test module that imports pytest.

    python purba_process.py <purba executable> <work dir>

It writes test modules to <work dir>, runs purba on each, and prints, as
Markdown, what each module recorded as it loaded, and purba's own output.

1. `state`: what the load sees: whether `pytest` is in `sys.modules`, how many
   modules are, `sys.path[0]`, the `sys.meta_path` finders, the test module's
   `__package__` and `__spec__`, and the tools of `sys.monitoring` in use.
2. `sibling`: whether a module next to the test module is importable.
3. `preloaded`: the same state with a `sitecustomize` on PYTHONPATH that
   imports pytest before purba loads the test module.
4. `per-test`: whether an import in one test's body is still in
   `sys.modules` for the next test, and the tools of `sys.monitoring` in use
   while a test runs.
5. `relative`: what `from . import x` raises in a module that purba loads.
"""

import json
import os
import subprocess
import sys

STATE = """\
import json, os, sys
_ = {
    "pytest_in_sys_modules": "pytest" in sys.modules,
    "_pytest_in_sys_modules": "_pytest" in sys.modules,
    "modules": len(sys.modules),
    "sys_path_0": os.path.basename(os.path.dirname(sys.path[0])) + "/"
    + os.path.basename(sys.path[0]),
    "meta_path": [getattr(f, "__name__", type(f).__name__) for f in sys.meta_path],
    "package": __package__,
    "spec": repr(__spec__),
    "monitoring_tools": [sys.monitoring.get_tool(i) for i in range(6)],
    "import_function_type": type(__import__).__name__,
}
with open(os.environ["PROBE_OUT"], "w") as f:
    json.dump(_, f)


def test_one():
    assert 1 == 1
"""

SIBLING = """\
import helper_next_to_me


def test_one():
    assert 1 == 1
"""

PER_TEST = """\
import json
import os
import sys


def test_imports():
    import pytest
    assert pytest.__name__ == "pytest"


def test_after():
    with open(os.environ["PROBE_OUT"], "a") as f:
        f.write(json.dumps({"pytest_in_sys_modules": "pytest" in sys.modules,
                            "monitoring_tools": [sys.monitoring.get_tool(i) for i in range(6)]}))
    assert 1 == 1
"""

RELATIVE = """\
import os
try:
    from . import anything
except ImportError as err:
    with open(os.environ["PROBE_OUT"], "w") as f:
        f.write(f"{type(err).__name__}: {err}")


def test_one():
    assert 1 == 1
"""

IMPORTS = """\
import pytest


def test_one():
    assert pytest.__name__ == "pytest"
"""


def run(purba, work, name, text, extra_env=None):
    path = os.path.join(work, f"test_{name}.py")
    with open(path, "w") as f:
        f.write(text)
    out = os.path.join(work, f"{name}.out")
    if os.path.exists(out):
        os.remove(out)
    env = dict(os.environ, PROBE_OUT=out, **(extra_env or {}))
    done = subprocess.run(
        [purba, os.path.basename(path)], cwd=work, capture_output=True, text=True, env=env
    )
    recorded = None
    if os.path.exists(out):
        with open(out) as f:
            recorded = f.read()
    if recorded and recorded.startswith("{"):
        recorded = json.dumps(json.loads(recorded), indent=1)
    print(f"\n## `{name}`\n")
    print(f"What the module recorded:\n\n```text\n{recorded}\n```\n")
    output = (done.stdout + done.stderr).replace(work, "<work>").rstrip()
    print(f"purba's output, exit {done.returncode}:\n\n```text\n{output}\n```")


def main():
    purba, work = sys.argv[1], sys.argv[2]
    os.makedirs(work, exist_ok=True)
    with open(os.path.join(work, "helper_next_to_me.py"), "w") as f:
        f.write("X = 1\n")
    custom = os.path.join(work, "custom")
    os.makedirs(custom, exist_ok=True)
    with open(os.path.join(custom, "sitecustomize.py"), "w") as f:
        f.write("import pytest\n")
    print("# What purba's process holds\n")
    print("Written by `purba_process.py`.")
    run(purba, work, "state", STATE)
    run(purba, work, "sibling", SIBLING)
    run(purba, work, "preloaded", STATE, {"PYTHONPATH": custom})
    run(purba, work, "per_test", PER_TEST)
    run(purba, work, "relative", RELATIVE)
    run(purba, work, "imports", IMPORTS)


main()
