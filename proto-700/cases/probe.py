"""Imports every module of one way, and prints what each test and fixture
signature in its test modules hands a resolver at runtime.
usage: python -I -B probe.py <way-dir>
"""

import importlib
import inspect
import platform
import re
import sys
import typing
from pathlib import Path


def short(value: object) -> str:
    text = repr(value)
    text = re.sub(r"<function ([\w.]+) at 0x[0-9a-f]+>", r"\1", text)
    text = re.sub(r"<class '([\w.]+)'>", r"\1", text)
    return re.sub(r"\b(?:fixtures|subject|sketch|typing)\.", "", text)


def main() -> None:
    way = Path(sys.argv[1])
    sys.path.insert(0, str(way))
    importlib.import_module("fixtures")
    for path in sorted(way.glob("test_c*.py")):
        try:
            module = importlib.import_module(path.stem)
        except Exception as error:  # noqa: BLE001
            print(f"{way.name} {path.stem} REFUSED {type(error).__name__}: {error}")
            continue
        for name, fn in vars(module).items():
            if not inspect.isfunction(fn) or fn.__module__ != path.stem:
                continue
            hints = typing.get_type_hints(fn, include_extras=True)
            parts = []
            for param in inspect.signature(fn).parameters.values():
                default = "" if param.default is inspect.Parameter.empty else f" = {short(param.default)}"
                parts.append(f"{param.name}: {short(hints.get(param.name, '<none>'))}{default}")
            using = getattr(fn, "__using__", None)
            pre = f"@using{ {k: short(v) for k, v in using.items()} } " if using else ""
            print(f"{way.name} {path.stem[:7]} {pre}{name}({', '.join(parts)})")


if __name__ == "__main__":
    main()
