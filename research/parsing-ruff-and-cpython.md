# Where ruff's parser and CPython disagree about whether a module parses

Research for [Where do ruff's parser and CPython disagree about whether a module parses, on each CPython that purba supports?](https://github.com/kalonji-tools/purba/issues/1127)
Measured on 2026-11-20, against purba `main` at `e1490a30`, on one Linux machine.
This file decides nothing. It lists what disagrees, and why.

## Summary

purba reads a test module, and ruff's parser parses it, before CPython ever sees it. I looked for modules where the two sides give a different verdict. I found 7 kinds of module that purba refuses and CPython runs. I found 11 kinds that ruff passes and CPython refuses at compile time. One of those 11 is a family: CPython's compiler and symbol table refuse a module for 64 distinct reasons, and ruff's parser checks none of them.

For purba's risk, "parsing refuses a file that CPython runs", the 7 kinds are the ones that matter. Most are rare in real code, but four can appear in an ordinary module:

- A file that declares an encoding other than UTF-8 is refused by purba's own reader, before ruff. This holds even when every byte of the file is ASCII, as in `# coding: latin-1` over plain code, or `# coding: ascii`. Every CPython runs it.
- `match` used as a name after a semicolon, as in `m = 1; match = 2`, is refused by ruff. Every CPython runs it.
- A bare `raise` before a semicolon, as in `raise; x = 1`, is refused by ruff. Every CPython runs it.
- Mixed tabs and spaces can go either way. ruff counts a tab as 2 columns and CPython as 8, so each refuses some indentation the other accepts.

ruff fixed the semicolon cases after the pinned version, in ruff 0.17.4. The rest are on 3.12 or 3.13 only, or need an April-fool `__future__` import.

The 7 kinds of direction A are in sections 1.1, 1.2, 2.1, 2.6, 3.3, 3.4 and 3.5. The 11 kinds of direction B are in 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.5, 4 and 4.1. Sections 2.1 and 3.5 hold one kind of each direction.

The other direction is large but safe in one sense. CPython still refuses the module when purba loads it, and purba then reports CPython's own error, with its file and line. The cost is that "ruff parsed it" never means "it will load". The largest cause is version-dependent syntax. purba gives ruff no target version and ignores ruff's version errors, so ruff accepts 3.13, 3.14 and even 3.15 syntax on every CPython. A t-string passes ruff and fails on 3.12 and 3.13, for example.

## How to read this file

- **[run]** means I ran it in this session, on the version named. **[source]** means a source says so, and I did not run it.
- **"ruff"** in a result means purba's reading and parsing, `purba_parsing::parse`. That is purba's own reader, then ruff 0.17.0's parser. Its verdict is the same on every CPython, because ruff is compiled into purba.
- **"refuses"** for CPython means `compile()` raises at compile time: a `SyntaxError`, `IndentationError`, `TabError`, or a decode error. Three other exceptions also appear at compile time, and I list them because a load fails the same way: `ValueError`, `MemoryError` and `RecursionError`.
- **Direction A**: purba or ruff refuses, CPython runs. **Direction B**: ruff passes, CPython refuses.
- A **kind** is one cause. Each kind has a minimal module. When the bytes matter, the module is a Python `bytes` literal.

## Method

### The two sides

**The ruff side [run].** The crate `ruff_python_parser 0.0.17` was published from ruff commit `5b27c1ccfaa230fb195855f5cf3404a21e3379cd`, says its `.cargo_vcs_info.json`. That commit is the tag [`0.17.0`](https://github.com/astral-sh/ruff/tree/0.17.0), "Bump version to 0.17.0 (#29259)", 2026-10-09. The checksum in purba's `Cargo.lock` is `37a22811…6640fbb`, the same as crates.io's. Every ruff link below is at that tag.

I called `purba_parsing::parse` from an uncommitted example in `crates/parsing/examples/`. I checked the important results again with the purba wheel built from `e1490a30`. That wheel runs on CPython 3.12.14. When ruff refuses, it prints `project error: SyntaxError: <ruff's words>` with no file and line. When ruff passes and CPython refuses, it prints CPython's words with `(file, line N)`, and the tests are `blocked`.

**The CPython side [run].** Four interpreters, each with `-I`:

| label | build | from |
|---|---|---|
| 3.12 | 3.12.14 | mise |
| 3.13 | 3.13.17 | nixpkgs (mise could not build it: no `make`) |
| 3.14 | 3.14.8, with the GIL | nixpkgs |
| 3.14t | 3.14.7, free-threaded (`Py_GIL_DISABLED` = 1) | mise |

Each module was compiled with `compile(source_bytes, path, "exec", dont_inherit=True)`. That is the call importlib makes when it imports a source file. For the encoding cases I also imported each file for real. The results matched `compile()`.

Sources are cited at the tags `v3.12.14`, `v3.13.17` and `v3.14.8`. The 3.14t interpreter is 3.14.7, one release older. In every probe, 3.14 and 3.14t gave the same verdict.

### Three ways of finding cases

1. **A hand corpus of 672 modules [run].** I collected CPython's sources of compile-time errors, and wrote at least one module for each:
   - the messages of the tokenizer (`Parser/lexer/lexer.c`, `Parser/tokenizer/*.c`; `Parser/tokenizer.c` at 3.12),
   - the 149 lines of `Grammar/python.gram` that raise an error, most in `invalid_*` rules,
   - `Parser/pegen.c`, `Parser/action_helpers.c` and `Parser/string_parser.c`,
   - `Python/symtable.c`, `Python/future.c`, and `Python/compile.c` (plus `Python/codegen.c` at 3.14).

   I added a module for each kind in ruff's own lists: its parse errors, its lexical errors, its version checks (`UnsupportedSyntaxErrorKind`) and its semantic errors (`SemanticSyntaxErrorKind`).
2. **A differential fuzz of 6,660,847 generated modules [run].** Each one went to ruff and to `ast.parse` on all four CPythons. `ast.parse` runs only the tokenizer and the grammar, so any disagreement it shows is a tokenizer or grammar disagreement. The generators:
   - every number-like token up to 4 characters from `019_.eEjJxXoObBaf+-` (28,960 modules),
   - string prefixes of up to 2 letters with 13 bodies (5,772),
   - every ASCII escape in 6 kinds of string (4,512),
   - random f-strings and t-strings (237,457),
   - random mixed indentation with `\n`, `\r\n` and `\r` (209,375),
   - random sequences of up to 7 tokens, at module level and inside `async def` (6,174,587, in six runs).
3. **Every Unicode code point [run].** For each of the 1,112,062 scalar values except `\n` and `\r`, three modules: `<c>=1`, `a<c>=1` and `a =<c>1`. Each went to ruff and to `compile()` on all four.

## How purba calls ruff

purba calls [`parse_unchecked_source(source, PySourceType::Python)`](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/lib.rs#L293-L300) and keeps the first entry of [`errors()`](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/lib.rs#L405-L413) ([purba `lib.rs`](https://github.com/kalonji-tools/purba/blob/e1490a302ad1abc913e252dba7bc626cf88dd9d5/crates/parsing/src/lib.rs#L376-L406)). Three facts follow, and they explain most of direction B.

1. **No target version.** `ParseOptions::from(PySourceType)` sets the [default target](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/parser/options.rs#L51-L58), which is [Python 3.11](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_ast/src/python_version.rs#L99-L103) [source].
2. **Version errors are ignored.** ruff's parser always accepts the newest syntax it knows. When the syntax is newer than the target, it records an entry in `unsupported_syntax_errors()`, a list apart from `errors()`. purba never reads that list. So the target version changes nothing for purba [source, and run: every version case below passes ruff].
3. **No semantic checks.** ruff checks scope rules, such as `return` outside a function, in a separate [`SemanticSyntaxChecker`](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/semantic_errors.rs#L19). purba never runs it [source].

Before ruff, purba's own reader [refuses a declared encoding other than UTF-8 and any byte that is not UTF-8, and drops a leading byte order mark](https://github.com/kalonji-tools/purba/blob/e1490a302ad1abc913e252dba7bc626cf88dd9d5/crates/parsing/src/lib.rs#L216-L237). Its refusal is an invocation error, exit 3, not a project error.

## 1. Encoding and decoding

### 1.1 A file that declares an encoding other than UTF-8 (A)

Minimal modules:

```python
b"# -*- coding: latin-1 -*-\nx = 1\n"       # every byte is ASCII
b"# coding: ascii\nx = 1\n"                  # ASCII is a subset of UTF-8
b"# -*- coding: latin-1 -*-\nx = '\xe9'\n"   # a Latin-1 byte
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| each module above | refuses | runs | runs | runs | runs |

All [run], with a real import too. purba's words: `invocation error: cannot read X: it declares the encoding latin-1, and purba reads only UTF-8`. I saw the same with `cp1252`, `iso-8859-15`, `euc_jp`, a declaration on line 2 after a shebang or a blank line, and `\r` line ends.

**Why.** This is purba's reader, not ruff. ruff takes only a Rust `&str`, which is UTF-8. CPython decodes the file with the declared codec ([encoding declarations](https://docs.python.org/3.12/reference/lexical_analysis.html#encoding-declarations), [PEP 263](https://peps.python.org/pep-0263/)). Open question [#644](https://github.com/kalonji-tools/purba/issues/644).

**Agreements [run].** Both sides pass a UTF-8 byte order mark ([#631](https://github.com/kalonji-tools/purba/issues/631)), `utf-8`, `utf8`, `u8`, `utf-8-sig`, and `# vim: set fileencoding=utf-8 :`. Both refuse a byte order mark with `# coding: latin-1`, an unknown codec, UTF-16, an encoded surrogate, and invalid UTF-8 in code. A declaration that CPython ignores is ignored by purba too: on line 3, after a code line, or after code on the same line.

### 1.2 A byte that is not UTF-8, inside a comment, with no declaration (A, 3.12 and 3.13)

```python
b"# \xe9\nx = 1\n"
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| | refuses | runs | runs | refuses | refuses |

All [run], with a real import too. purba: `stream did not contain valid UTF-8`. 3.14: `Non-UTF-8 code starting with '\xe9' on line 1, but no encoding declared`.

**Why.** Up to 3.13, the string tokenizer that `compile()` uses checks UTF-8 only in the tokens it reads, and a comment is not one ([v3.13.17 `string_tokenizer.c`](https://github.com/python/cpython/blob/v3.13.17/Parser/tokenizer/string_tokenizer.c#L98-L104)). 3.14 checks the whole input ([v3.14.8 `string_tokenizer.c`](https://github.com/python/cpython/blob/v3.14.8/Parser/tokenizer/string_tokenizer.c#L101-L110)). The change is [gh-63161](https://github.com/python/cpython/issues/63161), backported to 3.14 in commit [`9ff705c6`](https://github.com/python/cpython/commit/9ff705c6c59a7561b065639ebb07f3341083e869): "Detect decoding error in comments for UTF-8 encoding" [source].

### 1.3 A null byte inside a comment (B)

```python
b"# a\x00b\nx = 1\n"
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| | passes | refuses | refuses | refuses | refuses |

All [run]. CPython: `source code string cannot contain null bytes`, raised before tokenizing ([v3.12.14 `pythonrun.c`](https://github.com/python/cpython/blob/v3.12.14/Python/pythonrun.c#L1906), [v3.14.8 `pythonrun.c`](https://github.com/python/cpython/blob/v3.14.8/Python/pythonrun.c#L1647)). ruff keeps the byte as part of the comment. A null byte in code, `x = 1\x00`, is refused by both.

## 2. Tokenizer

### 2.1 Mixed tabs and spaces (A and B)

ruff and CPython measure a tab differently.

- CPython counts a tab to the next multiple of 8. It also counts each indent a second time, with a tab as 1 column. A dedent must match an earlier indent in the first count. A change of level must agree in both counts, or it is a `TabError` ([v3.14.8 `lexer.c`](https://github.com/python/cpython/blob/v3.14.8/Parser/lexer/lexer.c#L470-L570), with [`TABSIZE` 8](https://github.com/python/cpython/blob/v3.14.8/Parser/lexer/state.c#L9) and [`ALTTABSIZE` 1](https://github.com/python/cpython/blob/v3.14.8/Parser/lexer/lexer.c#L10); the same at [v3.12.14](https://github.com/python/cpython/blob/v3.12.14/Parser/tokenizer.c#L1767) and [v3.13.17](https://github.com/python/cpython/blob/v3.13.17/Parser/lexer/lexer.c#L431)). The [reference](https://docs.python.org/3.12/reference/lexical_analysis.html#indentation) describes the 8-column rule.
- ruff counts a tab to the next multiple of **2**, and counts characters. The two counts must order the same way, or it is an error ([`indentation.rs`](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/lexer/indentation.rs#L37-L80)) [source].

So when the 2-column and the 8-column counts disagree, so do the parsers. The fuzz found 1,674 such modules in one run, both ways [run]. Minimal ones, each with `\n` line ends:

| module | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| `b"if x:\n \t \tpass\n  \t\tpass\n"` | refuses: unindent does not match | runs | runs | runs | runs |
| `b"if x:\n \tpass\n  pass\n"` | passes | IndentationError: unindent does not match | same | same | same |
| `b"if x:\n\t  pass\n  \tpass\n"` | passes | IndentationError: unindent does not match | same | same | same |
| `b"if x:\n  pass\n \tpass\n"` | passes | TabError | same | same | same |

All [run]; the first and second were also checked with the wheel. In the first module, both lines are 16 columns wide to CPython and 4 characters long. To ruff, they are 4 and 6 columns wide. In the second, `" \t"` is 8 columns to CPython and 2 to ruff, and `"  "` is 2 to both.

The fuzz found no other indentation disagreement. A form feed resets the count on both sides. A backslash continuation inside indentation agreed in the seven modules I wrote for it, including the cases in ruff's own tests and [cpython#90249](https://github.com/python/cpython/issues/90249). I found no ruff issue about the tab width.

### 2.2 Identifiers with characters newer than CPython's Unicode database (B)

```python
"\U00010d4a = 1\n"   # GARAY LETTER, Unicode 16.0
"\U0002ebf0 = 1\n"   # a CJK ideograph of Extension I, Unicode 15.1
"a\u200c = 1\n"      # ZERO WIDTH NON-JOINER, an identifier character since 15.1
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| Garay (16.0) | passes | refuses | refuses | runs | runs |
| Extension I (15.1) | passes | refuses | runs | runs | runs |
| ZWNJ inside a name | passes | refuses | runs | runs | runs |
| any letter new in Unicode 17.0 | passes | refuses | refuses | refuses | refuses |

All [run]. CPython: `invalid non-printable character U+10D4A`.

Every code point was tested. In no position does ruff refuse a character that a CPython accepts. Each CPython refuses some characters that ruff accepts. The count of characters that ruff accepts and CPython refuses, by position:

| | at the start of a name | inside a name |
|---|---|---|
| 3.12 | 9,571 | 9,758 |
| 3.13 | 8,949 | 9,132 |
| 3.14 and 3.14t | 4,647 | 4,699 |

Every one of these is a code point that the CPython's own database calls unassigned (`Cn`), except four on 3.12. Those four are U+200C, U+200D, U+30FB and U+FF65. They exist in 15.0, and became identifier characters in 15.1 [run].

**Why.** ruff checks a name with the `unicode-ident` crate, `is_xid_start` and `is_xid_continue` ([`lexer.rs`](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/lexer.rs#L1597-L1619)). purba's `Cargo.lock` picks `unicode-ident` 1.0.24, whose tables are [Unicode 17.0.0](https://github.com/dtolnay/unicode-ident/blob/1.0.24/src/tables.rs#L12). CPython checks the same two properties with its own database ([v3.14.8 `verify_identifier`](https://github.com/python/cpython/blob/v3.14.8/Parser/lexer/lexer.c#L322-L366)). That database is [15.0 on 3.12](https://docs.python.org/3.12/library/unicodedata.html), [15.1 on 3.13](https://github.com/python/cpython/blob/v3.13.17/Doc/whatsnew/3.13.rst#L1423-L1426) and [16.0 on 3.14](https://github.com/python/cpython/blob/v3.14.8/Doc/whatsnew/3.14.rst#L2045-L2048). I also read it with `unicodedata.unidata_version` on each [run].

The set depends on purba's lock file, not on ruff's pin. A `cargo update` of `unicode-ident` can widen it.

### 2.3 `\N{name}` escapes with names newer than CPython's database (B)

```python
'x = "\\N{GARAY CAPITAL LETTER A}"\n'     # Unicode 16.0
'x = "\\N{CJK UNIFIED IDEOGRAPH-2EBF0}"\n' # Unicode 15.1
'x = "\\N{SIDETIC LETTER N01}"\n'         # Unicode 17.0
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| Garay (16.0) | passes | refuses | refuses | runs | runs |
| Extension I (15.1) | passes | refuses | runs | runs | runs |
| Sidetic (17.0) | passes | refuses | refuses | refuses | refuses |

All [run]. CPython: `(unicode error) 'unicodeescape' codec can't decode bytes in position 0-25: unknown Unicode character name`.

**Why.** ruff looks names up in `ruff_unicode_names2` ([`string.rs`](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/string.rs#L206-L235)), which holds [Unicode 17.0 names](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_unicode_names2/README.md#L6). CPython uses its own database, as in 2.2.

**Agreements [run].** Both accept lower-case names (`\N{digit one}`), aliases (`\N{NBSP}`), and computed names for CJK, Hangul, Tangut and Khitan. Both refuse named sequences, `\N{}`, `\N`, and names padded with spaces. In 4,512 fuzzed escapes, the only disagreements were t-strings (see 3.1).

### 2.4 A name whose NFKC form is `None`, `True`, `False` or `__debug__` (B)

```python
"\U0001d40d\U0001d428\U0001d427\U0001d41e = 1\n"   # bold "None" = 1
"x.\U0001d413rue = 1\n"                            # x.<bold T>rue = 1
"__\U0001d41debug__ = 1\n"                          # __<bold d>ebug__ = 1
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| bold `None` | passes | runs | refuses: `ValueError` | refuses: `ValueError` | refuses: `ValueError` |
| `x.` bold `True` | passes | runs | refuses: `ValueError` | refuses: `ValueError` | refuses: `ValueError` |
| bold `__debug__` | passes | refuses | refuses | refuses | refuses |

All [run]. CPython normalizes each name to NFKC first. 3.13 and 3.14 then raise `ValueError: identifier field can't represent 'None' constant` ([v3.13.17 `pegen.c`](https://github.com/python/cpython/blob/v3.13.17/Parser/pegen.c#L540-L555), [gh-133516](https://github.com/python/cpython/issues/133516), backported in [`a719e5fb`](https://github.com/python/cpython/commit/a719e5fbb63346c751fc9dfbc62af32215f6c0be)). The `__debug__` rule is in 4. ruff keeps the name as written.

### 2.5 The tokenizer's fixed limits (B)

| module | ruff | every CPython [run] | CPython's limit [source] |
|---|---|---|---|
| 101 nested indented blocks | passes | IndentationError: too many levels of indentation | `MAXINDENT` 100 |
| `x = ` and 201 nested brackets | passes | too many nested parentheses | `MAXLEVEL` 200 |
| 150 nested f-strings, `f"{f"{…}"}"` | passes | too many nested f-strings | `MAXFSTRINGLEVEL` 150 |

The limits are in [v3.12.14 `tokenizer.h`](https://github.com/python/cpython/blob/v3.12.14/Parser/tokenizer.h#L13-L15) and [v3.14.8 `state.h`](https://github.com/python/cpython/blob/v3.14.8/Parser/lexer/state.h#L6-L8). 100 levels and 200 brackets pass on both sides; 149 nested f-strings pass on 3.12 and 3.14 [run]. ruff has no such limit.

### 2.6 A newline inside the format spec of a single-quoted f-string (A, 3.12)

```python
'x = f"{x:\n}"\n'
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| | refuses | runs | refuses | refuses | refuses |

All [run], and checked with the wheel on 3.12. ruff: `f-string: newlines are not allowed in format specifiers when using single quotes`. CPython 3.13 and 3.14: the same rule ([v3.13.17 `lexer.c`](https://github.com/python/cpython/blob/v3.13.17/Parser/lexer/lexer.c#L1346)). CPython added it in [gh-129958](https://github.com/python/cpython/issues/129958), backported to 3.13 in [`a37d719d`](https://github.com/python/cpython/commit/a37d719d37ba0f45d0e88f1f70ccd66f2a4f209e), but not to 3.12. ruff applies it to every target version ([`lexer.rs`](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/lexer.rs#L837-L845), from [ruff#18632](https://github.com/astral-sh/ruff/issues/18632)).

### 2.7 Tokenizer agreements

- **Numbers [run].** All 28,960 number-like tokens agreed: underscores, leading zeros, bases, exponents, imaginary parts, and a number touching a keyword (`1if`, `0x1for`, `1_e5`).
- **Strings [run].** All prefixes agreed, except `t` before 3.14 (3.1). Bytes with a non-ASCII character, truncated `\x`, `\u` and `\U`, and code points past U+10FFFF are refused by both.
- **Lines [run].** `\r`, `\r\n` and mixed line ends agreed. So did a `\r` inside a string or a comment, which both treat as a line end. A backslash before end of file, before a blank line, or before a space is refused by both.
- **Odd characters [run].** `$`, `?`, a backtick, a lone `!`, U+00A0, U+3000, U+200B, a byte order mark in mid-file, and a vertical tab between tokens are refused by both.
- **Warnings.** Every `SyntaxWarning` I met is only a warning on all four versions: an invalid escape, `1if`, `x is 1`, `assert (x, "msg")`, `1(2)`, and 3.14's `return`, `break` or `continue` in `finally` ([PEP 765](https://peps.python.org/pep-0765/)) [run].

## 3. Grammar

### 3.1 Syntax that depends on the version (B)

ruff accepts all of it. CPython accepts each form from its release on.

| module | ruff | 3.12 | 3.13 | 3.14 | 3.14t | added in [source] |
|---|---|---|---|---|---|---|
| `def f[T = int](): pass` | passes | refuses | runs | runs | runs | 3.13, [PEP 696](https://peps.python.org/pep-0696/) |
| `except ValueError, TypeError:` | passes | refuses | refuses | runs | runs | 3.14, [PEP 758](https://peps.python.org/pep-0758/) |
| `x = t'{1}'` | passes | refuses | refuses | runs | runs | 3.14, [PEP 750](https://peps.python.org/pep-0750/) |
| `lazy import os` | passes | refuses | refuses | refuses | refuses | 3.15, [PEP 810](https://peps.python.org/pep-0810/) |
| `x = [*a for a in b]` | passes | refuses | refuses | refuses | refuses | 3.15, [PEP 798](https://peps.python.org/pep-0798/) |
| `case +1:` in a `match` | passes | refuses | refuses | refuses | refuses | 3.15, per ruff |
| `f((a)=1)` | passes | refuses | refuses | refuses | refuses | removed in 3.8 |

All results [run]. The same holds for the class and `type` forms of a default, `except*` without parentheses, `{**a for a in b}`, and the generator form of unpacking.

**Why.** ruff's parser knows each form and the version it [came or went in](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/error.rs#L1095-L1141). It reports a mismatch only in `unsupported_syntax_errors()`, which purba does not read. CPython's grammar has the rule only from that release on: compare `type_param` at [v3.12.14](https://github.com/python/cpython/blob/v3.12.14/Grammar/python.gram#L645-L652) with [v3.13.17](https://github.com/python/cpython/blob/v3.13.17/Grammar/python.gram#L650-L667), and `except_block` at [v3.13.17](https://github.com/python/cpython/blob/v3.13.17/Grammar/python.gram#L427) with [v3.14.8](https://github.com/python/cpython/blob/v3.14.8/Grammar/python.gram#L446-L460).

Passing ruff a target version would not change purba's verdict while purba reads only `errors()`.

### 3.2 A bare `yield` used as an operand (B)

```python
"def f():\n    yield,\n"
"def f():\n    x = yield if a else b\n"
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| `yield,` `yield, 1` `yield in x` `yield is x` `yield and x` `yield < 1` `yield == 1` `yield \| 1` `yield if x else y` `x = yield,` | passes | refuses | refuses | refuses | refuses |
| `yield + 1` `yield()` `yield[0]` | passes | runs | runs | runs | runs |

All [run]; the second module was also checked with the wheel. CPython: `invalid syntax`. `yield + 1` and the others in the second row are `yield (+1)`, `yield ()` and `yield [0]` on both sides.

**Why.** In CPython's grammar a yield is a whole statement or a whole right-hand side, and its value is `[star_expressions]` ([v3.14.8 `python.gram`](https://github.com/python/cpython/blob/v3.14.8/Grammar/python.gram#L713-L722)). Nothing may follow a bare `yield`. ruff parses `yield` as the left operand of an expression. When no expression follows it, ruff [returns the bare yield](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/parser/expression.rs#L406-L416), and the caller goes on to read `,`, `in` or `if` as an operator ([`parse_yield_expression`](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/parser/expression.rs#L2767-L2807)) [source]. I found no ruff issue about it.

### 3.3 A bare `raise` before a semicolon (A)

```python
"raise;\n"
"def f():\n    try:\n        pass\n    except E:\n        raise; log()\n"
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| | refuses | runs | runs | runs | runs |

All [run], also with the wheel. ruff: `Expected an expression`.

**Why.** CPython: `raise_stmt: 'raise' expression ['from' expression] | 'raise'`, and `;` separates simple statements ([v3.14.8 `python.gram`](https://github.com/python/cpython/blob/v3.14.8/Grammar/python.gram#L115-L117), [`raise_stmt`](https://github.com/python/cpython/blob/v3.14.8/Grammar/python.gram#L188-L190)). ruff ends a bare `raise` only at a newline. At `;` it [tries to parse an exception](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/parser/statement.rs#L530-L553). ruff fixed this in [ruff#29761](https://github.com/astral-sh/ruff/pull/29761), merged 2026-10-27, first in the tag 0.17.4 [source; the tag was checked with GitHub's compare].

### 3.4 `match` used as a name after a semicolon (A)

```python
"m = 1; match = 2\n"
"x; match(1)\n"
"def f():\n    x; match = 1\n"
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| | refuses | runs | runs | runs | runs |

All [run], also with the wheel. ruff: `Compound statements are not allowed on the same line as simple statements`. `match; x`, `if 1: x; match`, and `type`, `case` or `lazy` after `;` pass on both sides.

**Why.** `match` is a soft keyword. CPython tries a `match` statement and falls back to an expression ([v3.14.8 `match_stmt`](https://github.com/python/cpython/blob/v3.14.8/Grammar/python.gram#L480-L482)). ruff lists the `match` token among [compound statement starts](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/parser/statement.rs#L25-L36). After a simple statement and `;`, it [reports an error on that token](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/parser/statement.rs#L157-L185) before it knows the token is a name. ruff fixed this in [ruff#29760](https://github.com/astral-sh/ruff/pull/29760), merged 2026-10-28, first in the tag 0.17.4 [source]. The crate `ruff_python_parser` 0.0.21 was published the same day as 0.17.4, 2026-11-05. I did not download it to confirm that it holds both fixes.

The soft keywords agreed in the 129 other probes I wrote for them, except for `return match` at module level, which only CPython's compiler refuses (4). These include `match` as a call, a subscript, an attribute, an annotated target, the left side of every binary operator, and a decorator, and `type`, `case`, `lazy` and `_` in the same places [run].

### 3.5 `<>` after `from __future__ import barry_as_FLUFL` (A)

```python
"from __future__ import barry_as_FLUFL\nx = 1 <> 2\n"
```

| | ruff | 3.12 | 3.13 | 3.14 | 3.14t |
|---|---|---|---|---|---|
| `<>` after the import | refuses | runs | runs | runs | runs |
| `!=` after the import | passes | refuses | refuses | refuses | refuses |

All [run]; the first was also checked with the wheel. ruff: `Expected an expression`. CPython, second row: `with Barry as BDFL, use '<>' instead of '!='`.

**Why.** The import changes which spelling CPython accepts ([v3.14.8 `noteq_bitwise_or`](https://github.com/python/cpython/blob/v3.14.8/Grammar/python.gram#L787-L788), [`_PyPegen_check_barry_as_flufl`](https://github.com/python/cpython/blob/v3.14.8/Parser/action_helpers.c#L934-L947), [PEP 401](https://peps.python.org/pep-0401/)). ruff's lexer [has no `<>` token](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/lexer.rs#L562-L574). ruff closed [ruff#25859](https://github.com/astral-sh/ruff/issues/25859) as not planned.

### 3.6 Grammar agreements

Apart from 3.1 to 3.5, the fuzz found no grammar disagreement in 6,174,587 token sequences [run]. I wrote 258 grammar and pattern modules from CPython's `invalid_*` rules and from its pattern rules. Apart from the kinds above and the compiler checks in 4, they all agreed. They cover assignment and deletion targets, star expressions, call arguments, parameters, imports, `with`, comprehensions, `except`, f-string fields, and patterns. ruff often reports the same refusal with other words (6).

## 4. Compiler and symbol table checks (B)

CPython parses a module, then builds its symbol table and compiles it. Both later steps can refuse the module. ruff's parser checks none of these rules. ruff checks many of them in `SemanticSyntaxChecker`, which purba does not run (see "How purba calls ruff"). I probed 64 distinct CPython messages of this kind. That count masks the names and numbers in a message, and leaves out the messages of the other sections. ruff passed every module. Unless a row says otherwise, all four CPythons refuse [run].

| family | a minimal module | CPython's words | source at v3.14.8 |
|---|---|---|---|
| scope of `return`, `break`, `continue` | `return`; `break`; `def f():\n    continue` | `'return' outside function`, `'break' outside loop`, `'continue' not properly in loop` | [`codegen.c`](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L2205), [L2246](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L2246), [L2262](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L2262) |
| `yield` and `await` scope | `yield 1`; `await x`; `def f():\n    await x`; `[(yield) for x in y]` in a function | `'yield' outside function`, `'await' outside function`, `'await' outside async function`, `'yield' inside list comprehension` | [`codegen.c`](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L5236), [`symtable.c`](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L2516-L2524) |
| async forms outside `async def` | `def f():\n    async with x: pass`; `def f():\n    return [x async for x in y]` | `'async with' outside async function`, `asynchronous comprehension outside of an asynchronous function` | [`symtable.c`](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L79-L82), [L3083](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L3083) |
| generators | `async def f():\n    yield from x`; `async def f():\n    yield 1\n    return 2` | `'yield from' inside async function`, `'return' with value in async generator` | [`codegen.c`](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L2208), [L5251](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L5251) |
| `global` and `nonlocal` | `nonlocal x`; `def f():\n    nonlocal x`; `def f(x):\n    global x`; `def f():\n    x = 1\n    global x`; `global x\nx: int = 1` | `nonlocal declaration not allowed at module level`, `no binding for nonlocal 'x' found`, `name 'x' is parameter and global`, `… assigned to before global declaration`, `annotated name 'x' can't be global` | [`symtable.c`](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L17-L38), [L688-L697](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L688-L697) |
| `import *` | `def f():\n    from os import *` | `import * only allowed at module level` | [`symtable.c`](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L40) |
| duplicates | `def f(a, a): pass`; `f(a=1, a=2)`; `def f[T, T](): pass` | `duplicate argument 'a' in function definition`, `keyword argument repeated: a`, `duplicate type parameter 'T'` | [`symtable.c`](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L384), [L76](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L76), [`codegen.c`](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L4036) |
| `:=` in comprehensions | `[i := 0 for i in range(3)]`; `[i for i in (y := [1])]`; `class C:\n    x = [(y := i) for i in z]` | `assignment expression cannot rebind comprehension iteration variable 'i'`, `… cannot be used in a comprehension iterable expression`, `… cannot be used in a class body` | [`symtable.c`](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L43-L61) |
| annotation scopes | `type A = (x := int)`; `def f[T: (x := int)](): pass`; `class C[T]((x := object)): pass` | `named expression cannot be used within a type alias`, `… within a TypeVar bound`, `… within the definition of a generic` | [`symtable.c`](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L64-L73), [L3132-L3151](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L3132-L3151) |
| writes to `__debug__` | `__debug__ = 1`; `def f(__debug__): pass`; `import os as __debug__`; `f(__debug__=1)`; and 21 more modules | `cannot assign to __debug__` | [`symtable.c`](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L1604-L1617) |
| star expressions | `*a = b`; `a, *b, *c = d`; `x = *a`; `x = f"{*a}"` | `starred assignment target must be in a list or tuple`, `multiple starred expressions in assignment`, `can't use starred expression here` | [`codegen.c`](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L5318-L5321), [L3415](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L3415) |
| `__future__` | `import os\nfrom __future__ import annotations`; `b"doc"\nfrom __future__ import annotations`; `from __future__ import nope`; `from __future__ import braces` | `from __future__ imports must occur at the beginning of the file`, `future feature nope is not defined`, `not a chance` | [`symtable.c`](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L1837-L1840), [`future.c`](https://github.com/python/cpython/blob/v3.14.8/Python/future.c#L6), [L43](https://github.com/python/cpython/blob/v3.14.8/Python/future.c#L43) |
| `match` | `case y:` before another `case`; `case [a, a]:`; `case [a] \| (b,):`; `case {1: a, 1: b}:`; `case C(a=1, a=2):`; `case [*a, *b]:`; `case f'x':` | `name capture 'y' makes remaining patterns unreachable`, `multiple assignments to name 'a' in pattern`, `alternative patterns bind different names`, `mapping pattern checks duplicate key (1)`, `attribute name repeated in class pattern: a`, `multiple starred names in sequence pattern`, `patterns may only match literals and attribute lookups` | [`codegen.c`](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L5745-L6346) |
| `try` | `except:` before `except E:`; `break`, `continue` or `return` in `except*` | `default 'except:' must be last`, `'break', 'continue' and 'return' cannot appear in an except* block` | [`codegen.c`](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L2469), [L636](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L636) |

The same checks are in `Python/compile.c` and `Python/symtable.c` at [v3.12.14](https://github.com/python/cpython/blob/v3.12.14/Python/compile.c#L3143) and v3.13.17. Seven rows differ by version:

| module | ruff | 3.12 | 3.13 | 3.14 | 3.14t | why [source] |
|---|---|---|---|---|---|---|
| `def g():\n    def f(x: (yield)): pass` | passes | runs | runs | refuses | refuses | from 3.14, an annotation is an [annotation scope](https://github.com/python/cpython/blob/v3.14.8/Doc/reference/executionmodel.rst#L193-L223), and an annotation scope cannot hold `yield`, `await` or `:=` |
| `x: (y := int) = 1` | passes | runs | runs | refuses | refuses | the same |
| `del x.__debug__` | passes | runs | runs | refuses | refuses | 3.14 checks an attribute in every context ([v3.14.8 `symtable.c`](https://github.com/python/cpython/blob/v3.14.8/Python/symtable.c#L2564)); 3.13 checked only a store ([v3.13.17 `compile.c`](https://github.com/python/cpython/blob/v3.13.17/Python/compile.c#L6336-L6346)); [gh-122245](https://github.com/python/cpython/issues/122245) |
| `def f[T = int, U](): pass` | passes | refuses | refuses | refuses | refuses | 3.12 has no defaults (3.1); 3.13 on: `non-default type parameter 'U' follows default type parameter` |
| `def f[T = (x := int)](): pass` | passes | refuses | refuses | refuses | refuses | 3.12: no defaults; 3.13 on: `… within a TypeVar default` |
| `from .__future__ import braces` | passes | refuses | runs | runs | runs | 3.12 treats a relative `__future__` as the real one ([v3.12.14 `future.c`](https://github.com/python/cpython/blob/v3.12.14/Python/future.c#L86)); 3.13 checks the level ([v3.13.17](https://github.com/python/cpython/blob/v3.13.17/Python/future.c#L88)) |
| `async def g():\n    class C:\n        x: (await y)` | passes | refuses | refuses | refuses | refuses | the words differ: `'await' outside function` up to 3.13, `… within an annotation` on 3.14 |

### 4.1 Size limits in the compiler and parser (B)

| module | ruff | 3.12 | 3.13 | 3.14 | 3.14t | why [source] |
|---|---|---|---|---|---|---|
| `x = 1…1`, a literal of 4,301 digits | passes | refuses | refuses | refuses | refuses | `Exceeds the limit (4300 digits) for integer string conversion` ([v3.14.8 `pegen.c`](https://github.com/python/cpython/blob/v3.14.8/Parser/pegen.c#L715-L740)); 4,300 digits passes; `-I` ignores `PYTHONINTMAXSTRDIGITS` |
| 21 nested `async for` in an `async def`, or 22 nested `for`, `while`, `with` or `try` | passes | refuses | refuses | refuses | refuses | `too many statically nested blocks`; `CO_MAXBLOCKS` is 21 ([v3.12.14 `code.h`](https://github.com/python/cpython/blob/v3.12.14/Include/cpython/code.h#L210), [v3.14.8 `compile.c`](https://github.com/python/cpython/blob/v3.14.8/Python/compile.c#L761-L762)) |
| `a0, …, a255, *b = c` | passes | refuses | refuses | refuses | refuses | `too many expressions in star-unpacking assignment` ([v3.14.8 `codegen.c`](https://github.com/python/cpython/blob/v3.14.8/Python/codegen.c#L3407)) |
| `x = ` and 5,967 unary minus signs, or 2,984 `2 **`, or an `if` with 5,963 `elif` | passes | refuses | refuses | refuses | refuses | `MemoryError: Parser stack overflowed - Python source too complex to parse` ([v3.14.8 `pegen_errors.c`](https://github.com/python/cpython/blob/v3.14.8/Parser/pegen_errors.c#L414), `MAXSTACK` 6000 in [`parser.c`](https://github.com/python/cpython/blob/v3.14.8/Parser/parser.c#L11-L17)) |
| `x = a` and 10,000 `()`, `[0]` or `.b` | passes | refuses | refuses | runs | runs | `RecursionError: maximum recursion depth exceeded during compilation` up to 3.13 ([v3.12.14 `ast_opt.c`](https://github.com/python/cpython/blob/v3.12.14/Python/ast_opt.c#L715)); 3.14 measures the C stack instead, and refused only from about 29,000 |
| `x = 1+1+…`, 100,000 terms | passes | refuses | refuses | refuses | refuses | the same; the limit is about 10,000 up to 3.13 and about 29,000 on 3.14 |

The thresholds are the largest module that passed, by bisection [run]. The ones that measure the C stack depend on its size; I ran with 8 MB. ruff's limit was not reached: the wheel parsed `x = a` with 1,000,000 `.b` without a crash [run].

## 5. Error recovery

ruff's parser recovers from an error and goes on, so it can list many errors for one module. purba keeps only the first, and refuses whenever the list is not empty ([purba `lib.rs`](https://github.com/kalonji-tools/purba/blob/e1490a302ad1abc913e252dba7bc626cf88dd9d5/crates/parsing/src/lib.rs#L386-L405)). So recovery cannot turn a refusal into a pass [source]. It decides only which error purba shows, and (6) shows those can differ from CPython's. In the probes, ruff listed up to 4,999 errors for one module [run].

## 6. Error position and wording (both refuse)

When both sides refuse, the words usually differ, and so does the line sometimes. purba shows ruff's words with no file and line. [The record](https://github.com/kalonji-tools/purba/blob/e1490a302ad1abc913e252dba7bc626cf88dd9d5/docs/decisions/parsing-hands-on-no-tree-that-ruff-recovered.md) already says so. All [run]:

| module | ruff | CPython (all four) |
|---|---|---|
| `x = (\n` | line 2: `unexpected EOF while parsing` | line 1: `'(' was never closed` |
| `x = (\ny = 1\nz = 2\n` | line 2: ``Expected `)`, found `=` `` | line 1: `'(' was never closed` |
| `print 'hi'\n` | `Simple statements must be separated by newlines or semicolons` | `Missing parentheses in call to 'print'. Did you mean print(...)?` |
| `x = [1 2]\n` | ``Expected `,`, found int`` | `invalid syntax. Perhaps you forgot a comma?` |
| `if x = 1:\n    pass\n` | ``Expected `:`, found `=` `` | `invalid syntax. Maybe you meant '==' or ':=' instead of '='?` |
| `b"if 1:\n\tx = 1\n        y = 2\n"` | `Unexpected indentation` | `TabError: inconsistent use of tabs and spaces in indentation` |
| `"if True:\n    1\n      \\\n    2\n"` | line 3: `Unexpected indentation` | line 4: `IndentationError: unexpected indent` |
| `x = 1\x00` | `Got unexpected token` | `source code string cannot contain null bytes` (no line) |
| `x =\u00a01` | `Got unexpected token` | `invalid non-printable character U+00A0` |
| `x = 'abc\n` | `missing closing quote in string literal` | `unterminated string literal (detected at line 1)` |
| `x = 1 + \\` at end of file | `Expected a newline after line continuation character` | `unexpected EOF while parsing` |

A refusal by purba's reader (1.1, 1.2) is an invocation error with exit 3. A refusal by ruff is a project error with exit 4. When CPython refuses at load, purba also reports a project error, with CPython's words and `(file, line N)` [run, with the wheel].

## What was not checked

- **An AST passed to `compile()`.** `Python/ast.c` validates an AST object. A source file never reaches that check, so I left it out.
- **Warnings turned into errors.** Under `-W error`, each `SyntaxWarning` in 2.7 becomes a `SyntaxError` at compile time. I ran with the default filters only.
- **Compile flags and settings.** I did not try `PyCF_ALLOW_TOP_LEVEL_AWAIT`, `optimize`, `sys.set_int_max_str_digits`, or a `PYTHONINTMAXSTRDIGITS` that `-I` would ignore.
- **The C stack of purba's worker.** The 3.14 thresholds in 4.1 depend on the stack of the thread that imports. I ran on the main thread of a plain interpreter with an 8 MB stack.
- **3.14t at 3.14.8.** The free-threaded interpreter was 3.14.7. Its sources were not read separately.
- **Other platforms.** Only Linux.
- **Exhaustiveness of the grammar.** The fuzz samples token sequences at random, up to 7 tokens. It is not a proof. A disagreement that needs a longer or rarer sequence can remain. The hand corpus covers each CPython error site I found by reading, but I read the sources by searching for error calls, not line by line.
- **Compile-time fuzzing.** The fuzz compared against `ast.parse` only. The compiler checks in 4 come from reading the sources and from the hand corpus.
- **Every `\N{name}`.** I tested names from each Unicode release, not every name.
- **The ruff fixes.** I did not run `ruff_python_parser` 0.0.21 or later. That the fixes of 3.3 and 3.4 are in it rests on the tag compare and the publish date.
- **Issue searches.** "I found no ruff issue" means a few searches of ruff's tracker found none. It does not mean none exists.

## Open questions

1. Should purba move to a `ruff_python_parser` that holds [ruff#29760](https://github.com/astral-sh/ruff/pull/29760) and [ruff#29761](https://github.com/astral-sh/ruff/pull/29761)? That would remove two kinds of direction A.
2. Is the tab width of 2.1 worth a report to ruff? It is the one kind of direction A that ruff has not fixed or declined, and it goes both ways.
3. Is the bare `yield` of 3.2 worth a report to ruff? It is direction B only, so it matters to purba only if purba trusts ruff's verdict.
4. Should purba read a declared encoding and decode, as [#644](https://github.com/kalonji-tools/purba/issues/644) asks? Of the 7 kinds of direction A, 1.1 is the most likely in a real project, and the one most likely to be ASCII-only.
5. Does any part of purba treat "ruff parsed it" as "it will load"? Direction B is harmless only if nothing does.
6. If purba ever wants ruff to refuse what the running CPython refuses, it would need both a target version and a read of `unsupported_syntax_errors()`. It would also need `SemanticSyntaxChecker`, and still could not match 2.2, 2.3, 2.5 and 4.1.

## Appendix: the probes

None of this is committed. The ruff probe was an example in `crates/parsing/examples/`, run with `mise exec -- cargo run -q -p purba-parsing --example <name> -- <files>`:

```rust
match purba_parsing::parse(path) {
    Ok(_) => "PASS".to_string(),
    Err(purba_parsing::Error::Read(e)) => format!("READ-REFUSE {e}"),
    Err(purba_parsing::Error::Syntax(e)) => format!(
        "REFUSE line {} col {}: {}", e.start().line(), e.start().column(), e.message()
    ),
}
```

A second example parsed one hex-encoded module per line with `parse_unchecked_source`, for the fuzz. A third looped over every `char`. The CPython side ran, for each file:

```python
compile(open(path, "rb").read(), path, "exec", dont_inherit=True)
```

with `warnings.simplefilter("always")` inside `catch_warnings(record=True)`, and printed the exception's type, `lineno` and `msg`. The fuzz used `ast.parse(data)` in place of `compile()`. The real wheel was `purba-order/target/wheel-venv/bin/purba`, built from `e1490a30`, on CPython 3.12.14.
