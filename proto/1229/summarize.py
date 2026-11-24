"""Prototype for #1229; this branch is never merged.

Reads one tally of `crates/command/examples/tally.rs` (gzipped or plain) and
the per-file table of #1225, and prints one count per row and per suite.
It reads only these tables. It imports nothing from the suites.

    python -I proto/1229/summarize.py <tally.tsv[.gz]> <files.tsv> <functions.tsv> [<judged.tsv>]

Matchers:
- a test file: #1225's `kind == test` (test_*.py or *_test.py, anywhere).
- collected: #1225's `own_collects == 1`, the suite's own command collects it.
- a module-level test: a T row whose id is `<path>::<name>`.
- a method: a T row whose id holds a class, `<path>::<class>::<name>`.
- a helper caught as a test: a module-level test whose name does not start
  with `test`, or whose file is neither a test file nor collected.
- a collected test missed: a `test*` row of #1225's functions table, in a
  collected file, that no T row names.
- a near-miss hit: an R row of kind `name-near`, `name-prefix`, `asserts` or
  `file-near`; `judged.tsv` marks each one true or false.
"""

import csv
import gzip
import os
import sys
from collections import Counter, defaultdict

SUITES = ["loguru", "attrs", "click", "requests", "dateutil", "pyflakes", "toolz", "six", "schedule"]


def rows(path):
    opener = gzip.open if path.endswith(".gz") else open
    with opener(path, "rt", encoding="utf-8") as handle:
        for line in handle:
            yield line.rstrip("\n").split("\t")


def main():
    tally, files_tsv, functions_tsv = sys.argv[1], sys.argv[2], sys.argv[3]
    judged = {}
    if len(sys.argv) > 4 and os.path.exists(sys.argv[4]):
        for row in rows(sys.argv[4]):
            if row and row[0] in ("true", "false"):
                judged[tuple(row[1:5])] = row[0]
    info = {}
    with open(files_tsv, encoding="utf-8") as handle:
        for row in csv.DictReader(handle, delimiter="\t"):
            info[(row["suite"], row["path"])] = row
    count = defaultdict(Counter)
    picked = set()
    found_tests = set()
    for row in rows(tally):
        kind, suite = row[0], row[1]
        c = count[suite]
        if kind == "F":
            path = row[2]
            picked.add((suite, path))
            meta = info[(suite, path)]
            c["files picked"] += 1
            c["… test files"] += meta["kind"] == "test"
            c["… other files"] += meta["kind"] != "test"
            c["… … __init__.py"] += meta["kind"] == "init"
            c["… … conftest.py"] += meta["kind"] == "conftest"
            c["… … other modules"] += meta["kind"] == "other"
            c["… files that do not parse"] += row[3] != "ok"
            c["module faults (one line each)"] += row[4] != "-"
        elif kind == "T":
            path, ident, verdict, reasons = row[2], row[3], row[4], row[5]
            meta = info[(suite, path)]
            parts = ident.split("::")
            name = parts[-1]
            if len(parts) > 2:
                c["methods refused (one line each)"] += 1
                continue
            found_tests.add((suite, path, name))
            c["module-level tests found"] += 1
            c["… selected"] += verdict == "selected"
            c["… refused (one line each)"] += verdict == "refused"
            nodef = "with no def" in reasons
            c["… … of them a name bound with no def"] += nodef
            collected = meta["own_collects"] == "1"
            helper = not name.startswith("test") or (meta["kind"] != "test" and not collected)
            if helper:
                c["helpers caught as tests"] += 1
                c["… a def"] += not nodef
                c["… a name bound with no def"] += nodef
                c["… named test, in a file no command collects, not a test file"] += name.startswith("test")
            elif collected:
                c["… in a collected file, named test"] += 1
            else:
                c["… in a test file no command collects (benchmarks)"] += 1
        elif kind == "R":
            path, check, words = row[2], row[3], row[4]
            c["check lines"] += 1
            c[f"… {check}"] += 1
            if check in ("name-near", "name-prefix", "asserts", "file-near", "list-outside"):
                verdict = judged.get((suite, path, check, words), "unjudged")
                c[f"… … {check} {verdict}"] += 1
    with open(functions_tsv, encoding="utf-8") as handle:
        for row in csv.DictReader(handle, delimiter="\t"):
            meta = info[(row["suite"], row["path"])]
            if row["class"] == "test" and meta["own_collects"] == "1":
                if (row["suite"], row["path"], row["name"]) not in found_tests:
                    count[row["suite"]]["collected module-level tests missed"] += 1
    for (suite, path), meta in info.items():
        if meta["kind"] == "test" and (suite, path) not in picked:
            count[suite]["test files missed"] += 1
            count[suite]["… collected by the suite's own command"] += meta["own_collects"] == "1"
        if meta["kind"] != "test" and meta["own_collects"] == "1" and (suite, path) not in picked:
            count[suite]["other collected files missed"] += 1
    keys = []
    for c in count.values():
        for key in c:
            if key not in keys:
                keys.append(key)
    order = [
        "files picked", "… test files", "… other files", "… … __init__.py", "… … conftest.py",
        "… … other modules", "… files that do not parse", "test files missed",
        "… collected by the suite's own command", "other collected files missed",
        "collected module-level tests missed", "module-level tests found", "… selected", "… refused (one line each)",
        "… … of them a name bound with no def", "… in a collected file, named test",
        "… in a test file no command collects (benchmarks)", "helpers caught as tests", "… a def",
        "… a name bound with no def", "… named test, in a file no command collects, not a test file",
        "methods refused (one line each)", "module faults (one line each)", "check lines",
    ]
    rest = sorted(key for key in keys if key not in order)
    print("count\t" + "\t".join(SUITES) + "\ttotal")
    for key in order + rest:
        values = [count[suite][key] for suite in SUITES]
        print(key + "\t" + "\t".join(str(v) for v in values) + f"\t{sum(values)}")


main()
