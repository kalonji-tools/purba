You are a reader in a readability study of a Python test framework that does not exist yet. You did not write any of this code, and you know nothing about it beyond what the files show. That is the point: we want a cold reader.

## The material

Your directory is `{DIR}`. It holds eight folders, `A` to `H`. Each folder is one **style** of writing the same tests. In every folder:

- `subject.py` is the code under test. It is the same in every folder.
- `sketch.py` is the framework's helpers. It is the same in every folder; a style uses only some of them.
- `fixtures.py` holds the style's fixtures: functions that build the values that tests take.
- `test_c1_copy.py` … `test_c7_override.py` are seven test modules. Their bodies are identical across styles. Only how a test or fixture says *which fixture fills each parameter* differs.

Rules:

- Read only files inside `{DIR}`. Do not open anything outside it, do not use git, and do not search the rest of the disk.
- Answer by reading. Do not run, import or type-check any file.
- If a style leaves an answer unclear, say so. Do not guess silently: an honest "unclear" is worth more to the study than a lucky guess.

## The questions

Answer every question for every style, A to H.

- **Q1** (`test_c1_copy.py`): In `test_copy`, which fixture function builds the database that `src` works on?
- **Q2** (`test_c2_one_input.py`): In `test_insert`, which fixture function builds the database that `db` works on?
- **Q3** (`test_c3_two_types.py`): In `test_fetch`, which fixture function builds the value of each parameter? For a transaction, name the fixture that builds its database.
- **Q4** (`test_c4_five_inputs.py`): In `test_signup`, which fixture function builds the value of each parameter? Same rule for a transaction.
- **Q5a** (`test_c5_fixture_chain.py`): Which fixture function builds the database that `seeded` writes into?
- **Q5b**: Within one run of `test_copy_seeded`, is the `src` that `seeded` writes to the same transaction object as the test's own `src`? Answer yes, no, or unclear.
- **Q6** (`test_c6_many_providers.py`): How many times does `test_roundtrip` run, and on which databases?
- **Q7** (`test_c7_override.py`): In `test_copy_twice`, which fixture function builds the database that `src` reads, and how many rows does that database hold when the test starts?

For each answer, also report:

- `from_line_alone`: `true` only if you could answer from the test's (or fixture's) own signature, meaning its decorators and its `def` line, without opening another function or file. Otherwise `false`.
- `looked_at`: the files and function names you had to read beyond the signature, e.g. `["fixtures.py:source", "sketch.py:using"]`. Empty if none.
- `confidence`: 1 (a guess) to 5 (certain).

## The ranking

After answering, rank the eight styles for **readability**, once for each of the seven test modules. Readability here means: how fast a reader who did not write the test can say what the test takes. Rank fastest first. Ties are not allowed.

Then give each style one sentence: what helped you read it, or what slowed you down.

## Output

Write your result as JSON to `{DIR}/answers.json`, and also return the same JSON as your final message. Use this shape:

```json
{
  "answers": {
    "A": {
      "Q1": {"answer": "...", "from_line_alone": true, "looked_at": [], "confidence": 5},
      "Q2": {...}, "Q3": {...}, "Q4": {...}, "Q5a": {...}, "Q5b": {...}, "Q6": {...}, "Q7": {...}
    },
    "B": {...}
  },
  "ranking": {"c1": ["C", "F", ...], "c2": [...], "c3": [...], "c4": [...], "c5": [...], "c6": [...], "c7": [...]},
  "notes": {"A": "...", "B": "..."}
}
```

Q3 and Q4 answers may be an object that maps each parameter to a fixture name.
