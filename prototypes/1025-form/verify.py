"""Compare each probe's docstring with what the runner did, from the summary that `run.py` writes
to FORM_SUMMARY.

    python verify.py <summary.jsonl> <probe file> ...

A docstring's first line holds the expectation: a status for every case of the test (`PASS.`,
`REFUSED: …`), or a status for each named case (`both_ways PASS, one_way REFUSED.`). A status is
PASS, FAIL, REFUSED, BLOCKED or SKIPPED. Each `Says "…"` in the docstring is a text that the run must show
for the test. A module docstring whose first line is `FILE REFUSED` expects the file to be refused.
Exits 1 when any probe does not match.
"""

from __future__ import annotations

import ast
import json
import os
import re
import sys
from pathlib import Path

WORDS = {
    "PASS": "pass",
    "FAIL": "fail",
    "REFUSED": "refused",
    "BLOCKED": "blocked",
    "SKIPPED": "skipped",
}
PAIR = re.compile(r"\b([a-z_][a-z0-9_]*) (PASS|FAIL|REFUSED|BLOCKED|SKIPPED)\b")
SAYS = re.compile(r'Says "([^"]+)"')


def expected(doc: str) -> tuple[dict[str, str] | str | None, list[str]]:
    first = doc.strip().splitlines()[0] if doc.strip() else ""
    says = SAYS.findall(" ".join(doc.split()))
    pairs = PAIR.findall(first)
    if pairs:
        return {case: WORDS[word] for case, word in pairs}, says
    word = re.match(r"(PASS|FAIL|REFUSED|BLOCKED|SKIPPED)\b", first)
    return (WORDS[word.group(1)] if word else None), says


def main(summary: str, files: list[str]) -> int:
    records = [json.loads(line) for line in Path(summary).read_text().splitlines()]
    bad = 0
    for file in files:
        real = os.path.realpath(file)
        tree = ast.parse(Path(file).read_text())
        mine = [r for r in records if os.path.realpath(r["file"]) == real]
        module_doc = ast.get_docstring(tree) or ""
        if module_doc.startswith("FILE REFUSED"):
            got = [r for r in mine if r["status"] == "file refused"]
            text = "\n".join(r["text"] for r in got)
            missing = [s for s in SAYS.findall(" ".join(module_doc.split())) if s not in text]
            ok = bool(got) and not missing
            bad += not ok
            print(f"{'ok ' if ok else 'BAD'} {Path(file).name}: the file is refused", end="")
            print(f"; missing {missing}" if missing else "")
            continue
        for node in tree.body:
            if not (isinstance(node, ast.FunctionDef) and node.name.startswith("test")):
                continue
            want, says = expected(ast.get_docstring(node) or "")
            if want is None:
                continue
            got = [r for r in mine if r["test"] == node.name]
            status = {r["case"]: r["status"] for r in got}
            if isinstance(want, str):
                wrong = {c: s for c, s in status.items() if s != want}
                ok = bool(got) and not wrong
                shown = want.upper()
            else:
                wrong = {c: status.get(c, "absent") for c, s in want.items() if status.get(c) != s}
                ok = not wrong
                shown = ", ".join(f"{c} {s.upper()}" for c, s in want.items())
            text = "\n".join(r["text"] for r in got)
            flat = " ".join(text.split())
            missing = [s for s in says if " ".join(s.split()) not in flat]
            ok = ok and not missing
            bad += not ok
            line = f"{'ok ' if ok else 'BAD'} {Path(file).name}::{node.name}: {shown}"
            if wrong:
                line += f"; got {wrong}"
            if missing:
                line += f"; missing {missing}"
            print(line)
    print(f"{'every probe matches its docstring' if not bad else f'{bad} probe(s) do not match'}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2:]))
