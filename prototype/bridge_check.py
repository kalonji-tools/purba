"""Prove the compiled extension loaded, not merely that a name resolved.

`import purba` reaches `purba/__init__.py`, whose first line is
`from .purba import *`. That does load the extension, but `purba.__file__`
reports the `__init__.py`, so it proves nothing on its own. This asserts on the
submodule that is the compiled artifact.
"""

import sys

import purba
import purba.purba as extension

path = extension.__file__ or ""
suffixes = (".so", ".pyd", ".dylib")

print(f"package  : {purba.__file__}")
print(f"extension: {path}")
print(f"python   : {sys.version.split()[0]}")

if not path.endswith(suffixes):
    print(f"FAIL: not a compiled artifact, the path ends with {path[-12:]!r}")
    raise SystemExit(1)

print("OK: the compiled extension loaded, so the PyO3 module init ran")
