"""Count, for each reading of the criteria, the mutants that no selected test kills.

usage: python -I readings.py <rows.tsv> <bats.tsv> <mutants.out dir>...

rows.tsv is the output of classify.py. bats.tsv has one line per mutant that
bats.sh ran: the name of its diff, `exit=N` and `not-ok=N`. A mutant that build
of the wheel or bats did not reach has no line, and counts as not run.
Only the mutants that build count. A timeout counts as caught by every reading
that runs the Rust tests, since a run past the timeout fails too.
"""

import json
import sys
from pathlib import Path

rows_path, bats_path, *roots = sys.argv[1:]
rows = [line.rstrip("\n").split("\t") for line in open(rows_path)][1:]

diff_of = {}
for root in map(Path, roots):
    for o in json.loads((root / "outcomes.json").read_text())["outcomes"]:
        scenario = o["scenario"]
        if isinstance(scenario, dict) and "Mutant" in scenario:
            diff_of[scenario["Mutant"]["name"]] = Path(o["diff_path"]).stem

bats = {}
for line in open(bats_path):
    parts = line.rstrip("\n").split("\t")
    if len(parts) >= 3 and parts[2].startswith("not-ok="):
        bats[parts[0]] = int(parts[2].removeprefix("not-ok=")) > 0

viable = [r for r in rows if r[3] != "unviable"]


def through(keep: set[str], with_bats: bool) -> tuple[int, int]:
    """(let through, not run under bats) for a reading that runs the buckets in `keep`."""
    out = unknown = 0
    for r in viable:
        if r[3] == "timeout":
            continue
        if set(r[4].split(",")) & keep:
            continue
        if with_bats:
            killed = bats.get(diff_of[r[1]])
            if killed is None:
                unknown += 1
                continue
            if killed:
                continue
        out += 1
    return out, unknown


OWN = {"own-lib", "own-int", "own-doc"}
DEP_INT = {"dep-int", "dep-doc"}
readings = [
    ("today: every Rust test, and bats", OWN | DEP_INT | {"dep-lib"}, True),
    ("every Rust test, no bats", OWN | DEP_INT | {"dep-lib"}, False),
    ("strict criterion 3: the edited crate's tests, dependents' `tests/`, and bats", OWN | DEP_INT, True),
    ("strict criterion 3, no bats", OWN | DEP_INT, False),
    ("loose criterion 3: the edited crate's tests, and bats", OWN, True),
    ("the edited crate's tests alone", OWN, False),
]
print(f"viable mutants: {len(viable)}")
print("reading\tlet through\tnot run under bats")
for name, keep, with_bats in readings:
    n, unknown = through(keep, with_bats)
    print(f"{name}\t{n}\t{unknown}")
