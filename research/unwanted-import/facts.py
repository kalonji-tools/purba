"""Single facts that the findings cite, each checked in a fresh interpreter.

    python facts.py

The target here is `colorsys`, a standard module that imports nothing and that
no interpreter loads at start-up, so each fact holds for any module name.
"""

import subprocess
import sys

FACTS = {
    "an import statement raises the audit event `import`": """
import sys
seen = []
sys.addaudithook(lambda e, a: seen.append(a[0]) if e == "import" else None)
import colorsys
print("colorsys" in seen)
""",
    "importlib.import_module raises no audit event `import` for its module": """
import sys, importlib
seen = []
sys.addaudithook(lambda e, a: seen.append(a[0]) if e == "import" else None)
importlib.import_module("colorsys")
print("colorsys" in seen, "colorsys" in sys.modules)
""",
    "importlib.import_module never calls builtins.__import__ for its module": """
import builtins, importlib, sys
seen = []
original = builtins.__import__
def spy(name, *args, **kwargs):
    seen.append(name)
    return original(name, *args, **kwargs)
builtins.__import__ = spy
importlib.import_module("colorsys")
print("colorsys" in seen, "colorsys" in sys.modules)
""",
    "-X importtime prints no line for a module that import_module loads": """
import subprocess, sys
for code in ("import colorsys", "import importlib; importlib.import_module('colorsys')"):
    done = subprocess.run([sys.executable, "-X", "importtime", "-c", code],
                          capture_output=True, text=True)
    print(code, "->", any(l.endswith("| colorsys") for l in done.stderr.splitlines()))
""",
    "a meta path finder is not asked about a module already in sys.modules": """
import sys
import colorsys
asked = []
class Spy:
    @staticmethod
    def find_spec(name, path=None, target=None):
        asked.append(name)
sys.meta_path.insert(0, Spy)
import colorsys
print(asked)
""",
    "builtins.__import__ is called for a module already in sys.modules": """
import builtins
import colorsys
seen = []
original = builtins.__import__
def spy(name, *args, **kwargs):
    seen.append(name)
    return original(name, *args, **kwargs)
builtins.__import__ = spy
import colorsys
print(seen)
""",
    "None in sys.modules makes find_spec report the module as absent": """
import sys, importlib.util
before = importlib.util.find_spec("colorsys") is not None
sys.modules["colorsys"] = None
print(before, importlib.util.find_spec("colorsys"))
""",
    "a finder that raises makes find_spec raise, though find_spec runs no module": """
import sys, importlib.util
class Refuse:
    @staticmethod
    def find_spec(name, path=None, target=None):
        if name == "colorsys":
            raise ImportError("refused")
sys.meta_path.insert(0, Refuse)
try:
    importlib.util.find_spec("colorsys")
except ImportError as err:
    print("raised", err, "colorsys" in sys.modules)
""",
    "an audit hook cannot be removed by Python code, and stays for each later import": """
import sys
sys.addaudithook(lambda e, a: None)
print(hasattr(sys, "removeaudithook"), [n for n in dir(sys) if "audit" in n])
""",
    "spec_from_file_location runs a module without the meta path and without sys.modules": """
import sys, importlib.util, colorsys
path = colorsys.__file__
del sys.modules["colorsys"]
asked = []
class Spy:
    @staticmethod
    def find_spec(name, p=None, t=None):
        asked.append(name)
sys.meta_path.insert(0, Spy)
spec = importlib.util.spec_from_file_location("other_name", path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
print(asked, "other_name" in sys.modules, "colorsys" in sys.modules, module.rgb_to_hsv)
""",
}


def main():
    print(f"CPython {sys.version.split()[0]}")
    for claim, code in FACTS.items():
        done = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True)
        print(f"- {claim}\n    {(done.stdout + done.stderr).strip()}")


main()
