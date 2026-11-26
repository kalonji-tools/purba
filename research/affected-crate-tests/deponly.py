"""Name the failed tests of each mutant that only another crate's tests kill.

usage: python -I deponly.py <rows.tsv> <mutants.out dir>...  > dep-only.tsv

rows.tsv is the output of classify.py. A test's name comes from the
`---- <name> stdout ----` line that libtest prints for a failed test. A test
binary that does not use libtest's harness prints no such line, so its field
is `-` while its target still shows in `killers`.
"""

import json
import re
import sys
from pathlib import Path

rows_path, *roots = sys.argv[1:]
rows = {
    r[1]: r for r in (line.rstrip("\n").split("\t") for line in list(open(rows_path))[1:])
}
print("mutant\tclass\tkillers\tfailed tests")
for root in map(Path, roots):
    for o in json.loads((root / "outcomes.json").read_text())["outcomes"]:
        scenario = o["scenario"]
        if not (isinstance(scenario, dict) and "Mutant" in scenario):
            continue
        r = rows[scenario["Mutant"]["name"]]
        if r[3] not in ("dep-lib", "dep-int", "dep-mixed"):
            continue
        log = (root / o["log_path"]).read_text(errors="replace")
        tests = sorted(set(re.findall(r"^---- (\S+) stdout ----$", log, re.M)))
        print("\t".join([r[1], r[3], r[5], ",".join(tests) or "-"]))
