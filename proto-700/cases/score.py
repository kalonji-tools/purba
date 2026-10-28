"""Grades the blind readers' answers and scores each way, case by case.

usage: python -I -B score.py <readers-dir> <mapping.json> <measure.jsonl>
Rules are fixed before the answers arrive; RATING.md states them.
"""

import json
import re
import sys
from pathlib import Path

BY_TYPE = {"w01_record_690", "w03_alias"}
OVERRIDE_IN_MODULE = BY_TYPE | {"w06_pytest"}
CASE_QUESTIONS = {"c1": ["Q1"], "c2": ["Q2"], "c3": ["Q3"], "c4": ["Q4"], "c5": ["Q5a", "Q5b"], "c6": ["Q6"], "c7": ["Q7"]}

# way-level facts, from runs/cases_checkers.txt and runs/cases_measure.jsonl
CLEAN_PASSES = {w: w != "w09_default_bare" for w in
                ["w01_record_690", "w03_alias", "w06_pytest", "w08_decorator_map", "w09_default_bare",
                 "w10_default_untyped_marker", "w11_default_typed_marker", "w12_annotated_provider"]}
LIE = {  # checkers that report a fixture of the wrong type: None = it cannot be written on the line
    "w01_record_690": None, "w03_alias": None, "w06_pytest": 0, "w08_decorator_map": 0,
    "w09_default_bare": 3, "w10_default_untyped_marker": 0, "w11_default_typed_marker": 3,
    "w12_annotated_provider": 0,
}
GOTO_REACHES_FIXTURE = {w: w not in BY_TYPE for w in CLEAN_PASSES}


def text(value: object) -> str:
    return json.dumps(value).lower() if not isinstance(value, str) else value.lower()


def names(answer: str, name: str) -> bool:
    return re.search(rf"(?<![\w]){name}(?![\w])", answer) is not None


def right(way: str, question: str, answer: object) -> bool | None:
    """True, False, or None where the rule cannot judge (printed for review)."""
    a = text(answer)
    if question in ("Q1", "Q5a"):
        return names(a, "source") and not names(a, "big_source") and not names(a, "target")
    if question == "Q2":
        return names(a, "database")
    if question in ("Q3", "Q4"):
        want = {"client": "client", "db": "database"}
        if question == "Q4":
            want |= {"settings": "settings", "clock": "clock", "mailer": "mailer"}
        if not isinstance(answer, dict):
            return None
        got = {k.lower(): text(v) for k, v in answer.items()}
        return all(k in got and names(got[k], v) for k, v in want.items())
    if question == "Q5b":
        return a.strip().startswith("yes")
    if question == "Q6":
        return bool(re.search(r"\b(2|two|twice)\b", a)) and "sqlite" in a and "postgres" in a
    if question == "Q7":
        rows = bool(re.search(r"\b(3|three)\b", a))
        if way in OVERRIDE_IN_MODULE:
            local = any(k in a for k in ("test_c7", "override", "module", "local", "this file", "same file"))
            return rows and names(a, "source") and local
        return rows and names(a, "big_source")
    raise ValueError(question)


def by_rank(values: dict[str, float], lower_is_better: bool = True) -> dict[str, float]:
    """5 for the best of the eight, 1 for the worst; ties share the mean of their places."""
    order = sorted(values, key=lambda w: values[w], reverse=not lower_is_better)
    places: dict[str, float] = {}
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and values[order[j + 1]] == values[order[i]]:
            j += 1
        for k in range(i, j + 1):
            places[order[k]] = (i + j) / 2
        i = j + 1
    last = len(order) - 1
    return {w: 5 - 4 * p / last for w, p in places.items()}


def main() -> None:
    readers_dir, mapping_path, measure_path = map(Path, sys.argv[1:4])
    mapping = json.loads(mapping_path.read_text())
    measure = [json.loads(line) for line in measure_path.read_text().splitlines()]
    cases = [r for r in measure if "case" in r]
    totals = {r["way"]: r["lines_written_total"] for r in measure if "case" not in r}
    ways = sorted(totals)

    graded: dict[tuple[str, str], list[bool]] = {}
    alone: dict[tuple[str, str], list[bool]] = {}
    ranks: dict[tuple[str, str], list[int]] = {}
    review = []
    for reader, letters in mapping.items():
        result = json.loads((readers_dir / reader / "answers.json").read_text())
        for letter, way in letters.items():
            for case, questions in CASE_QUESTIONS.items():
                for q in questions:
                    entry = result["answers"][letter][q]
                    verdict = right(way, q, entry["answer"])
                    if verdict is None:
                        review.append((reader, way, q, entry["answer"]))
                        verdict = False
                    graded.setdefault((way, case), []).append(verdict)
                    alone.setdefault((way, case), []).append(bool(entry["from_line_alone"]))
        for case, order in result["ranking"].items():
            for place, letter in enumerate(order, 1):
                ranks.setdefault((letters[letter], case), []).append(place)

    lines_score = by_rank(totals)
    out = {"cells": [], "review": review}
    for case in CASE_QUESTIONS:
        row = {r["way"]: r for r in cases if r["case"] == case}
        chars = by_rank({w: row[w]["chars"] for w in ways})
        noise = by_rank({w: row[w]["framework_tokens"] + row[w]["depth"] for w in ways})
        refs = by_rank({w: row[w]["fixture_refs_on_line"] for w in ways})
        mean_rank = {w: sum(ranks[(w, case)]) / len(ranks[(w, case)]) for w in ways}
        for w in ways:
            reader_read = 5 - 4 * (mean_rank[w] - 1) / 7
            readability = reader_read / 2 + chars[w] / 4 + noise[w] / 4
            lie = LIE[w]
            usability = sum([
                5 if CLEAN_PASSES[w] else 1,
                5 if lie is None else 1 + 4 * lie / 3,
                5 if GOTO_REACHES_FIXTURE[w] else 1,
                lines_score[w],
                refs[w],
            ]) / 5
            correct = sum(graded[(w, case)]) / len(graded[(w, case)])
            from_line = sum(alone[(w, case)]) / len(alone[(w, case)])
            clarity = sum([1 + 4 * correct, 1 + 4 * from_line, 5 if lie is None else 1]) / 3
            out["cells"].append({
                "way": w, "case": case,
                "readability": round(readability, 2), "usability": round(usability, 2), "clarity": round(clarity, 2),
                "total": round((3 * readability + 2 * usability + clarity) / 6, 2),
                "mean_reader_rank": round(mean_rank[w], 2), "correct": round(correct, 2), "from_line": round(from_line, 2),
            })
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
