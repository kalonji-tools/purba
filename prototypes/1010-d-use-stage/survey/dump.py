"""Dump every pytest fixture under the given roots: params, statement and call counts, yield, and
its body. Reads the files as data (ast only); imports nothing from them."""

from __future__ import annotations

import ast
import json
import sys
from pathlib import Path


def is_fixture(dec: ast.expr) -> bool:
    target = dec.func if isinstance(dec, ast.Call) else dec
    return ast.unparse(target) in {"pytest.fixture", "fixture", "pytest.yield_fixture"}


def body_of(fn: ast.FunctionDef | ast.AsyncFunctionDef) -> list[ast.stmt]:
    body = fn.body
    if (
        body
        and isinstance(body[0], ast.Expr)
        and isinstance(body[0].value, ast.Constant)
        and isinstance(body[0].value.value, str)
    ):
        body = body[1:]
    return body


def main(roots: list[str]) -> None:
    out = []
    for root in roots:
        for path in sorted(Path(root).rglob("*.py")):
            try:
                tree = ast.parse(path.read_text(encoding="utf-8"))
            except (SyntaxError, UnicodeDecodeError):
                continue
            for node in ast.walk(tree):
                if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    continue
                decs = [d for d in node.decorator_list if is_fixture(d)]
                if not decs:
                    continue
                body = body_of(node)
                calls = sum(isinstance(n, ast.Call) for s in body for n in ast.walk(s))
                yields = any(
                    isinstance(n, (ast.Yield, ast.YieldFrom)) for s in body for n in ast.walk(s)
                )
                autouse = any("autouse=True" in ast.unparse(d) for d in decs)
                params = [a.arg for a in node.args.args if a.arg not in {"self", "request"}]
                out.append(
                    dict(
                        file=str(path),
                        line=node.lineno,
                        name=node.name,
                        params=params,
                        statements=len(body),
                        calls=calls,
                        yields=yields,
                        autouse=autouse,
                        one_call=(
                            len(body) == 1 and isinstance(body[0], ast.Return) and calls <= 1
                        ),
                        source=ast.get_source_segment(path.read_text(encoding="utf-8"), node),
                    )
                )
    json.dump(out, sys.stdout, indent=1)


if __name__ == "__main__":
    main(sys.argv[1:])
