"""Sort the killers of each mutant that cargo-mutants tested.

usage: python -I classify.py <mutants.out dir>...  > rows.tsv

For each mutant, the log of `cargo test --workspace --no-fail-fast` ends with
the list of targets that failed, such as `-p purba-reporting --lib`. Each target
goes in one bucket, by its package and its kind:

  own-lib   a unit test of the edited crate (`--lib`)
  own-int   a test under `crates/<edited>/tests` (`--test <name>`)
  own-doc   a doc test of the edited crate (`--doc`)
  dep-lib   a unit test of another crate (`--lib`)
  dep-int   a test under `crates/<other>/tests`
  dep-doc   a doc test of another crate

A row's `class` names the set of buckets that killed the mutant:

  missed     no Rust test failed
  unviable   the mutant does not build
  timeout    a test ran past the timeout
  own        only the edited crate's tests
  own+dep    the edited crate's tests, and another crate's
  dep-lib    ONLY another crate's unit tests
  dep-int    ONLY another crate's `tests/`
  dep-mixed  only another crate's tests, of two kinds or more
"""

import json
import re
import sys
from pathlib import Path

FAILED = re.compile(r"^error: \d+ targets? failed:$")
TARGET = re.compile(r"^\s+`(-p (\S+) (--lib|--doc|--test \S+|--bin \S+))`")


def killers(log: Path) -> list[tuple[str, str, str]]:
    """Each (target, package, kind) that the log's last test phase names as failed."""
    lines = log.read_text(errors="replace").splitlines()
    out: list[tuple[str, str, str]] = []
    inside = False
    for line in lines:
        if FAILED.match(line):
            inside, out = True, []
            continue
        if inside:
            m = TARGET.match(line)
            if m:
                kind = m.group(3).split()[0]
                out.append((m.group(1), m.group(2), kind))
            else:
                inside = False
    return out


def bucket(edited: str, package: str, kind: str) -> str:
    side = "own" if package == edited else "dep"
    return f"{side}-" + {"--lib": "lib", "--test": "int", "--doc": "doc", "--bin": "bin"}[kind]


def classify(summary: str, buckets: set[str]) -> str:
    if summary == "Unviable":
        return "unviable"
    if summary == "Timeout":
        return "timeout"
    if not buckets:
        return "missed"
    own = {b for b in buckets if b.startswith("own")}
    dep = buckets - own
    if own and dep:
        return "own+dep"
    if own:
        return "own"
    return next(iter(dep)) if len(dep) == 1 else "dep-mixed"


def main() -> None:
    print("crate\tmutant\tsummary\tclass\tbuckets\tkillers")
    for root in map(Path, sys.argv[1:]):
        data = json.loads((root / "outcomes.json").read_text())
        for o in data["outcomes"]:
            scenario = o["scenario"]
            if scenario == "Baseline" or "Mutant" not in scenario:
                continue
            m = scenario["Mutant"]
            found = killers(root / o["log_path"])
            if o["summary"] == "CaughtMutant" and not found:
                sys.exit(f"caught, but no target listed: {o['log_path']}")
            if o["summary"] == "MissedMutant" and found:
                sys.exit(f"missed, but a target listed: {o['log_path']}")
            buckets = {bucket(m["package"], p, k) for _, p, k in found}
            print(
                "\t".join(
                    [
                        m["package"],
                        m["name"],
                        o["summary"],
                        classify(o["summary"], buckets),
                        # `-` for an empty field: the hook that trims trailing
                        # whitespace would drop a trailing tab, and the column.
                        ",".join(sorted(buckets)) or "-",
                        ",".join(t for t, _, _ in found) or "-",
                    ]
                )
            )


main()
