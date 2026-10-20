# Does a Graphify graph cut the agent time of a step-3 post pass?

This directory answers [#602](https://github.com/kalonji-tools/purba/issues/602), on the map [#603](https://github.com/kalonji-tools/purba/issues/603).
It is never merged.

## Answer

No. Neither graph meets the bar, so Graphify is **no help to the post pass**.

- The agents did not use the graph. 2 of the 12 graph runs called it, and neither call found a breach that a run with no call missed.
- The link map missed known breaches that the arm as it is today found.
- The concept graph found the same breaches as the arm as it is today. It used 13% fewer cache-read tokens and 8% fewer turns. The bar is 25%.
- The spread between two runs of the same arm is larger than the bar. On #597, the two runs as it is today used 1.28M and 2.04M cache-read tokens.

## The bar

The person fixed it before the runs.
Graphify helps only when both hold:

| condition | link map | concept graph |
|---|---|---|
| at least 25% fewer cache-read tokens, or at least 25% fewer turns | no: −5% and −3% | no: −13% and −8% |
| each known breach that the arm as it is today finds | no: it missed on #594 and #504 | yes |

The build cost of the concept graph also counts against it.
It is in [the build table](#build-cost).

## Breaches found

Each pull request was replayed at the head that its step-3 pass read.
The known breach is the breach that pass found.

| pull request | known breach | as it is today | link map | concept graph |
|---|---|---|---|---|
| #597 at `d46812b` | three new comments give the reason that issue #555 holds | 1 of 2 | 0 of 2 | 1 of 2 |
| #594 at `1f9d948` | a bare `# shellcheck disable=SC2016` at `check-placement.bats:246` | 2 of 2 | 1 of 2 | 2 of 2 |
| #504 at `7beb77b` | two editorconfig-checker exclusions that give no reason | 2 of 2 | 1 of 2 | 2 of 2 |

- Each record that a known breach breaks is on the first list, so every run had it. Each miss is a judgement: some runs saw the line and ruled it admissible, for example "a bare URL is its own reason", and some runs did not flag the line.
- One link-map run on #504 reported a false breach: "seventy-two" in `a-gate-owns-the-mechanical.md:90`. The original pass ruled that a frozen number.
- Three runs reported one more judgement call on #594, of two kinds. Other runs ruled each kind out. [`verdicts.md`](verdicts.md) lists them.
- No run judged a line on the third list stale.

## Cost of a run

The mean of 2 runs. [`runs.tsv`](runs.tsv) holds each run.

| pull request | arm | cache-read tokens | vs today | turns | vs today | wall-clock | graph calls |
|---|---|---:|---:|---:|---:|---:|---:|
| #597 | as it is today | 1.66M | | 20 | | 3:28 | 0 |
| #597 | link map | 0.99M | −40% | 13.5 | −32% | 2:21 | 0 |
| #597 | concept graph | 1.41M | −15% | 18 | −10% | 2:33 | 1 |
| #594 | as it is today | 1.00M | | 15 | | 2:45 | 0 |
| #594 | link map | 1.26M | +26% | 17.5 | +17% | 2:58 | 0 |
| #594 | concept graph | 0.98M | −2% | 15.5 | +3% | 2:05 | 0 |
| #504 | as it is today | 2.28M | | 25 | | 4:19 | 0 |
| #504 | link map | 2.45M | +7% | 27 | +8% | 4:31 | 0 |
| #504 | concept graph | 1.93M | −16% | 22 | −12% | 2:54 | 1 |

| arm, all 6 runs | cache-read tokens | turns |
|---|---:|---:|
| as it is today | 9.89M | 120 |
| link map | 9.41M | 116 |
| concept graph | 8.62M | 111 |

- The link map on #597 is cheaper because both of its runs missed the breach and stopped early. Neither run called the graph.
- The concept arm called the graph twice in 6 runs. So the graph does not cause the difference between the arms. The spread between runs causes it.

## Build cost

| build | wall-clock | tokens |
|---|---:|---|
| link map, `graphify update`, each head | 0.4 to 1.2 s | 0 |
| concept graph, #597 head, 68 records and the template | 8:17 | 176,730 in, 75,177 out |
| concept graph, #594 head | 9:55 | 175,703 in, 87,936 out |
| concept graph, #504 head, 47 records and the template | 6:08 | 135,204 in, 54,779 out |

- The research in #601 inferred about 95,500 input tokens for a build. The measured count is about 1.8 times that. Each `claude -p` call also loads the CLI's own system prompt and the connectors of the account.
- The concept graph holds few concepts: 29 of the 142 nodes at the #597 head. The node of a record holds its links and similar records, and not its rule.

## How it ran

- `graphifyy==0.9.93` in a scratch venv. Each build ran under `env -i` with `HOME` in scratch. `claude -p` read a copy of the credentials there, and wrote only there. The copy was deleted after the builds.
- `mise run list:records` ran once for each head. Each arm got the same lists, in [`lists/`](lists).
- Each run was a fresh Claude Code subagent on Claude Opus 5.5. It did not know the known breach, and it posted nothing. [`prompts/`](prompts) holds each prompt; `<scratch>` stands for the scratch directory.
- [`measure.jq`](measure.jq) reads the cost of one run from its subagent transcript. One API response spans several transcript lines, so it counts each response once.

| file | what it holds |
|---|---|
| `build-concept.sh` | builds the concept graph for each head |
| `bin/graphify` | the scratch `graphify`, with no model and no global write |
| `measure.jq` | the cost of one run |
| `runs.tsv` | the cost of each of the 18 runs |
| `verdicts.md` | the breaches each run reported |

## What it cannot show

- **A small effect.** Two runs for each arm settle a large effect, and the spread here is larger than 25%.
- **Output tokens.** A transcript redacts thinking.
- **Another model or another prompt.** A prompt that tells the agent to use the graph first could change how often it uses the graph. The bar measured the graph as an aid that the agent may use.
- **The transcripts.** They live on one maintainer's machine. `runs.tsv` holds what `measure.jq` read from them.
