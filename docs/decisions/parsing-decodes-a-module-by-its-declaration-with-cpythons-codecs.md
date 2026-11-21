# Parsing decodes a module by its declaration, with CPython's codecs

## Context and Problem Statement

A module can declare its encoding on its first or second line.
CPython decodes it in any text encoding that Python knows.
Parsing read UTF-8 alone, and refused each module that declares another encoding.

| fact | source |
|---|---|
| a comment on line 1 or 2 declares the encoding, and CPython decodes the module in it | [the language reference](https://docs.python.org/3.12/reference/lexical_analysis.html#encoding-declarations) |
| CPython gives a `SyntaxError` for an unknown name, for `rot13`, and for a byte that the codec refuses | [the probe of the pull request](https://github.com/kalonji-tools/purba/pull/1139#issuecomment-7061183650) |
| CPython's codecs agree with CPython on 10 of 11 modules on 3.12, and on 11 of 11 on 3.14 | [the prototype](https://github.com/kalonji-tools/purba/issues/1137#issuecomment-7060907518) |
| Rust's own decoders of `latin-1` and `ascii` agree with CPython on 8 of 11 modules on 3.12, and on 9 of 11 on 3.14, and refuse `cp1252` and `euc_jp` | [the prototype](https://github.com/kalonji-tools/purba/issues/1137#issuecomment-7060907518) |
| `encoding_rs` knows no label `latin-1` or `euc_jp`, and reads `iso-8859-1` as windows-1252 | [the prototype](https://github.com/kalonji-tools/purba/issues/1137#issuecomment-7060907518) |
| `importlib.util.decode_source` says `encoding problem: utf-8` for a byte order mark before a declaration of `latin-1`, where CPython names `iso-8859-1` | [the prototype](https://github.com/kalonji-tools/purba/issues/1137#issuecomment-7060907518) |
| 3.12 and 3.13 run a module whose comment holds a byte that is not UTF-8, and 3.14 refuses it | [the prototype](https://github.com/kalonji-tools/purba/issues/1137#issuecomment-7060907518) |

## Considered Options

- **`importlib.util.decode_source` on the bytes.** It gives the results of the codecs. For a byte order mark before another declaration, it gives the wrong words.
- **Rust decodes `latin-1` and `ascii` itself.** It refuses `cp1252` and `euc_jp`, which CPython runs. It copies a list of names that CPython keeps.
- **`encoding_rs`.** It refuses the label `latin-1`. It passes a module that CPython refuses.
- **UTF-8 alone.** A module that declares another encoding cannot run.
- **A module that its encoding cannot decode is an invocation error.** The tester fixes it in the module, and not in the command.
- **Command hands parsing a decoder from execution.** Parsing then holds no Python, and execution, which does not decode, crosses for it.
- **The codec decodes every module, a UTF-8 module too.** Each module then costs a call into Python.

## Decision Outcome

**Reach:** `crates/parsing/**` `crates/command/**`

**Area:** parsing

**Parsing decodes a module by its declaration, with CPython's codecs.**
Rust reads the declaration.
Where it names an encoding other than UTF-8, parsing hands the bytes to the text codec of that name.

- **Rust decodes a module with no declaration, or one that declares UTF-8.**
- **A module that its encoding cannot decode is a project error.** CPython gives it a `SyntaxError`.

| a module that its encoding cannot decode | an example |
|---|---|
| it names no codec | `# coding: foobar` |
| it names a codec that is not a text encoding | `# coding: rot13` |
| the codec refuses a byte | `0x81` under `# coding: cp1252` |
| a byte order mark comes before a declaration other than UTF-8 | a byte order mark, then `# coding: latin-1` |
| it declares no encoding, and a byte is not UTF-8 | `0xff` |

**Downside:**

- **Parsing crosses the seam.** [The skeleton crosses the seam in execution, assertion and parsing](the-skeleton-crosses-the-seam-in-execution-and-assertion.md#decision-outcome) names what crosses.
- **purba refuses a module whose comment holds a byte that is not UTF-8, which CPython 3.12 and 3.13 run.** [Does purba run a module whose comment holds a byte that is not UTF-8, as CPython 3.12 and 3.13 do?](https://github.com/kalonji-tools/purba/issues/1138) holds it.
- **The line of the project error names the encoding as the module declares it.** For a byte order mark before a declaration of `latin-1`, purba says `latin-1 with BOM`, and CPython says `iso-8859-1 with BOM`.

## Confirmation

`mise run test:rust` checks the text of a module for each of `latin-1`, `iso-8859-1`, `cp1252` and `euc_jp`.
It also checks the error of each module that its encoding cannot decode.
`mise run test:wheel` checks on each supported CPython that a module that declares `latin-1` runs its test.
It also checks that a byte order mark before a declaration of `latin-1`, and the unknown encoding `foobar`, each give a project error.

[How does each way of decoding a module look to the tester?](https://github.com/kalonji-tools/purba/issues/1137) holds the measurement.
