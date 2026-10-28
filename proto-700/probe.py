"""What test_copy's signature hands a resolver at runtime, for each way.

usage: python -I -B probe.py <dir-of-ways>
Prints one line per parameter: name, the hint as
typing.get_type_hints(include_extras=True) returns it, and the default.
"""

import importlib
import re
import inspect
import platform
import sys
import typing
from pathlib import Path


def short(value: object) -> str:
    text = repr(value)
    text = re.sub(r"<function ([\w.]+) at 0x[0-9a-f]+>", r"\1", text)
    text = re.sub(r"<class '([\w.]+)'>", r"\1", text)
    return re.sub(r"\b(?:w\d\d_\w+|subject|sketch|typing)\.", "", text)


def main() -> None:
    ways = Path(sys.argv[1])
    sys.path.insert(0, str(ways))
    print(f"## Python {platform.python_version()}, {ways.name}")
    for path in sorted(ways.glob("w*.py")):
        name = path.stem
        try:
            module = importlib.import_module(name)
            test = module.test_copy
            hints = typing.get_type_hints(test, include_extras=True)
        except Exception as error:  # noqa: BLE001 - the probe reports, it does not fix
            print(f"{name:28} REFUSED {type(error).__name__}: {error}")
            continue
        mapping = getattr(test, "__purba_using__", None)
        if mapping is not None:
            shown = {k: v.__name__ for k, v in mapping.items()}
            print(f"{name:28} @using {shown}")
        for param in inspect.signature(test).parameters.values():
            hint = hints.get(param.name, "<none>")
            default = "" if param.default is inspect.Parameter.empty else f" = {short(param.default)}"
            print(f"{name:28} {param.name}: {short(hint)}{default}")


if __name__ == "__main__":
    main()
