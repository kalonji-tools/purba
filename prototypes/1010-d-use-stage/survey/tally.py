"""Tally the hand labels against the dump: classes, acts, what rule (b) refuses, and
fixture-to-fixture depth.

Depth: 1 for a fixture that takes no fixture of its project; else 1 + the deepest it takes. pytest's
own fixtures (monkeypatch, tmp_path, request, db, settings, pytestconfig) do not count. A name
defined twice takes its deepest visible definition; a fixture that takes its own name (an override)
takes the other definitions."""

from __future__ import annotations

import collections
import json
import sys
from functools import lru_cache
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from labels import L, SALEOR_BROAD_II, SALEOR_I, SALEOR_III  # noqa: E402

HERE = Path(__file__).parent


def visible(user: str, owner: str) -> bool:
    """pytest's scope: the same file, a conftest.py of an ancestor folder, or a plugin module
    (saleor's fixtures/)."""
    if owner == user or "/fixtures/" in owner or owner.endswith("/fixtures.py"):
        return True
    return owner.endswith("/conftest.py") and user.startswith(owner[: -len("conftest.py")])


def depths(graph_src: list[dict]) -> dict[tuple[str, int], int]:
    by_name: dict[str, list[dict]] = collections.defaultdict(list)
    for f in graph_src:
        by_name[f["name"]].append(f)

    @lru_cache(None)
    def depth(file: str, line: int, stack: frozenset = frozenset()) -> int:
        f = next(x for x in graph_src if x["file"] == file and x["line"] == line)
        best = 0
        for p in f["params"]:
            for g in by_name.get(p, []):
                key = (g["file"], g["line"])
                if not visible(file, g["file"]):
                    continue
                if key == (file, line) or key in stack:
                    continue
                best = max(best, depth(g["file"], g["line"], stack | {(file, line)}))
        return 1 + best

    return {(f["file"], f["line"]): depth(f["file"], f["line"]) for f in graph_src}


def label(project: str, i: int, f: dict) -> tuple[str, int, int]:
    if project == "saleor":
        if i in SALEOR_III:
            return "iii", 1, 1
        if i in SALEOR_I:
            return "i", 0, 0
        return "ii", 0, int(i in SALEOR_BROAD_II)
    tail = f["file"].split(f"ext-d/{project}/")[1]
    cls, strict, broad, _ = L[(project, tail, f["line"])]
    return cls, strict, broad


rows = []
for project, graph in (
    ("flask", "flask.json"),
    ("loguru", "loguru.json"),
    ("saleor", "saleor_all.json"),
):
    surveyed = json.loads((HERE / f"{project}.json").read_text())
    d = depths(json.loads((HERE / graph).read_text()))
    c = collections.Counter()
    deep = collections.Counter()
    for i, f in enumerate(surveyed, 1):
        cls, strict, broad = label(project, i, f)
        refused = not f["one_call"]
        c[cls] += 1
        c["strict"] += strict
        c["broad"] += broad
        c["refused"] += refused
        c["refused_no_act"] += refused and not broad
        c["accepted_but_acts"] += (not refused) and strict
        c["autouse"] += f["autouse"]
        deep[d[(f["file"], f["line"])]] += 1
    c["n"] = len(surveyed)
    rows.append((project, c, deep))

total = sum((r[1] for r in rows), collections.Counter())
alldeep = sum((r[2] for r in rows), collections.Counter())
rows.append(("total", total, alldeep))
cols = [
    "n",
    "i",
    "ii",
    "iii",
    "iv",
    "strict",
    "broad",
    "refused",
    "refused_no_act",
    "accepted_but_acts",
    "autouse",
]
print("project | " + " | ".join(cols) + " | depth 1/2/3/4/5+ | max depth")
for project, c, deep in rows:
    dd = [deep[k] for k in (1, 2, 3, 4)] + [sum(v for k, v in deep.items() if k >= 5)]
    print(
        f"{project} | "
        + " | ".join(str(c[k]) for k in cols)
        + f" | {'/'.join(map(str, dd))} | {max(deep)}"
    )


if (
    len(sys.argv) > 1
):  # print the deepest chain of one fixture name: python tally.py <project> <name>
    project, name = sys.argv[1], sys.argv[2]
    graph = json.loads(
        (HERE / ("saleor_all.json" if project == "saleor" else f"{project}.json")).read_text()
    )
    d = depths(graph)
    f = max((g for g in graph if g["name"] == name), key=lambda g: d[(g["file"], g["line"])])
    chain = [f["name"]]
    while True:
        nxt = [
            g
            for p in f["params"]
            for g in graph
            if g["name"] == p and visible(f["file"], g["file"]) and g is not f
        ]
        if not nxt:
            break
        f = max(nxt, key=lambda g: d[(g["file"], g["line"])])
        chain.append(f["name"])
    print(" -> ".join(chain))
