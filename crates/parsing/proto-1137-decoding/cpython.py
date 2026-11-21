"""PROTOTYPE #1137: what CPython gives each module, by a real import, then a call of test_one."""

import importlib.util
import pathlib
import sys

for path in map(pathlib.Path, sys.argv[1:]):
    try:
        spec = importlib.util.spec_from_file_location(path.stem, path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
    except SyntaxError as error:
        print(f"{path.name}\tSyntaxError: {error.msg}")
        continue
    try:
        module.test_one()
    except AssertionError:
        print(f"{path.name}\tfailed")
    else:
        print(f"{path.name}\tpassed")
