# PROTOTYPE #391: results (throwaway, never merges)

Question: does an agent that sweeps the added lines once for each prose rule
find the prose breaches that one judgement of the whole pull request missed?

Answer: yes, and the sweep for each rule is not what finds them. A single pass
over the added record lines, with the prose rules in front of the agent, finds
the same breaches. The judgement of the whole change misses them.

## What was built

- `build-bundle.sh <base> <head> <out>`: writes what each agent reads, from git
  alone, so `bundles/` is not committed. Run it with base `fcb019a`. The
  pass reads `added.txt`, every line the change adds under `docs/decisions/`
  (the reach of the prose record), and `rules.md`. The judgement reads the
  whole diff, the commit messages and the Decision Outcome of the 20 records
  #388 named.
- `rules.md`: the three prose rules the prose record reports and no command
  refuses (R1 active voice, R2 no wordy word, R3 no part left out), quoted from
  its Decision Outcome. C1, "one idea in one sentence", is a control: only
  `.template.md` states it (#396).
- `sweep-prompt.md`: one rule at a time. `judge-prompt.md`: one judgement of
  the whole change. The control prompt (all rules in one pass) is the sweep
  prompt with its loop replaced by "judge each line against all the rules
  together".
- `score.sh`: counts each run's rows by `data/verdicts.tsv`, and lists each
  known breach (`data/known.tsv`) a run missed.

The judgement prompt is rebuilt: #388 did not keep its own. It still missed
both breaches #388 missed, so the rebuild reproduces #388.

## The known breaches

`5db5182` fixed three prose breaches: "the outcome opened in the passive" and
"two record sentences held two ideas". #374 listed the second as one breach.

| run | head | passive outcome | `, and` join | three clauses |
|---|---|---|---|---|
| sweep, 3 runs | `083508a` | 3 of 3 | 3 of 3 | 3 of 3 |
| sweep, 1 run | `85dfe79` | found | found | found |
| all rules in one pass, 2 runs | `083508a` | 2 of 2 | 2 of 2 | 2 of 2 |
| one judgement | `083508a` | missed | missed | missed |
| one judgement | `85dfe79` | missed | missed | missed |

## What the pass reports beyond the known breaches

Labelled by hand in `data/verdicts.tsv`. `runs/score.tsv` has every run.

| rule | true | false | doubtful | what the false ones are |
|---|---|---|---|---|
| R1 active voice | 3 (`were put to the owner`, `Rejected by the owner`, `is checked by build.yml`) | 4, in 3 of 6 runs | 0 | `Rejected.` and `Chosen.` read as passives |
| R2 no wordy word | 0 | 0 | 1 (`reveals`) | |
| R3 no part left out | 0 | 5, in 6 of 6 runs | 0 | `Rejected.` and `Chosen.` read as fragments |
| C1 one idea (control) | 7 | 0 | 2 | |

- The false reports have one cause. `.template.md` prescribes
  `- **<option>.** <Chosen. Why it won.>`, and `rules.md` did not say so.
- The C1 control finds 7 true joins that no review found, in three records.
  A review found 2 joins in this pull request.

## Cost

`runs/cost.tsv`.

| agent | tokens | seconds |
|---|---|---|
| the prose pass, either prompt (6 runs) | 53,572 to 56,513 | 49 to 69 |
| one judgement (2 runs) | 83,738 to 85,840 | 87 to 106 |

The pass finds no breach that is not prose, so it runs beside the judgement,
not instead of it. In parallel it adds about two thirds to the tokens and no
time.

## Verdict against the falsifiers

| falsifier | result |
|---|---|
| the sweep still misses one of the breaches | does not fire: 0 misses in 6 runs |
| most of the breaches it reports that a review did not find are false | does not fire overall, but R3 is all false: 5 of 5 in every run |
| the reader would not wait for it | does not fire: it is cheaper and shorter than the judgement |
| (added) the sweep for each rule is what finds them | **fires**: one pass with all rules finds the same |

## Limits

- One pull request, 47 to 56 added lines. The cost of the pass grows with the
  added lines, and a larger record change was not measured.
- The verdicts are mine. A second reader did not label them.
