"""PROTOTYPE for #1019, never merge: how find_spec and pytest.importorskip meet a module.

A missing module, a module whose import raises (a broken install), and a submodule
of a missing package.
"""

import pathlib
import sys
import tempfile
from importlib.util import find_spec

import pytest

site = pathlib.Path(tempfile.mkdtemp())
(site / "brokenmod.py").write_text('raise ImportError("broken install")\n')
sys.path.insert(0, str(site))

for name in ("json", "no_such_mod", "brokenmod", "no_such_pkg.sub"):
    try:
        print(f"{name:18} find_spec is None -> {find_spec(name) is None}")
    except Exception as error:
        print(f"{name:18} find_spec RAISES {type(error).__name__}: {error}")
for name in ("no_such_mod", "brokenmod"):
    try:
        pytest.importorskip(name)
    except BaseException as error:
        print(f"importorskip({name!r}) -> {type(error).__name__}: {error}")
