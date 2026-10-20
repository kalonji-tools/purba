# Where a post pass spends its time

This directory backs the "Measured at charting" section of [Wayfinder: a post pass that stays bounded as the records grow](https://github.com/kalonji-tools/purba/issues/603).
It is never merged.

## What it measured

232 Claude Code subagent transcripts, 2026-10-03 to 2026-10-20, whose first prompt asks for a post pass or a step of one.
`files.txt` lists them, relative to `~/.claude/projects`.
They live on one maintainer's machine, so only that machine can run this again.

| kind | runs | total | model turns | share |
|---|---:|---:|---:|---:|
| step 3 | 88 | 527:58 | 479:48 | 91% |
| step 4 | 84 | 207:22 | 178:59 | 86% |
| steps 1 and 3 | 22 | 139:08 | 106:18 | 76% |
| steps 3 and 4 | 9 | 47:04 | 42:38 | 91% |
| steps 1, 3 and 4 | 3 | 17:24 | 15:07 | 87% |
| full, posts the comment | 8 | 52:22 | 44:04 | 84% |
| audit, before `list:records` existed | 18 | 101:26 | 85:44 | 85% |
| **all** | **232** | **1092:44** | **952:39** | **87%** |

| bucket, all runs | time |
|---|---:|
| find a record: `ls`, `grep`, `list:records` | 15:33 |
| read a record | 17:30 |
| read the diff and other files | 57:12 |
| `gh` calls | 30:54 |
| other tools | 3:06 |
| idle, waiting on a message | 15:50 |

## How it attributes time

- A gap that ends at an assistant message is a model turn.
- A gap that ends at a tool result goes to the calls still pending, split equally between parallel calls.
- A Bash command that touches more than one bucket goes whole to one, in this order: `list:records`, a record, `gh`, other. 1178 of 4409 calls were mixed.
- A Read of a file that an earlier call wrote takes that call's bucket.
- Idle is time spent waiting on a message that is not a tool result. It counts in the total.

## What it cannot see

- **Output tokens.** A transcript keeps only the last message's count, and it redacts thinking.
- **Every record read.** A glob read such as `sed … docs/decisions/*.md` names no record, so `records_read` is a lower bound.
- **The parent session.** Step 2 and the posted comment run there, and are not timed.
- **Which transcripts are post passes.** `files.txt` was chosen by reading each first prompt. No script makes that choice.

## Run it

```sh
./run.sh                     # reads ~/.claude/projects
./run.sh <transcripts>       # reads another root
```

It needs `jq`, and prints one row for each kind of run and one for all of them.

| file | what it holds |
|---|---|
| `analyze3.jq` | times one transcript |
| `kind.jq` | names the kind of run from its first prompt |
| `agg3.jq` | sums the runs by kind |
| `names.json` | the record names a Bash command is matched against |
| `per_run.tsv` | one row for each run |
| `results3.jsonl` | every field for each run |
