# PROTOTYPE #1137: how does each way of decoding a module look to the tester?

Throwaway. This branch is never merged.

`parsing::parse` reads two switches:

- `PURBA_DECODER` picks the way that decodes a module:
  - `utf8` (the default) reads UTF-8 alone, as `main` does.
  - `latin1-ascii`: Rust decodes `latin-1` and `ascii`, and refuses every other declaration.
  - `encoding_rs` decodes by the label that the module declares, with the crate `encoding_rs`.
  - `cpython-codecs`: Rust reads the declaration, and CPython's codec of that name decodes.
  - `cpython`: `importlib.util.decode_source` decodes the bytes.
- `PURBA_REFUSAL` picks the line of a module that the way cannot decode:
  - `invocation` (the default): `purba: invocation error: cannot read …`, exit 3.
  - `project`: `… project error: SyntaxError: …`, exit 4.

## Run it

From the repository root:

```sh
maturin build
maturin build -i python3.14t --out target/wheels-t
crates/parsing/proto-1137-decoding/grid.sh 3.12=<python3.12> 3.13=<python3.13> 3.14=<python3.14> 3.14t=<python3.14t>
```

`corpus.py` writes the modules. `cpython.py` imports each one and calls `test_one`, for CPython's own result.
`grid.sh` writes `target/proto-1137/results.tsv`: one line for each Python, module, way and refusal.
`results.tsv` beside this file is the run on 3.12.14, 3.13.13, 3.14.4 and 3.14.10t.
