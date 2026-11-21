# Parsing refuses a byte that is not UTF-8 in a comment, as CPython 3.14.1 does

## Context and Problem Statement

A module that declares no encoding is UTF-8.
CPython's tokenizer also decodes a module that declares `utf-8` as UTF-8, with no codec.
Rust decodes each such module, and refuses it at its first byte that is not UTF-8.

The import of CPython 3.12, 3.13 and 3.14.0 does not check the bytes of a comment.
It runs a module whose comment holds a byte that is not UTF-8.
CPython 3.14.1 refuses that module.

| fact | source |
|---|---|
| on 3.12 and 3.13, `import` and `compile()` run a module whose comment holds `0xe9`, and pytest 9.2.0 passes its test; on 3.14.4 and 3.14.10t, `import` and `compile()` refuse it | [the probe](https://github.com/kalonji-tools/purba/issues/1138#issuecomment-7063716653) |
| on 3.12 and 3.13, `python test_comment.py` refuses the same module | [the probe](https://github.com/kalonji-tools/purba/issues/1138#issuecomment-7063716653) |
| CPython checks the bytes of a comment from 3.14.1, as the fix of a bug in its support of PEP 263, and the fix did not reach 3.13 | [Non-UTF8 encoding line](https://github.com/python/cpython/issues/63161), [the fix](https://github.com/python/cpython/pull/139481), [the notes of 3.14.1](https://github.com/python/cpython/blob/v3.14.1/Misc/NEWS.d/3.14.1.rst?plain=1#L1801-L1809) |
| CPython's tokenizer decodes `utf-8`, and each name that starts `utf-8-`, with no codec, in any case and with `_` for `-`, and hands each other name to a codec | [`Parser/tokenizer/helpers.c` L306-L331](https://github.com/python/cpython/blob/v3.14.4/Parser/tokenizer/helpers.c#L306-L331), [L418](https://github.com/python/cpython/blob/v3.14.4/Parser/tokenizer/helpers.c#L418) |
| CPython's words name the byte and its line, and its tokenizer ends a line at `\r\n`, `\r` or `\n` | [`helpers.c` L529-L535 at v3.14.1](https://github.com/python/cpython/blob/v3.14.1/Parser/tokenizer/helpers.c#L529-L535), [L236-L239 at v3.14.4](https://github.com/python/cpython/blob/v3.14.4/Parser/tokenizer/helpers.c#L236-L239), [L519-L520 at v3.14.4](https://github.com/python/cpython/blob/v3.14.4/Parser/tokenizer/helpers.c#L519-L520) |
| CPython checks the bytes as it decodes the module, before it reads a token | [`string_tokenizer.c` L61 and L108](https://github.com/python/cpython/blob/v3.14.4/Parser/tokenizer/string_tokenizer.c#L61-L108) |
| CPython 3.14.4 says `but no encoding declared` for a module that declares `utf-8`, and gives a module that declares `utf8`, `u8` or `cp65001` the words of its `utf-8` codec | [the probe](https://github.com/kalonji-tools/purba/issues/1138#issuecomment-7063716653) |
| on `54c3457e`, purba says `stream did not contain valid UTF-8`, and names no byte and no line | [the probe](https://github.com/kalonji-tools/purba/issues/1138#issuecomment-7063716653) |

## Considered Options

- **Parsing asks the CPython of the run whether it checks the bytes of a comment.** On 3.12, 3.13 and 3.14.0, purba then runs the module, as CPython's import does. purba then copies a bug that CPython fixed.
- **Parsing passes a byte that is not UTF-8 in a comment, on each CPython.** On 3.14.1 and later, purba then runs a module that CPython refuses.
- **Parsing keeps the words of Rust's decoder, `stream did not contain valid UTF-8`.** The tester then gets no line and no byte to fix.
- **Each alias of UTF-8 gets the words of CPython 3.14.1.** CPython gives an alias such as `utf8` the words of its codec, so purba's words then differ from CPython's.

## Decision Outcome

**Reach:** `crates/parsing/**`

**Area:** parsing

**Parsing refuses a byte that is not UTF-8, in a comment too, on each supported CPython.**
A module that CPython's tokenizer decodes as UTF-8 is UTF-8 in each of its bytes.
CPython 3.14.1 and later check each byte of such a module, in a comment too.

- **Parsing gives the module a project error, in the words of CPython 3.14.1.** The error names the first byte that is not UTF-8, and its line.
- **A module that declares an alias of UTF-8, such as `utf8`, gets the words of CPython's codec.** CPython's tokenizer hands such a module to the codec of that alias. Parsing hands the bytes to that codec when Rust cannot decode them. [Parsing decodes a module by its declaration, with CPython's codecs](parsing-decodes-a-module-by-its-declaration-with-cpythons-codecs.md#decision-outcome) decodes each other declared encoding the same way.
- **CPython finds such a byte as it decodes the module, before it reads a token.** So the rule of [CPython alone refuses the syntax that its version lacks](cpython-alone-refuses-the-syntax-that-its-version-lacks.md#decision-outcome) does not decide it.

**Downside:**

- **Parsing crosses the seam for a module that declares an alias of UTF-8 and that Rust cannot decode.** [The skeleton crosses the seam in execution, assertion and parsing](the-skeleton-crosses-the-seam-in-execution-and-assertion.md#decision-outcome) names what crosses.
- **On 3.12, 3.13 and 3.14.0, purba refuses a module that CPython imports.** The tester declares the encoding that the byte is in, such as `latin-1`, or fixes the byte.
- **The words say `but no encoding declared` for a module that declares `utf-8`.** CPython 3.14.4 says the same.

## Confirmation

`mise run test:rust` checks the words for a byte that is not UTF-8 in a comment and in a string.
It also checks the line of such a byte after a byte order mark, and after a `\r`.
It checks the codec's words for a module that declares `utf8`, `u8` or `cp65001`.
`mise run test:wheel` checks on each supported CPython that a module whose comment holds `0xe9` gives a project error and the exit code 4.
[Does purba run a module whose comment holds a byte that is not UTF-8, as CPython 3.12 and 3.13 do?](https://github.com/kalonji-tools/purba/issues/1138) holds the measurement.
