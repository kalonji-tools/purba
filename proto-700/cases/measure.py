"""Mechanical proxies for each way, and jedi's go-to-definition from the test line.

usage: uv run --python 3.12 --with jedi --with pytest python -I -B measure.py <scratch-dir>
Prints JSON lines: one per way and case, then one per way.
"""

import ast
import json
import re
import sys
from pathlib import Path

import jedi

HERE = Path(__file__).parent
FRAMEWORK = {"Annotated", "From", "use", "use_any", "using", "each", "tx"}
FIXTURES = {"database", "source", "target", "big_source", "sqlite", "postgres", "settings",
            "client", "clock", "mailer", "seeded", "tx"}


def depth(text: str) -> int:
    level = top = 0
    for char in text:
        if char in "([":
            level += 1
            top = max(top, level)
        elif char in ")]":
            level -= 1
    return top


def signature(source: str, fn: ast.FunctionDef) -> str:
    """The text a reader reads for the inputs: @using's arguments and the parameters."""
    parts = []
    for deco in fn.decorator_list:
        if isinstance(deco, ast.Call) and getattr(deco.func, "id", "") == "using":
            parts.append("@" + ast.get_source_segment(source, deco))
    args = fn.args
    defaults = [None] * (len(args.args) - len(args.defaults)) + list(args.defaults)
    for arg, default in zip(args.args, defaults, strict=True):
        text = arg.arg
        if arg.annotation:
            text += ": " + ast.get_source_segment(source, arg.annotation)
        if default is not None:
            text += " = " + ast.get_source_segment(source, default)
        parts.append(text)
    return ", ".join(parts)


def fixture_refs(way: str, fn: ast.FunctionDef) -> int:
    """Names on the line that point at a fixture: what renaming a fixture must touch."""
    if way == "w06_pytest":
        return sum(a.arg in FIXTURES | {"src", "dst", "db", "engine"} for a in fn.args.args)
    nodes: list[ast.AST] = list(fn.args.defaults)
    nodes += [a.annotation for a in fn.args.args if a.annotation]
    for deco in fn.decorator_list:
        if isinstance(deco, ast.Call) and getattr(deco.func, "id", "") == "using":
            nodes += [k.value for k in deco.keywords]
    return sum(isinstance(x, ast.Name) and x.id in FIXTURES - {"tx"} for node in nodes for x in ast.walk(node))


def per_case(way: Path) -> list[dict[str, object]]:
    rows = []
    for path in sorted(way.glob("test_c*.py")):
        source = path.read_text()
        tree = ast.parse(source)
        sigs = [signature(source, n) for n in tree.body if isinstance(n, ast.FunctionDef)]
        names = [t for s in sigs for t in re.findall(r"[A-Za-z_]\w*", s)]
        rows.append({
            "way": way.name, "case": path.stem[5:7],
            "chars": round(sum(map(len, sigs)) / len(sigs), 1),
            "depth": max(map(depth, sigs)),
            "framework_tokens": sum(n in FRAMEWORK for n in names),
            "fixture_refs_on_line": sum(fixture_refs(way.name, n) for n in tree.body if isinstance(n, ast.FunctionDef)),
            "lines": sum(1 for line in source.splitlines() if line.strip()),
        })
    return rows


TOKEN = {  # the token on test_c1's line that names src's value
    "w01_record_690": "SourceDb", "w03_alias": "SourceTx", "w06_pytest": "src",
    "w08_decorator_map": "source", "w09_default_bare": "source",
    "w10_default_untyped_marker": "source", "w11_default_typed_marker": "source",
    "w12_annotated_provider": "source",
}


def goto(way: Path, scratch: Path) -> str:
    path = way / "test_c1_copy.py"
    lines = path.read_text().splitlines()
    token = TOKEN[way.name]
    for number, line in enumerate(lines, 1):
        match = re.search(rf"(?<![\w.]){token}(?!\w)", line)
        if match and (line.startswith(("def test_copy", "@using", "    src")) or "src" in line):
            break
    else:
        return "token not found"
    if way.name == "w06_pytest":
        return pytest_goto(scratch)
    script = jedi.Script(path=str(path), project=jedi.Project(str(way)))
    found = script.goto(number, match.start(), follow_imports=True)
    return "; ".join(f"{Path(d.module_path).name}:{d.line} {d.get_line_code().strip()}" for d in found) or "nothing"


def pytest_goto(scratch: Path) -> str:
    """The same question asked of real pytest fixtures, which jedi knows."""
    root = scratch / "jedi_pytest"
    root.mkdir(parents=True, exist_ok=True)
    (root / "conftest.py").write_text(
        "import pytest\n\n\n@pytest.fixture\ndef src() -> list[str]:\n    return []\n")
    test = root / "test_copy.py"
    test.write_text("def test_copy(src: list[str]) -> None:\n    assert src == []\n")
    script = jedi.Script(path=str(test), project=jedi.Project(str(root)))
    found = script.goto(1, len("def test_copy("), follow_imports=True)
    return "real pytest: " + ("; ".join(f"{Path(d.module_path).name}:{d.line} {d.get_line_code().strip()}"
                                       for d in found) or "nothing")


def main() -> None:
    scratch = Path(sys.argv[1])
    for way in sorted(HERE.glob("w*/")):
        cases = per_case(way)
        for row in cases:
            print(json.dumps(row))
        fixtures = (way / "fixtures.py").read_text()
        total = sum(1 for line in fixtures.splitlines() if line.strip()) + sum(int(r["lines"]) for r in cases)
        print(json.dumps({"way": way.name, "lines_written_total": total, "goto_from_src": goto(way, scratch)}))


if __name__ == "__main__":
    main()
