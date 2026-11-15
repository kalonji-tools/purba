"""Read runs/*_replay.txt and runs/*_tree.txt (#1010).

For each target: the outcome of each case in each mode, the chain cases, and each "diverged on
replay" line, classified as a failed earlier check or real drift. Exits 1 when an outcome differs.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

here = Path(__file__).resolve().parent
runs = here / "runs"
MARK = {"✓": "pass", "✗": "fail", "!": "refused", "⊘": "blocked"}
FOLDERS_980 = (
    "first-attempts/filesink-delay",
    "first-attempts/threading",
    "first-attempts/add-option-catch",
    "regress/filesink-delay",
    "regress/add-option-catch",
    "regress/threading",
    "regress/defaults",
    "regress/colorama",
    "probes",
)
P980 = here.parent / "980-rules-at-run-time"
TARGETS = {  # name in runs/ → the test files it ran
    **{f"980_{d.replace('/', '_')}": sorted((P980 / d).glob("test_*.py")) for d in FOLDERS_980},
    "pipeline": [here / "pipeline" / "test_pipeline.py"],
    "pipeline_part": [here / "pipeline" / "test_part.py"],
    "chains": sorted((here / "chains").glob("test_*.py")),
}


def outcomes(text: str) -> dict[tuple[str, str, str], str]:
    """Each case's status, as the runner's `report` counts it: refused over failed over blocked
    over passed."""
    out: dict[tuple[str, str, str], str] = {}
    rank = ["pass", "blocked", "fail", "refused"]
    file, test = "", ""
    for line in text.splitlines():
        if line.startswith("── "):
            file = line[3:]
        elif m := re.match(r"^ (test\w*)  \(", line):
            test = m.group(1)
        elif m := re.match(r"^   ([✓✗!⊘]) (\S+)", line):
            key, status = (file, test, m.group(2)), MARK[m.group(1)]
            if rank.index(status) > rank.index(out.get(key, "pass")):
                out[key] = status
            out.setdefault(key, status)
    return out


def chain_cases(files: list[Path]) -> int:
    pattern = r"^\w+\s*=\s*case\(.*\.product"
    return sum(len(re.findall(pattern, f.read_text(), re.M)) for f in files)


def divergences(text: str) -> list[str]:
    found = []
    for line in text.splitlines():
        if m := re.search(r"blocked: (\S+) › (.+?) diverged on replay", line):
            stage, check = m.groups()
            own = rf"^   ✗ {re.escape(stage)} › {re.escape(check)}"
            kind = "a failed earlier check" if re.search(own, text, re.M) else "REAL DRIFT"
            found.append(f"{line.strip()[:90]}  →  {kind}")
    return found


def main() -> int:
    worst = 0
    for name, files in TARGETS.items():
        replay = (runs / f"{name}_replay.txt").read_text()
        tree = (runs / f"{name}_tree.txt").read_text()
        a, b = outcomes(replay), outcomes(tree)
        differ: list[object] = sorted(k for k in a.keys() | b.keys() if a.get(k) != b.get(k))
        last_r, last_t = replay.splitlines()[-1], tree.splitlines()[-1]
        if last_r != last_t:  # the count also holds the refusals that no case line names
            differ.append(("the count", last_r, last_t))
        worst |= bool(differ)
        total = re.search(r"══ (\d+) case", replay)
        cases = total.group(1) if total else "?"
        verdict = f"differ: {differ}" if differ else "the same"
        chains = chain_cases(files)
        print(f"{name:<36} {cases:>3} cases, {chains:>2} chain cases   outcomes {verdict}")
        for mode, text in (("replay", replay), ("tree", tree)):
            for d in divergences(text):
                print(f"    {mode:<6} {d}")
    return worst


if __name__ == "__main__":
    sys.exit(main())
