"""Sort the module-level functions of the form's ports by the marker of their name.

usage: python -I form_ports.py <file.py> ...
Reads each file with `ast`; imports nothing.
"""

import ast
import sys

for path in sys.argv[1:]:
    with open(path, "rb") as fh:
        tree = ast.parse(fh.read(), path)
    test, public, private = [], [], []
    for n in tree.body:
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if n.name.startswith("test"):
                test.append(n.name)
            elif n.name.startswith("_"):
                private.append(n.name)
            else:
                public.append(n.name)
    print(f"{path}\ttest={len(test)}\tpublic={len(public)}\tprivate={len(private)}\tpublic_names={','.join(public)}")
