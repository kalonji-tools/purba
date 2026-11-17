"""The ways a test module can come to import or bind `pytest` or `_pytest`.

Each way is a test module, the helper modules it reaches through the path, and
what it does by construction:

- "imports": it runs code of pytest's package in this process;
- "imports-at-exit": it does so from an `atexit` callback, after the tests;
- "binds": it binds pytest's module, which was already loaded before the test;
- "no-import": it names pytest but imports nothing of it;
- "shadow": it imports a module named `pytest` that is not pytest;
- "other-process": it imports pytest in a child process only.

`matrix.py` writes each way to a directory: the test as `test_way.py`, each
helper under `lib/`, which the child puts first on `sys.path`.
"""

SITE_FILE = 'os.path.join(os.environ["PROBE_SITE"], {!r})'

WAYS = [
    # A. The import statement, with the target written in the source.
    ("import-stmt", "A", "imports", "import pytest\n", {}, False),
    ("import-as", "A", "imports", "import pytest as pt\n", {}, False),
    ("from-import", "A", "imports", "from pytest import raises\n", {}, False),
    ("star-import", "A", "imports", "from pytest import *\n", {}, False),
    ("private-submodule", "A", "imports", "import _pytest.outcomes\n", {}, False),
    ("from-private", "A", "imports", "from _pytest.outcomes import fail\n", {}, False),
    (
        "in-test-body",
        "A",
        "imports",
        "def test_x():\n    import pytest\n    assert pytest\n",
        {},
        False,
    ),
    (
        "under-type-checking",
        "A",
        "no-import",
        "from typing import TYPE_CHECKING\nif TYPE_CHECKING:\n    import pytest\n",
        {},
        False,
    ),
    (
        "try-optional",
        "A",
        "imports",
        "try:\n    import pytest\nexcept ImportError:\n    pytest = None\n",
        {},
        False,
    ),
    (
        "relative-from",
        "A",
        "no-import",
        "try:\n    from . import pytest\nexcept ImportError:\n    pass\n",
        {},
        False,
    ),
    # B. A call that imports a module whose name it takes as data.
    ("dunder-import-literal", "B", "imports", '__import__("pytest")\n', {}, False),
    ("dunder-import-computed", "B", "imports", '__import__("py" + "test")\n', {}, False),
    (
        "import-module",
        "B",
        "imports",
        'import importlib\nimportlib.import_module("pytest")\n',
        {},
        False,
    ),
    ("exec-string", "B", "imports", 'exec("import pytest")\n', {}, False),
    ("eval-string", "B", "imports", "eval(\"__import__('pytest')\")\n", {}, False),
    (
        "find-spec-submodule",
        "B",
        "imports",
        'import importlib.util\nimportlib.util.find_spec("_pytest.outcomes")\n',
        {},
        False,
    ),
    (
        "find-spec-top",
        "B",
        "no-import",
        'import importlib.util\nimportlib.util.find_spec("pytest")\n',
        {},
        False,
    ),
    (
        "resolve-name",
        "B",
        "imports",
        'import pkgutil\npkgutil.resolve_name("pytest:raises")\n',
        {},
        False,
    ),
    (
        "mock-patch-target",
        "B",
        "imports",
        'from unittest import mock\nwith mock.patch("pytest.raises"):\n    pass\n',
        {},
        False,
    ),
    (
        "pickle-global",
        "B",
        "imports",
        'import pickle\npickle.loads(b"cpytest\\nraises\\n.")\n',
        {},
        False,
    ),
    (
        "resources-files",
        "B",
        "imports",
        'import importlib.resources\nimportlib.resources.files("pytest")\n',
        {},
        False,
    ),
    (
        "entry-point-load",
        "B",
        "imports",
        "import importlib.metadata\n"
        'eps = importlib.metadata.entry_points(group="console_scripts", name="pytest")\n'
        "[ep] = list(eps)\nep.load()\n",
        {},
        False,
    ),
    (
        "runpy-submodule",
        "B",
        "imports",
        'import runpy\nrunpy.run_module("_pytest.outcomes")\n',
        {},
        False,
    ),
    (
        "c-api-import-module",
        "B",
        "imports",
        "import ctypes\nf = ctypes.pythonapi.PyImport_ImportModule\n"
        'f.restype = ctypes.py_object\nf.argtypes = [ctypes.c_char_p]\nf(b"pytest")\n',
        {},
        False,
    ),
    (
        "c-api-import-level",
        "B",
        "imports",
        "import ctypes\nf = ctypes.pythonapi.PyImport_ImportModuleLevel\n"
        "f.restype = ctypes.py_object\n"
        "f.argtypes = [ctypes.c_char_p, ctypes.py_object, ctypes.py_object,"
        " ctypes.py_object, ctypes.c_int]\n"
        'f(b"pytest", None, None, None, 0)\n',
        {},
        False,
    ),
    (
        "metadata-version",
        "B",
        "no-import",
        'import importlib.metadata\nimportlib.metadata.version("pytest")\n',
        {},
        False,
    ),
    (
        "walk-packages",
        "B",
        "imports",
        "import os, pkgutil\n"
        'for _ in pkgutil.walk_packages([os.environ["PROBE_SITE"]], onerror=lambda name: None):\n'
        "    pass\n",
        {},
        False,
    ),
    (
        "logging-config-factory",
        "B",
        "imports",
        "import logging.config\n"
        "logging.config.dictConfig(\n"
        '    {"version": 1, "filters": {"f": {"()": "pytest.MonkeyPatch"}}}\n'
        ")\n",
        {},
        False,
    ),
    # C. Code that runs a file of pytest's package without importing its name.
    (
        "spec-from-file-package",
        "C",
        "imports",
        "import os, importlib.util\n"
        f"p = {SITE_FILE.format('pytest/__init__.py')}\n"
        's = importlib.util.spec_from_file_location("not_the_target", p)\n'
        "m = importlib.util.module_from_spec(s)\ns.loader.exec_module(m)\n",
        {},
        False,
    ),
    (
        "spec-from-file-leaf",
        "C",
        "imports",
        "import os, importlib.util\n"
        f"p = {SITE_FILE.format('_pytest/_version.py')}\n"
        's = importlib.util.spec_from_file_location("not_the_target", p)\n'
        "m = importlib.util.module_from_spec(s)\ns.loader.exec_module(m)\n",
        {},
        False,
    ),
    (
        "exec-file-text",
        "C",
        "imports",
        "import os\n"
        f"p = {SITE_FILE.format('_pytest/_version.py')}\n"
        "with open(p) as f:\n    text = f.read()\n"
        'ns = {}\nexec(compile(text, "<a copy>", "exec"), ns)\n',
        {},
        False,
    ),
    # D. The test's source does not show the target.
    (
        "transitive-stmt",
        "D",
        "imports",
        "import helper_imports\n",
        {"helper_imports.py": "import pytest\n"},
        False,
    ),
    (
        "transitive-dynamic-two-hops",
        "D",
        "imports",
        "import helper_chain\n",
        {
            "helper_chain.py": "import helper_dyn\n",
            "helper_dyn.py": 'import importlib\nimportlib.import_module("py" + "test")\n',
        },
        False,
    ),
    (
        "star-reexport",
        "D",
        "imports",
        "from helper_imports import *\n",
        {"helper_imports.py": "import pytest\n"},
        False,
    ),
    (
        "lazy-module-getattr",
        "D",
        "imports",
        "import helper_lazy\n\ndef test_x():\n    assert helper_lazy.thing\n",
        {
            "helper_lazy.py": "def __getattr__(name):\n"
            "    import pytest\n    return pytest\n"
        },
        False,
    ),
    (
        "dependency-call",
        "D",
        "imports",
        "import helper_fn\n\ndef test_x():\n    assert helper_fn.make()\n",
        {"helper_fn.py": "def make():\n    import pytest\n    return pytest.approx\n"},
        False,
    ),
    ("preloaded-import-stmt", "D", "binds", "import pytest\n", {}, True),
    (
        "preloaded-sys-modules",
        "D",
        "binds",
        'import sys\nP = sys.modules["py" + "test"]\n',
        {},
        True,
    ),
    (
        "shadow-same-name",
        "D",
        "shadow",
        "import pytest\n",
        {"pytest.py": "SHADOW = True\n"},
        False,
    ),
    (
        "atexit-import",
        "D",
        "imports-at-exit",
        'import atexit\natexit.register(__import__, "pytest")\n',
        {},
        False,
    ),
    (
        "other-process",
        "D",
        "other-process",
        "import subprocess, sys\n"
        'subprocess.run([sys.executable, "-c", "import pytest"], check=True)\n',
        {},
        False,
    ),
    (
        "annotation-expression",
        "D",
        "imports",
        'def f(x: __import__("pytest").MonkeyPatch):\n    pass\n',
        {},
        False,
    ),
    (
        "annotation-read",
        "D",
        "imports",
        'def f(x: __import__("pytest").MonkeyPatch):\n    pass\n\nA = f.__annotations__\n',
        {},
        False,
    ),
    (
        "future-annotations-name",
        "D",
        "no-import",
        "from __future__ import annotations\n\ndef f(x: pytest.MonkeyPatch):\n    pass\n",
        {},
        False,
    ),
    # E. A test that takes apart what an in-process means put in place.
    (
        "undo-then-import",
        "E",
        "imports",
        "import builtins, importlib, sys\n"
        "for k in [k for k, v in sys.modules.items() if v is None]:\n"
        "    del sys.modules[k]\n"
        "sys.meta_path[:] = [f for f in sys.meta_path\n"
        '    if getattr(f, "__module__", "").startswith("_frozen_importlib")]\n'
        "builtins.__import__ = importlib.__import__\n"
        "for tool in range(6):\n"
        "    if sys.monitoring.get_tool(tool):\n"
        "        sys.monitoring.set_events(tool, 0)\n"
        'P = importlib.import_module("py" + "test")\n'
        "for k in [k for k in sys.modules if k.split('.')[0] in ('pytest', '_pytest')]:\n"
        "    del sys.modules[k]\n",
        {},
        False,
    ),
]
