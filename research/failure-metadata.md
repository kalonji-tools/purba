# Which facts about a failure other tools collect where they find it, and which purba can collect

This file answers [#710](https://github.com/kalonji-tools/purba/issues/710).
It is never merged.

## Answer

Every tool in this survey finds a failure's location at one stage. Most compute what a person reads later, from the source text.
rustc and ruff keep only a byte range when they find an error. They compute the line, the column and the source line when they print it.
CPython, mypy, `go test` and pytest compute a line and a column when the error is found, and keep those numbers.
No two tools count a column the same way.
A column counts UTF-8 bytes in CPython's runtime positions, in Go and in mypy's older parser.
It counts Unicode scalar values in rustc's JSON, in ruff's `LineIndex` and in a `SyntaxError` from CPython's parser.
LSP counts UTF-16 units by default. SARIF counts UTF-16 units or code points, and has no column in bytes.
A Rust panic's column counts display width, so a tab counts 4.
Five tools disagree with themselves:
- **CPython.** A `SyntaxError` from the compiler or the symbol table counts bytes. A `SyntaxError` from the parser counts characters, and the documentation says characters for both.
- **mypy.** Its text output and its JSON output differ by one. Its new parser converts each column twice.
- **rustc.** Its JSON `text` is the normalized line, but its documentation says "the original source code".
- **pytest.** Its JUnit `line` is 0-based, and its traceback line is 1-based.
- **ruff.** Its SARIF leaves out the `columnKind` that SARIF requires.

A few fields exist only at the stage that finds the failure: the range, the label, the suggested fix, the error code, the cause chain, and a frame's locals and code.
Each tool's form for carrying a failure to another process loses some of these fields.
nextest, `go test`, mypy's workers, dmypy and Python's `multiprocessing` carry only text.
ruff's JSON drops every secondary location. pytest-xdist keeps pytest's tree of text and line numbers.
Only rustc's JSON keeps every secondary span, label and suggestion.
For purba, parsing can collect every syntax error that ruff finds, each with a byte range.
Selection and assertion hold the range of each `def` and each assert, and keep only names and text.
Execution holds the `PyErr`. From it, execution can copy the type, the message, the notes, the cause chain, each frame's line and byte columns, and a `SyntaxError`'s location, all as plain data.
This copy has two traps.
First, the unit of a `SyntaxError`'s offset depends on the CPython stage that raised it.
Second, CPython's positions on an assert's line point into assertion's rewritten text, not into the file.
No part of purba can collect a suggested fix, an error code, a help text, a related location from the parser, or the values in a failed assert.
Only one cost breaks [an outcome is data](https://github.com/kalonji-tools/purba/blob/b64bb14/LANDSCAPE.md?plain=1#L140): to keep the traceback, the exception or a frame as an object.
CPython's own `TracebackException` does not escape this cost: from 3.13.3 it holds a live code object and does not pickle.

| tool or format | the location it keeps when it finds a failure | a column counts | the end | crosses a process as |
|---|---|---|---|---|
| rustc 1.99.1 | a `Span`, a byte range in the session's `SourceMap` | Unicode scalars, in JSON | kept, exclusive | JSON: one `Diagnostic` for each line |
| ruff 0.17.0 parser, purba's pin | `TextRange`, a UTF-8 byte range | `line_column` counts Unicode scalars | kept, exclusive | not carried: threads |
| ruff 0.17.3 linter | `Annotation { span: Span }`, a byte range | Unicode scalars in each output | kept, exclusive | JSON drops secondary annotations |
| CPython 3.15.2 `SyntaxError` | `lineno`, `offset`, `end_lineno`, `end_offset` | the parser: code points; the compiler and the symbol table: bytes + 1 | kept, exclusive; `0` means none | `pickle` keeps it: the location is in `args` |
| CPython 3.15.2 traceback | `tb_lineno`, `tb_lasti` → `co_positions()` | 0-based UTF-8 bytes | kept, exclusive | as a string, in `multiprocessing` |
| pytest 9.2.0 | `location` (lineno is 0-based), `ReprFileLocation.lineno` (1-based) | no column kept | none | JSON of builtins, over execnet |
| mypy 2.4.1 | `ErrorInfo.line`, `.column`, `.end_line`, `.end_column` | its legacy parser: bytes; its native parser: wrong after non-ASCII | kept, exclusive | worker and daemon: formatted text |
| cargo-nextest 0.9.150 | none; a panic's `file:line:col` lies in stderr text | display width + 1, from rustc | none | stdout and stderr pipes |
| `go test` go1.27.3 | `file.go:LINE:` text; no column | the compiler: 1-based bytes | none | `test2json` events of text |
| LSP 3.17 `Diagnostic` | `range: Range`, 0-based | UTF-16 by default; UTF-8 and UTF-32 by negotiation | kept, exclusive | it is the carrier |
| SARIF 2.1.0 | `region`, 1-based | `utf16CodeUnits` or `unicodeCodePoints`; no bytes | kept, exclusive | it is the carrier |
| JUnit XML | none; pytest's `xunit1` adds `line`, 0-based | none | none | it is the carrier |

Each row is [A], read at a tag or measured in this session, unless a section marks it otherwise.
The probes ran on versions that can differ from the tag that was read. Appendix A gives both.

## 1. The fields

### Which field holds each fact

A cell names the field. `—` means the tool has no field for that fact. "text" means that the fact exists only inside a message string.

| tool | file | start, end | source text | label | note, help | fix | code | severity | related location | cause |
|---|---|---|---|---|---|---|---|---|---|---|
| rustc `DiagInner` | in the `Span`, through the `SourceMap` | `span: MultiSpan` | — in memory; JSON `text` | `span_labels: Vec<(Span, DiagMessage)>` | `children: Vec<Subdiag>`, level `Note` or `Help` | `suggestions: Suggestions` of `CodeSuggestion` with `Applicability` | `code: Option<ErrCode>`; a lint name in `is_lint` | `level: Level` | a span that is not primary, or a child's span | — |
| ruff parser `ParseError` | — | `location: TextRange` | — | — | — | — | the variant of `error: ParseErrorType` | — | — | — |
| ruff linter `DiagnosticInner` | `Span { file: UnifiedFile }` | `Span { range: Option<TextRange> }` | `SourceFile.code`, shared | `Annotation { message }` | `subs: Vec<SubDiagnostic>`, `Help` or `Info` | `fix: Option<Fix>` with `edits`, `applicability` | `id: DiagnosticId`, `secondary_code` | `severity: Severity` | an `Annotation` with `is_primary: false` | — |
| CPython `SyntaxError` | `filename` | `lineno`, `offset`, `end_lineno`, `end_offset` | `text`, the whole line | — | — | — | the class | — | text in `msg` | — |
| CPython exception and traceback | `co_filename` of each frame | `co_positions()` of each frame | through `linecache` | — | `__notes__: list[str]` | — | the class | — | — | `__cause__`, `__context__`, `BaseExceptionGroup.exceptions` |
| pytest `TestReport` | `location[0]`, `ReprFileLocation.path` | `ReprFileLocation.lineno` only | `ReprEntry.lines`, rendered | — | text: the notes join `reprcrash.message` | — | — | `outcome`, `when` | — | `ExceptionChainRepr.chain` |
| mypy `ErrorInfo` | the key of `error_info_map` | `line`, `column`, `end_line`, `end_column` | — ; re-read for `--pretty` | — | a second `ErrorInfo` with `severity="note"` | — ; advice is note text | `code: ErrorCode` | `severity`, `"error"` or `"note"` | text | — |
| cargo-nextest `ExecuteStatus` | text in stderr | text in stderr | — | — | — | — | `FailureStatus`, an exit code or a signal | — | — | — |
| `go test` event | text in `Output` | text, a line only | — | — | — | — | — | `OutputType` `error` | — | — |
| LSP `Diagnostic` | `uri` on the envelope | `range: Range` | — | `relatedInformation[].message` only | `codeDescription.href`, a link | — ; a separate `CodeAction` | `code?: integer \| string` | `severity?: DiagnosticSeverity` | `relatedInformation?: DiagnosticRelatedInformation[]` | — |
| SARIF `result` | `artifactLocation.uri` | `region` | `region.snippet`, `contextRegion` | `region.message`, `location.annotations` | on the rule: `help`, `helpUri` | `fixes: fix[]` | `ruleId` | `level` | `relatedLocations: location[]` | — on a result |
| JUnit XML | — ; pytest `xunit1` adds `file` | — ; pytest `xunit1` adds `line` | text in the body | — | — | — | `type` on `failure` (Ant, Surefire) | the element: `failure` or `error` | — | text in the body |

- **Only rustc and ruff's linter keep a label, a note or help, and a fix as fields.** Both keep them on the stage's own diagnostic type, beside a byte range.
- **A parser alone keeps the least.** ruff's `ParseError` holds a kind and a byte range. CPython's `SyntaxError` holds one location, the line and a message.
- **Only Python keeps a cause chain as data.** CPython keeps it on the exception. pytest keeps it as a list of rendered tracebacks. SARIF keeps a chain only on a tool's own `notification`, never on a `result`.
- **No test runner in the survey keeps a column.** pytest, nextest, `go test` and JUnit XML keep at most a line. pytest's live `TracebackEntry` has a column, and pytest uses it only to draw `^^^` into the text.

### rustc

| claim | tier | source |
|---|---|---|
| `DiagInner` fields: `level: Level`, `messages: Vec<(DiagMessage, Style)>`, `code: Option<ErrCode>`, `lint_id: Option<LintExpectationId>`, `span: MultiSpan`, `children: Vec<Subdiag>`, `suggestions: Suggestions`, `args: DiagArgMap`, `sort_span: Span`, `is_lint: Option<IsLint>`, `long_ty_path: Option<PathBuf>`, `emitted_at: DiagLocation` | [A] | [rustc_errors/src/diagnostic.rs#L261-L286](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/diagnostic.rs#L261-L286) |
| `emitted_at` is rustc's own location in its source, not the user's: "With `-Ztrack_diagnostics` enabled, we print where in rustc this error was emitted." | [A] | [diagnostic.rs#L283-L285](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/diagnostic.rs#L283-L285) |
| `Subdiag { level: Level, messages: Vec<(DiagMessage, Style)>, span: MultiSpan }`. A child has no code and no suggestion of its own | [A] | [diagnostic.rs#L427-L433](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/diagnostic.rs#L427-L433) |
| `MultiSpan { primary_spans: Vec<Span>, span_labels: Vec<(Span, DiagMessage)> }` | [A] | [rustc_error_messages/src/lib.rs#L87-L90](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_error_messages/src/lib.rs#L87-L90) |
| `CodeSuggestion { substitutions: Vec<Substitution>, msg: DiagMessage, style: SuggestionStyle, applicability: Applicability }`; `SubstitutionPart { span: Span, snippet: String }` | [A] | [rustc_errors/src/lib.rs#L156-L201](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/lib.rs#L156-L201) |
| `Applicability` = `MachineApplicable`, `MaybeIncorrect`, `HasPlaceholders`, `Unspecified` | [A] | [rustc_lint_defs/src/lib.rs#L55-L82](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_lint_defs/src/lib.rs#L55-L82) |
| `Level` = `Bug, Fatal, Error, DelayedBug, ForceWarning, Warning, Note, OnceNote, Help, OnceHelp, FailureNote, Allow, Expect` | [A] | [rustc_errors/src/lib.rs#L1585-L1635](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/lib.rs#L1585-L1635) |
| `ErrCode` has `#[max = 9999] // Because all error codes have four digits.` | [A] | [rustc_errors/src/codes.rs#L12-L24](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/codes.rs#L12-L24) |
| The JSON form: `Diagnostic { message, code: Option<DiagnosticCode>, level, spans: Vec<DiagnosticSpan>, children: Vec<Diagnostic>, rendered: Option<String> }` | [A] | [rustc_errors/src/json.rs#L182-L194](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/json.rs#L182-L194) |
| `DiagnosticSpan { file_name, byte_start: u32, byte_end: u32, line_start, line_end, column_start, column_end, is_primary, text: Vec<DiagnosticSpanLine>, label, suggested_replacement, suggestion_applicability, expansion }`; `column_start` is documented "/// 1-based, character offset." | [A] | [json.rs#L196-L221](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/json.rs#L196-L221) |
| In JSON, each `CodeSuggestion` becomes a child `Diagnostic` of level `help`, whose spans carry `suggested_replacement` and `suggestion_applicability` | [A] | [json.rs#L301-L316](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/json.rs#L301-L316); P-rustc-4 |
| The module says: "The format of the JSON output should be considered *unstable*." | [A] | [json.rs#L7-L8](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/json.rs#L7-L8) |

### ruff

purba pins `ruff_python_parser`, `ruff_python_ast` and `ruff_text_size` at `=0.0.17`. They were published from commit `5b27c1cc`, which is the tag `0.17.0` (Appendix A).

| claim | tier | source |
|---|---|---|
| `pub struct ParseError { pub error: ParseErrorType, pub location: TextRange }`, with no serde derive | [A] | [ruff_python_parser/src/error.rs#L11-L15](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/error.rs#L11-L15) |
| `Display` is `"{} at byte range {:?}"`. Measured: `Expected a statement at byte range 11..12` | [A] | [error.rs#L31-L35](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/error.rs#L31-L35); P-ruff-1 |
| `ParseErrorType` has 42 variants, such as `ExpectedToken { expected: TokenKind, found: TokenKind }` and `Lexical(LexicalErrorType)` | [A] | [error.rs#L104-L205](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/error.rs#L104-L205) |
| `Parsed<T> { syntax, tokens, errors: Vec<ParseError>, unsupported_syntax_errors: Vec<UnsupportedSyntaxError> }` | [A] | [ruff_python_parser/src/lib.rs#L387-L413](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/lib.rs#L387-L413) |
| `UnsupportedSyntaxError { pub kind, pub range: TextRange, pub target_version: PythonVersion }` | [A] | [error.rs#L479-L485](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/error.rs#L479-L485) |
| `SemanticSyntaxError { pub kind, pub range: TextRange, pub python_version }` has 41 kinds, among them `ReturnOutsideFunction` and `NonlocalWithoutBinding(String)` | [A] | [semantic_errors.rs#L1336-L1340](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/semantic_errors.rs#L1336-L1340) |
| The linter's `DiagnosticInner { id, documentation_url, severity, message, custom_concise_message, annotations: Vec<Annotation>, subs: Vec<SubDiagnostic>, fix: Option<Fix>, parent, noqa_offset, secondary_code, header_offset }` | [A] | [ruff_db/src/diagnostic/mod.rs#L554-L568](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_db/src/diagnostic/mod.rs#L554-L568) |
| `Annotation { span: Span, message: Option<DiagnosticMessage>, is_primary: bool, tags: Vec<DiagnosticTag>, hide_snippet: bool }` | [A] | [mod.rs#L809-L827](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_db/src/diagnostic/mod.rs#L809-L827) |
| `Severity { Hint, Info, Warning, Error, Fatal }`; `SubDiagnosticSeverity` adds `Help` | [A] | [mod.rs#L1335-L1380](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_db/src/diagnostic/mod.rs#L1335-L1380) |
| `Fix { edits: Vec<Edit>, applicability: Applicability, isolation_level }` holds no message. A fix's message is the first sub of severity `Help` | [A] | [ruff_diagnostics/src/fix.rs#L8-L56](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_diagnostics/src/fix.rs#L8-L56); [mod.rs#L483-L488](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_db/src/diagnostic/mod.rs#L483-L488) |
| A syntax error becomes `Diagnostic::invalid_syntax`, with id `InvalidSyntax`, severity `Error` and one primary annotation | [A] | [mod.rs#L91-L101](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_db/src/diagnostic/mod.rs#L91-L101) |

### CPython

| claim | tier | source |
|---|---|---|
| `SyntaxError` members: `msg`, `filename`, `lineno`, `offset`, `text`, `end_lineno`, `end_offset`, `print_file_and_line`, `_metadata` | [A] | [Objects/exceptions.c#L2953-L2973](https://github.com/python/cpython/blob/v3.15.2/Objects/exceptions.c#L2953-L2973) |
| The documentation lists `filename`, `lineno`, `offset`, `text`, `end_lineno`, `end_offset`; `text` is "The source code text involved in the error." | [A] | [Doc/builtins/exceptions.rst#L529-L574](https://github.com/python/cpython/blob/v3.15.2/Doc/builtins/exceptions.rst#L529-L574) |
| `BaseException` carries `args`, `__traceback__`, `__cause__`, `__context__`, `__suppress_context__` and, from 3.11, `__notes__` | [A] | [exceptions.rst#L49-L82](https://github.com/python/cpython/blob/v3.15.2/Doc/builtins/exceptions.rst#L49-L82), [#L141-L160](https://github.com/python/cpython/blob/v3.15.2/Doc/builtins/exceptions.rst#L141-L160) |
| `FrameSummary.__slots__` at 3.15.2: `('filename', 'lineno', 'end_lineno', 'colno', 'end_colno', 'name', '_lines', '_lines_dedented', 'locals', '_code')` | [A] | [Lib/traceback.py#L339-L366](https://github.com/python/cpython/blob/v3.15.2/Lib/traceback.py#L339-L366) |
| `AssertionError` holds only `args`: `('msg',)` with a message, `()` for a bare `assert`. It holds no operand | [A] | P-cpython-3 |
| A suggestion is appended to the message string, such as `. Did you mean: '…'?` | [A] | [Lib/traceback.py#L1186-L1248](https://github.com/python/cpython/blob/v3.15.2/Lib/traceback.py#L1186-L1248) |

### pytest

| claim | tier | source |
|---|---|---|
| `TestReport.__init__(nodeid, location: tuple[str, int \| None, str], keywords, outcome, longrepr, when, sections, duration, start, stop, user_properties, **extra)` | [A] | [reports.py#L344-L361](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/reports.py#L344-L361) |
| `location`: "The line number is 0-based." | [A] | [reports.py#L367-L371](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/reports.py#L367-L371) |
| `ExceptionChainRepr(chain: Sequence[tuple[ReprTraceback, ReprFileLocation \| None, str \| None]])` | [A] | [_code/code.py#L1318-L1343](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/_code/code.py#L1318-L1343) |
| `ReprFileLocation(path: str, lineno: int, message: str)`, with no column and no end | [A] | [_code/code.py#L1505-L1526](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/_code/code.py#L1505-L1526) |
| The live `TracebackEntry` has `colno` and `end_colno`; its docstring says "Starting byte offset of the expression in the traceback entry." pytest uses them only to append a `^^^` line to `ReprEntry.lines` | [A] | [_code/code.py#L214-L255](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/_code/code.py#L214-L255), [#L996-L1032](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/_code/code.py#L996-L1032) |
| Assertion rewriting raises `AssertionError(<one formatted explanation str>)`; the operands reach the report only as text | [A] | [assertion/rewrite.py#L905-L910](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/assertion/rewrite.py#L905-L910) |
| A module with a syntax error gives `CollectErrorRepr`, whose one field is `longrepr: str`; the `SyntaxError`'s `lineno` and `offset` survive only as text | [A] | [reports.py#L542-L547](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/reports.py#L542-L547); P-pytest-2 |

### mypy

| claim | tier | source |
|---|---|---|
| `ErrorInfo` fields: `import_ctx`, `local_ctx`, `line`, `column`, `end_line`, `end_column`, `severity`, `message`, `code: ErrorCode \| None`, `blocker`, `only_once`, `module`, `target`, `origin_span`, `priority`, `hidden`, `parent_error` | [A] | [mypy/errors.py#L71-L157](https://github.com/python/mypy/blob/v2.4.1/mypy/errors.py#L71-L157) |
| A note is its own `ErrorInfo`; `parent_error` is allowed only on a note: "Only notes can specify parent errors" | [A] | [errors.py#L654-L658](https://github.com/python/mypy/blob/v2.4.1/mypy/errors.py#L654-L658) |
| The JSON keys: `file`, `line`, `column`, `end_line`, `end_column`, `message`, `hint`, `code`, `severity` | [A] | [mypy/error_formatter.py#L19-L36](https://github.com/python/mypy/blob/v2.4.1/mypy/error_formatter.py#L19-L36) |
| The native parser's `ParseError(TypedDict)` holds `line`, `column`, `message`, `blocker`, `code`, and no end | [A] | [mypy/nodes.py#L322-L327](https://github.com/python/mypy/blob/v2.4.1/mypy/nodes.py#L322-L327) |

### cargo-nextest and `go test`

| claim | tier | source |
|---|---|---|
| nextest keeps each attempt as `ExecuteStatus { retry_data, output, result, start_time, time_taken, is_slow, delay_before_start, error_summary, output_error_slice }` | [A] | [nextest-runner/src/reporter/events.rs#L1437-L1468](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/reporter/events.rs#L1437-L1468) |
| A failure's description is a slice of stderr found by a regex, `"^thread '([^']+)' (\\(\\d+\\) )?panicked at "`; "The process of extracting errors is heuristic and may not always be accurate." | [A] | [reporter/error_description.rs#L283-L289](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/reporter/error_description.rs#L283-L289), [#L347-L354](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/reporter/error_description.rs#L347-L354) |
| No nextest type parses the panic's `file:line:col` | [A] | the types above; P-nextest-1 |
| `go test` writes `fmt.Sprintf("%s:%d: ", file, line)` before a `t.Errorf` message. It writes no column | [A] | [src/testing/testing.go#L1155-L1177](https://github.com/golang/go/blob/go1.27.3/src/testing/testing.go#L1155-L1177) |
| A `go test -json` event: `Time`, `Action`, `Package`, `Test`, `Elapsed`, `Output`, `OutputType`, `FailedBuild`, `Key`, `Value`, `Path` | [A] | [src/cmd/internal/test2json/test2json.go#L31-L43](https://github.com/golang/go/blob/go1.27.3/src/cmd/internal/test2json/test2json.go#L31-L43) |

### LSP, SARIF and JUnit XML

| claim | tier | source |
|---|---|---|
| LSP `Diagnostic { range; severity?; code?: integer \| string; codeDescription?; source?; message; tags?; relatedInformation?; data? }` | [A] | [lsp/3.17/types/diagnostic.md#L6-L64](https://github.com/microsoft/language-server-protocol/blob/112d978951f2223a962bf19516637862f587bf55/_specifications/lsp/3.17/types/diagnostic.md#L6-L64) |
| An LSP fix is a `CodeAction` with `diagnostics?: Diagnostic[]` and `edit?: WorkspaceEdit`, outside the `Diagnostic` | [A] | [lsp/3.17/language/codeAction.md#L341-L421](https://github.com/microsoft/language-server-protocol/blob/112d978951f2223a962bf19516637862f587bf55/_specifications/lsp/3.17/language/codeAction.md#L341-L421) |
| LSP 3.18 changes only `message: string \| MarkupContent` | [A] | [lsp/3.18/types/diagnostic.md#L44-L48](https://github.com/microsoft/language-server-protocol/blob/112d978951f2223a962bf19516637862f587bf55/_specifications/lsp/3.18/types/diagnostic.md#L44-L48) |
| SARIF `result` holds `ruleId`, `level`, `message`, `locations`, `relatedLocations`, `stacks`, `codeFlows`, `fixes` and more | [A] | [sarif-schema-2.1.0.json#L2036-L2283](https://github.com/oasis-tcs/sarif-spec/blob/1742a253eb654bf8a4ed7c60eaef7412b2c23553/sarif-2.1/schema/sarif-schema-2.1.0.json#L2036-L2283) |
| SARIF `region`: `startLine`, `startColumn`, `endLine`, `endColumn`, `charOffset`, `charLength`, `byteOffset`, `byteLength`, `snippet`, `message`, `sourceLanguage` | [A] | [schema#L1707-L1789](https://github.com/oasis-tcs/sarif-spec/blob/1742a253eb654bf8a4ed7c60eaef7412b2c23553/sarif-2.1/schema/sarif-schema-2.1.0.json#L1707-L1789) |
| SARIF's cause chain, `exception.innerExceptions`, sits only in a `notification`, which concerns "the operation of the tool itself, as opposed to being relevant to an artifact being analyzed" | [A] | [SARIF 2.1.0 §3.58.1](https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/sarif-v2.1.0-errata01-os-complete.html#_Toc141791145) |
| Ant writes `failure` and `error` with `message`, `type` and the trace as text; `testcase` has `name`, `classname`, `time` | [A] | [XMLJUnitResultFormatter.java#L304-L325](https://github.com/apache/ant/blob/803411d1b9f9eae433abe555c580b4b05d5e80a8/src/main/org/apache/tools/ant/taskdefs/optional/junit/XMLJUnitResultFormatter.java#L304-L325) |
| Surefire's XSD gives `testcase` no `file` and no `line` | [A] | [surefire-test-report.xsd#L115-L119](https://github.com/apache/maven-surefire/blob/surefire-3.6.1/maven-surefire-plugin/src/site/resources/xsd/surefire-test-report.xsd#L115-L119) |
| pytest's `<failure>` has `message` and no `type`; only `xunit1` keeps `file` and `line`, and the default family is `xunit2` | [A] | [junitxml.py#L117-L126](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/junitxml.py#L117-L126), [#L199-L202](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/junitxml.py#L199-L202), [#L462-L467](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/junitxml.py#L462-L467) |
| Open Test Reporting, the format JUnit 5 writes, has `FilePosition { @line (required); @column }`. JUnit fills it with where the test is, not where it failed | [A] | [core-0.2.0.xsd#L108-L131](https://github.com/ota4j-team/open-test-reporting/blob/r0.3.0/schema/src/main/resources/org/opentest4j/reporting/schema/core-0.2.0.xsd#L108-L131); [OpenTestReportGeneratingListener.java#L261-L295](https://github.com/junit-team/junit-framework/blob/r6.1.3/junit-platform-reporting/src/main/java/org/junit/platform/reporting/open/xml/OpenTestReportGeneratingListener.java#L261-L295) |

## 2. The location

### What each column counts

The probes put the failure after `😀` (4 UTF-8 bytes, 2 UTF-16 units, 1 scalar), after a tab, and in a file that starts with a byte order mark.

| tool | offset or line and column | base | a column counts | end | a tab | a byte order mark |
|---|---|---|---|---|---|---|
| rustc JSON | both: `byte_start`, and `line_start` with `column_start` | bytes 0, lines and columns 1 | Unicode scalars | exclusive | 1 column; the human output prints 4 spaces | counted in `byte_start`, not in a column |
| ruff `TextRange` | byte offset | 0 | — | exclusive | — | counted: it is an offset into the text it was given |
| ruff `LineIndex::line_column` | line and column | 1 | Unicode scalars | — | 1 column | skipped, on line 1 only |
| CPython `SyntaxError` from the parser | line and column | 1 | code points | exclusive; `0` means no end | 1 | not counted |
| CPython `SyntaxError` from the compiler or symbol table | line and column | 1 | UTF-8 bytes | exclusive | 1 | [C] not probed |
| CPython `co_positions`, `FrameSummary.colno` | line and column | lines 1, columns 0 | UTF-8 bytes | exclusive | 1 | not counted |
| pytest | a line only | 0 in `location`, 1 in `ReprFileLocation` | — | — | — | — |
| mypy text output | line and column | 1 | its legacy parser: bytes; its native parser: neither | exclusive, printed raw | 1 | legacy: not counted; native: 2 columns short |
| mypy JSON | line and column | lines 1, columns 0 | as above | exclusive | 1 | as above |
| Rust panic `Location`, in nextest | text `file:line:col` | 1 | display width: `'\t' => 4`, `😀` 2 | none | 4 | not counted |
| Go compiler, vet, gofmt | text `file:line:col` | 1 | UTF-8 bytes | none | 1 | counted on line 1 |
| `go test` failure | text `file:line:` | 1 | no column | none | — | — |
| LSP `Position` | line and column | 0 | UTF-16 by default; `utf-8` or `utf-32` by negotiation | exclusive | the spec says nothing | the spec says nothing |
| SARIF `region` | both | lines and columns 1, offsets 0 | `columnKind`: `utf16CodeUnits` or `unicodeCodePoints` | exclusive | 1 column | values "SHALL NOT depend on" it |
| JUnit XML | none; pytest's `xunit1` `line` | 0 | — | — | — | — |

- **Only LSP can carry a column in bytes, and only by negotiation.** SARIF and JUnit XML cannot. A tool that keeps bytes must convert them to send a column through SARIF.
- **A column that counts display width depends on the terminal, not on the source.** A Rust panic counts a tab as 4. rustc counts a tab as 1 in its JSON, and prints 4 spaces in its human output.
- **A byte order mark splits the survey.** rustc keeps it in byte offsets and drops it from columns. ruff's `line_column` skips it on line 1, and ruff's `source_location` counts it. Go counts it in line 1's columns.

### Claims

| claim | tier | source |
|---|---|---|
| rustc's book: "Character offsets are offsets of Unicode Scalar Values."; `byte_end` "(0-based, exclusive)"; `column_end` "(1-based, exclusive)" | [A] | [src/doc/rustc/src/json.md#L14-L18](https://github.com/rust-lang/rust/blob/1.99.1/src/doc/rustc/src/json.md#L14-L18), [#L73-L84](https://github.com/rust-lang/rust/blob/1.99.1/src/doc/rustc/src/json.md#L73-L84) |
| rustc measured: after `😀`, `1i32` is at byte 43 and `column_start` 41. 40 scalars come before it. UTF-16 would give 42 and bytes 44 | [A] | P-rustc-1 |
| rustc's `normalize_src` removes the BOM and turns `\r\n` into `\n`, and records each change in `NormalizedPos { pos, diff }`. `byte_start` adds the difference back | [A] | [rustc_span/src/lib.rs#L2551-L2600](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_span/src/lib.rs#L2551-L2600), [#L2422-L2437](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_span/src/lib.rs#L2422-L2437); P-rustc-3 |
| rustc's book says JSON `text` is "The entire line of the original source code." The probes show the normalized line, without the BOM and without `\r` | [A] | [json.md#L106](https://github.com/rust-lang/rust/blob/1.99.1/src/doc/rustc/src/json.md#L106); P-rustc-3 |
| rustc's human output replaces a tab: `('\t', "    "), // We do our own tab replacement` | [A] | [rustc_errors/src/emitter.rs#L432-L503](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/emitter.rs#L432-L503) |
| ruff's `TextSize`: "This is a UTF-8 bytes offset stored as `u32`"; `TextRange::contains`: "The end index is considered excluded." | [A] | [ruff_text_size/src/size.rs#L11-L27](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_text_size/src/size.rs#L11-L27), [range.rs#L150-L167](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_text_size/src/range.rs#L150-L167) |
| ruff's `LineColumn` column is "The column (UTF scalar values) relative to the start of the line except any potential BOM on the first line." | [A] | [ruff_source_file/src/lib.rs#L223-L233](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_source_file/src/lib.rs#L223-L233) |
| `line_column` is `source_location(.., PositionEncoding::Utf32)` with 1 taken off on line 0 when the content starts with `'\u{feff}'` | [A] | [ruff_source_file/src/line_index.rs#L72-L131](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_source_file/src/line_index.rs#L72-L131) |
| ruff measured: `)` after `😀` at byte 11 gives `line_column` 1:9, Utf8 12, Utf16 10, Utf32 9 | [A] | P-ruff-1 |
| CPython's documentation: `offset` "is 1-indexed: the first character in the line has an ``offset`` of 1" | [A] | [Doc/builtins/exceptions.rst#L544-L566](https://github.com/python/cpython/blob/v3.15.2/Doc/builtins/exceptions.rst#L544-L566) |
| The parser converts with `_PyPegen_byte_offset_to_character_offset`. The compiler passes `loc.col_offset + 1`, and the symbol table adds 1 too. Neither converts | [A] | [Parser/pegen_errors.c#L347-L362](https://github.com/python/cpython/blob/v3.15.2/Parser/pegen_errors.c#L347-L362); [Python/compile.c#L1239-L1252](https://github.com/python/cpython/blob/v3.15.2/Python/compile.c#L1239-L1252); [Python/symtable.c#L86-L88](https://github.com/python/cpython/blob/v3.15.2/Python/symtable.c#L86-L88) |
| Measured: `x = "😀"; return 5` gives `offset=13 end_offset=21`. Taken as bytes they select `b'return 5'`; taken as characters, `'urn 5\n'`. The same on 3.12.14, 3.13.15, 3.14.7 and 3.15.0 | [A] | P-cpython-1; P-purba-2 |
| CPython's own caret line then marks `urn 5` | [A] | P-cpython-5 |
| `co_positions()`: "Column information is 0-indexed utf-8 byte offsets on the given source line." | [A] | [Doc/reference/datamodel.rst#L1604-L1635](https://github.com/python/cpython/blob/v3.15.2/Doc/reference/datamodel.rst#L1604-L1635) |
| `traceback` converts bytes to characters only when it renders, with `_byte_offset_to_character_offset` | [A] | [Lib/traceback.py#L617-L619](https://github.com/python/cpython/blob/v3.15.2/Lib/traceback.py#L617-L619) |
| mypy prints `f"{file}:{line}:{1 + column}"` and then `end_column` with no `+1`. Its JSON writes `column` with no `+1` | [A] | [mypy/errors.py#L1033-L1037](https://github.com/python/mypy/blob/v2.4.1/mypy/errors.py#L1033-L1037); [error_formatter.py#L26-L30](https://github.com/python/mypy/blob/v2.4.1/mypy/error_formatter.py#L26-L30); P-mypy-7 |
| mypy's documentation: "(note that column offsets are 0-based)". The text output is 1-based | [A] | [docs/source/command_line.rst#L912-L918](https://github.com/python/mypy/blob/v2.4.1/docs/source/command_line.rst#L912-L918); P-mypy-1 |
| mypy's native parser takes ruff's `line_column`, which already counts scalars, and converts it again with `convert_column_to_codepoint`. After `😀` the column comes out 3 short; after a BOM, 2 short | [A] | [ast_serialize serialize_ast.rs#L568-L603](https://github.com/mypyc/ast_serialize/blob/v0.13.1/src/serialize_ast.rs#L568-L603); P-mypy-1, P-mypy-2 |
| rustc fills a panic's column from `caller.col_display` + 1; `char_width` is `'\t' => 4`, otherwise `unicode_width` | [A] | [rustc_const_eval/src/const_eval/machine.rs#L212-L224](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_const_eval/src/const_eval/machine.rs#L212-L224); [rustc_span/src/lib.rs#L2504-L2545](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_span/src/lib.rs#L2504-L2545) |
| `Location::column` says only "Returns the column from which the panic originated." It names no unit | [A] | [library/core/src/panic/location.rs#L272-L295](https://github.com/rust-lang/rust/blob/1.99.1/library/core/src/panic/location.rs#L272-L295) |
| Go's `token.Position.Column`: "column number, starting at 1 (byte count)" | [A] | [src/go/token/position.go#L26-L31](https://github.com/golang/go/blob/go1.27.3/src/go/token/position.go#L26-L31) |
| Go's compiler adds the BOM's width to the column on line 1: `bom/bom.go:1:25` against 22 with no BOM | [A] | [src/cmd/compile/internal/syntax/source.go#L157-L163](https://github.com/golang/go/blob/go1.27.3/src/cmd/compile/internal/syntax/source.go#L157-L163); P-go-4 |
| LSP: "If the server omits the position encoding in its initialize result the encoding defaults to the string value `utf-16`." | [A] | [lsp/3.17/general/initialize.md#L661](https://github.com/microsoft/language-server-protocol/blob/112d978951f2223a962bf19516637862f587bf55/_specifications/lsp/3.17/general/initialize.md#L661) |
| LSP: "the end position is exclusive." | [A] | [lsp/3.17/types/range.md#L3](https://github.com/microsoft/language-server-protocol/blob/112d978951f2223a962bf19516637862f587bf55/_specifications/lsp/3.17/types/range.md#L3) |
| SARIF: "The values of text properties SHALL NOT depend on the presence or absence of a byte order mark (BOM) at the start of the artifact." | [A] | [SARIF 2.1.0 §3.30.2](https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/sarif-v2.1.0-errata01-os-complete.html#_Toc141790937) |
| SARIF `columnKind` has two values and no default; the schema accepts a run that leaves it out | [A] | [SARIF 2.1.0 §3.14.27](https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/sarif-v2.1.0-errata01-os-complete.html#_Toc141790761); P-sarif-1, P-sarif-2 |
| ruff's SARIF output holds no `columnKind`, although §3.14.27 says the run "SHALL contain" it | [A] | [ruff_db/src/diagnostic/render/sarif.rs#L105-L110](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_db/src/diagnostic/render/sarif.rs#L105-L110); P-ruff-4 |
| pytest's JUnit `line` is `str(location[1])`, so it is 0-based: a test defined on line 10 shows `line="9"` | [A] | [junitxml.py#L159-L197](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/junitxml.py#L159-L197); P-pytest-5 |
| LSP says nothing about a byte order mark or a tab | [C] | a search of the 3.17 and 3.18 spec found none |

## 3. Where each field is collected

```
            the stage that finds it                          a later stage, from the source
            ───────────────────────                          ──────────────────────────────
rustc       Span, label, suggestion, ErrCode, args    ──▶    line, column, text, rendered
ruff        TextRange, ParseErrorType                 ──▶    line, column, source line
CPython     SyntaxError location, text                       character column, from bytes + the line
            cause chain, frames, locals, type         ──▶    the line, through linecache
pytest      live ExceptionInfo, until makereport      ──▶    caret, line text, rendered entries
mypy        message with types, code, scope           ──▶    the source line, for --pretty
nextest     exit status; the panic is stderr text     ──▶    the description, by a regex
go test     file:line, from runtime.Caller            ──▶    nothing; it is already text
```

| claim | tier | source |
|---|---|---|
| rustc's finding stage stores only spans and messages: `span_label`, `note`, `help`, `span_suggestion`, `code` | [A] | [diagnostic.rs#L607-L610](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/diagnostic.rs#L607-L610), [#L950-L983](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/diagnostic.rs#L950-L983) |
| rustc computes file, line, column and text when it emits, through `sm.lookup_char_pos(span.lo())` and `sm.span_to_lines` | [A] | [json.rs#L437-L507](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/json.rs#L437-L507) |
| rustc's macro backtrace, `expansion`, needs the span's hygiene data, which only a live session holds | [A] | [json.rs#L421-L435](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/json.rs#L421-L435) |
| ruff computes line and column when it renders: each output calls `source_code.line_column(range.start())` | [A] | [render/json.rs#L69-L86](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_db/src/diagnostic/render/json.rs#L69-L86) |
| ruff's parser never collects a semantic syntax error. "This checker is not responsible for traversing the AST itself". Its caller must implement `SemanticSyntaxContext`, an 18-method trait that needs a model of scopes | [A] | [semantic_errors.rs#L1-L5](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/semantic_errors.rs#L1-L5), [#L2540-L2656](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/semantic_errors.rs#L2540-L2656) |
| So `parse_unchecked_source` passes `return 1` at module level and `nonlocal y` with no binding: `errors=0 … has_valid_syntax=true`. ruff's linter reports them as F706 and PLE0117 | [A] | P-ruff-1; P-ruff-2; [checkers/ast/mod.rs#L773-L903](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_linter/src/checkers/ast/mod.rs#L773-L903) |
| `parse_unchecked*` keeps every error it recovers from; `parse_module` returns only the first | [A] | [ruff_python_parser/src/lib.rs#L112-L118](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/lib.rs#L112-L118), [#L289-L300](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/lib.rs#L289-L300) |
| CPython's parser stops at the first error: "Bail out if we already have an error set." Two broken `def`s give one error | [A] | [Parser/pegen_errors.c#L200](https://github.com/python/cpython/blob/v3.15.2/Parser/pegen_errors.c#L200); P-cpython-1 |
| CPython's compiler raises `'return' outside function`, and the symbol table raises `no binding for nonlocal 'q' found` | [A] | [Python/codegen.c#L2262-L2271](https://github.com/python/cpython/blob/v3.15.2/Python/codegen.c#L2262-L2271); [Python/symtable.c#L698-L702](https://github.com/python/cpython/blob/v3.15.2/Python/symtable.c#L698-L702) |
| For both, `text` is a fresh read of the file, `PyErr_ProgramTextObject(filename, lineno)`. With filename `<string>`, `text=None` | [A] | [Python/errors.c#L1955-L1972](https://github.com/python/cpython/blob/v3.15.2/Python/errors.c#L1955-L1972); P-cpython-1 |
| Only the live stage can collect the cause chain, a frame's locals, the exception's type and the instruction's position. A later stage can compute the line text through `linecache`, and a character column from a byte column and the line | [A] | [Lib/traceback.py#L391-L405](https://github.com/python/cpython/blob/v3.15.2/Lib/traceback.py#L391-L405), [#L841-L843](https://github.com/python/cpython/blob/v3.15.2/Lib/traceback.py#L841-L843); P-cpython-2 |
| pytest's live `ExceptionInfo` lives from `CallInfo.from_call` to `pytest_runtest_makereport`. `repr_failure` turns it into text before `pytest_runtest_logreport` | [A] | [runner.py#L236-L259](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/runner.py#L236-L259); P-pytest-1 |
| mypy's message, code, scope and import chain exist only when `report()` runs; `--pretty` reads the source line again when it formats | [A] | [mypy/errors.py#L628-L681](https://github.com/python/mypy/blob/v2.4.1/mypy/errors.py#L628-L681), [#L1090-L1111](https://github.com/python/mypy/blob/v2.4.1/mypy/errors.py#L1090-L1111) |
| Only the test's own panic hook can know a Rust panic's location; nextest finds the description again in the bytes of stderr when it reports | [A] | [error_description.rs#L28-L60](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/reporter/error_description.rs#L28-L60) |
| `go test` finds the location when `t.Errorf` runs, through `runtime.Callers`, and writes it as text at once | [A] | [testing.go#L801-L895](https://github.com/golang/go/blob/go1.27.3/src/testing/testing.go#L801-L895) |

- **Across the survey, the facts that only the finding stage has are the same.** They are the range, which span is primary, the label, the suggested fix, the code, the cause chain, and a frame's locals, type and code.
- **The line, the column and the source line can always be computed again from a range and the source text.** rustc and ruff do this. CPython's `traceback` does it from a byte column and `linecache`.
- **A later stage computes them again only if it has the same text that the finding stage read.** rustc normalizes the BOM and `\r\n` first. ruff's `line_column` and `source_location` disagree on line 1 of a file with a BOM.

## 4. More than one location

| tool | how a second location shows | tier | source |
|---|---|---|---|
| rustc | a span with `is_primary: false` and its own `label` in the same `spans`; a child note with its own spans; a macro's `expansion` chain | [A] | P-rustc-4; [json.rs#L475-L489](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/json.rs#L475-L489) |
| ruff linter | an `Annotation` with `is_primary: false`, drawn with `-`: `previous definition of `os` here` | [A] | P-ruff-3 |
| ruff parser | none; two errors on one line are two `ParseError`s, at 4:15 and 4:16 | [A] | P-ruff-1 |
| CPython | one location for each `SyntaxError`; a second location only in `msg`, such as `expected an indented block after 'if' statement on line 1`. A cause chain is a chain of exceptions, `__cause__` and `__context__`; a group holds `exceptions` | [A] | P-cpython-1; [exceptions.rst#L74-L82](https://github.com/python/cpython/blob/v3.15.2/Doc/builtins/exceptions.rst#L74-L82) |
| pytest | a chain is a list of `(ReprTraceback, ReprFileLocation, description)`; an `ExceptionGroup` falls back to one block of CPython's text | [A] | [_code/code.py#L1214-L1269](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/_code/code.py#L1214-L1269); P-pytest-1 |
| mypy | none; the other place is text, as in `Name "f" already defined on line 1`, or a note at the same position | [A] | [semanal.py#L7718-L7726](https://github.com/python/mypy/blob/v2.4.1/mypy/semanal.py#L7718-L7726); P-mypy-5, P-mypy-6 |
| cargo-nextest | none; of several panics, only the last match of the regex: "Look for the last instance to handle situations like proptest" | [A] | [error_description.rs#L310-L331](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/reporter/error_description.rs#L310-L331) |
| `go test` | none; each `t.Errorf` is its own `error` event, and nothing links them | [A] | P-go-2 |
| LSP | `relatedInformation[]`, a flat list of `{ location, message }` | [A] | [diagnostic.md#L126-L141](https://github.com/microsoft/language-server-protocol/blob/112d978951f2223a962bf19516637862f587bf55/_specifications/lsp/3.17/types/diagnostic.md#L126-L141) |
| SARIF | `relatedLocations[]`; `location.annotations[]`; `relationships[]`; `codeFlows`; `stacks` | [A] | [SARIF 2.1.0 §3.27.22](https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/sarif-v2.1.0-errata01-os-complete.html#_Toc141790910) |
| JUnit XML | none; Surefire's `rerunFailure` and `flakyFailure` are attempts, not locations | [A] | [surefire-test-report.xsd#L49-L111](https://github.com/apache/maven-surefire/blob/surefire-3.6.1/maven-surefire-plugin/src/site/resources/xsd/surefire-test-report.xsd#L49-L111) |

- **The static tools show a second place in the code, and the runtime tools show a second exception.** rustc, ruff's linter, LSP and SARIF keep a list of places with a message each. CPython and pytest keep a chain of exceptions, each with its own traceback.
- **No format in the survey carries both kinds as data.** SARIF's cause chain lives only on a tool's notification. LSP has no chain at all.

## 5. Across a process

| tool | the form between processes | what that form drops | tier | source |
|---|---|---|---|---|
| rustc → cargo | one JSON `Diagnostic` for each line on stderr; cargo wraps it as `{"reason":"compiler-message","message":…}` | `SuggestionStyle`, `sort_span`, `lint_id`, the span's hygiene, `ErrCode` as a number; cargo also drops "aborting due to" and strips ANSI from `rendered` | [A] | [json.rs#L300-L390](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/json.rs#L300-L390); [cargo src/compiler/mod.rs#L2248-L2288](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/compiler/mod.rs#L2248-L2288); P-rustc-7 |
| ruff | none: files lint on threads, `paths.par_iter()`; its cache keeps `FileCacheData { linted, formatted }` and "We don't cache files with diagnostics." | — | [A] | [ruff/src/commands/check.rs#L83](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff/src/commands/check.rs#L83); [ruff/src/cache.rs#L335-L361](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff/src/cache.rs#L335-L361) |
| ruff `--output-format json` | `code`, `location`, `end_location`, `message`, `fix`, `url`, `noqa_row`, `severity` | byte offsets, secondary annotations, every sub but the first help, tags; outside preview, `severity` is always `error` | [A] | [render/json.rs#L51-L121](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_db/src/diagnostic/render/json.rs#L51-L121); P-ruff-3 |
| CPython `pickle` | `(type, args, __dict__)` | `__traceback__`, `__cause__`, `__context__`, `__suppress_context__`; `__notes__` is kept. A `SyntaxError`'s location is kept, because it lives in `args` | [A] | [Objects/exceptions.c#L211-L218](https://github.com/python/cpython/blob/v3.15.2/Objects/exceptions.c#L211-L218); P-cpython-3; P-purba-3 |
| CPython `multiprocessing` and `concurrent.futures` | the exception, with the traceback as a string from `format_exception` | every frame as data; `concurrent.futures` sets `__traceback__ = None` so that the frames can be freed | [A] | [Lib/multiprocessing/pool.py#L57-L74](https://github.com/python/cpython/blob/v3.15.2/Lib/multiprocessing/pool.py#L57-L74); [Lib/concurrent/futures/process.py#L130-L149](https://github.com/python/cpython/blob/v3.15.2/Lib/concurrent/futures/process.py#L130-L149) |
| pytest-xdist | pytest's own `pytest_report_to_serializable`, a dict of builtins over an execnet channel | the live `ExceptionInfo`, the column, `CollectReport.result`, the classes `CollectErrorRepr` and `ReprTracebackNative`; a `user_properties` value that execnet cannot dump crashes the run | [A] | [xdist remote.py#L280-L298](https://github.com/pytest-dev/pytest-xdist/blob/v3.8.0/src/xdist/remote.py#L280-L298); [reports.py#L582-L651](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/reports.py#L582-L651); P-pytest-4, P-pytest-6 |
| mypy workers | `ModuleResult(interface_hash, error_lines: list[str])`, lines already formatted | everything but the text | [A] | [mypy/build.py#L5364-L5381](https://github.com/python/mypy/blob/v2.4.1/mypy/build.py#L5364-L5381) |
| dmypy | `{"out": …, "err": "", "status": …}`; the client counts errors by matching `": error:"` in the text | everything but the text | [A] source; [C] not run | [mypy/dmypy_server.py#L836-L841](https://github.com/python/mypy/blob/v2.4.1/mypy/dmypy_server.py#L836-L841) |
| mypy cache | `ErrorTuple`: `(path, line, column, end_line, end_column, severity, message, code)` | `import_ctx`, `local_ctx`, `blocker`, `parent_error`, the `ErrorCode` object | [A] | [mypy/cache.py#L77-L79](https://github.com/python/mypy/blob/v2.4.1/mypy/cache.py#L77-L79); P-mypy-10 |
| cargo-nextest | two OS pipes of bytes from each test process; libtest JSON writes stderr into `"stdout"`: "note the stdout field actually contains both stdout and stderr despite the name" | in libtest JSON: the exit code or signal, earlier attempts, `error_summary`, `output_error_slice` | [A] | [test_command.rs#L245-L300](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/test_command.rs#L245-L300); [structured/libtest.rs#L48-L56](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/reporter/structured/libtest.rs#L48-L56); P-nextest-2 |
| `go test` | the test binary writes text with control bytes, and `test2json` turns that text back into events | nothing structured exists to drop; stdout and stderr merge | [A] | [testing.go#L565-L570](https://github.com/golang/go/blob/go1.27.3/src/testing/testing.go#L565-L570); [cmd/test2json/main.go#L77-L82](https://github.com/golang/go/blob/go1.27.3/src/cmd/test2json/main.go#L77-L82); P-go-8 |
| LSP | `PublishDiagnosticsParams { uri, version?, diagnostics }`; a fix travels in a `textDocument/codeAction` reply | a cause, a stack, source text, help text; ruff's server maps secondary annotations to `related_information` only when the client supports it | [A] | [publishDiagnostics.md#L73-L91](https://github.com/microsoft/language-server-protocol/blob/112d978951f2223a962bf19516637862f587bf55/_specifications/lsp/3.17/language/publishDiagnostics.md#L73-L91); [ruff_server/src/lint.rs#L450-L503](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff_server/src/lint.rs#L450-L503) |
| SARIF | itself | a cause chain on a result; a column in bytes; help on the result, which sits on the rule | [A] | Section 1 |
| JUnit XML | itself | a column, an end, a code, a related location, a fix; a cause only as text | [A] | Section 1 |

- **Most tools turn a failure into text before it leaves the process that found it.** nextest, `go test`, mypy's workers, dmypy and `multiprocessing` send text, and the receiver parses that text again or shows it as it is.
- **Each structured form drops the secondary locations, except rustc's.** ruff's JSON keeps one location, and keeps the second only inside the message: `Redefinition of unused `os` from line 1: `os` redefined here`.
- **A live object never crosses a process.** pytest-xdist and `pickle` drop the traceback. A value that cannot be serialized crashes pytest-xdist, as P-pytest-6 shows; it is not dropped.

## 6. What purba can collect

All purba links are at `b64bb14`. "Today" is what the part keeps; "can collect" is what its inputs already hold.
⚠️ marks a field that no part of purba can collect today.

### By part

| part | what it holds | can collect today | keeps today | tier | source |
|---|---|---|---|---|---|
| parsing | `ruff_python_parser::parse_unchecked_source(&source, …)`, the `Parsed`'s `errors()`, `source: String`, `path` | every `ParseError` ruff recovers from: `error: ParseErrorType` (42 kinds, and its text) and `location: TextRange`, a UTF-8 byte range with an exclusive end. Also the file path and the source text | `has_syntax_error: bool` only | [A] | [crates/parsing/src/lib.rs#L135-L149](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/parsing/src/lib.rs#L135-L149); P-ruff-1 |
| parsing | the same | line and column, through `ruff_source_file::LineIndex::line_column`: 1-based, Unicode scalars. `ruff_source_file` is in `Cargo.lock` as a dependency of ruff's crates, not of any purba crate | — | [A] | [Cargo.toml#L19-L21](https://github.com/kalonji-tools/purba/blob/b64bb14/Cargo.toml#L19-L21); [Cargo.lock#L642-L645](https://github.com/kalonji-tools/purba/blob/b64bb14/Cargo.lock#L642-L645) |
| parsing | the same | `UnsupportedSyntaxError` only when given `ParseOptions::with_target_version`; purba gives none | — | [A] | [parser/options.rs#L33](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/parser/options.rs#L33) |
| selection | each `StmtFunctionDef`: `range`, `name: Identifier { range }`, `parameters`, each `ParameterWithDefault { range, parameter: Parameter { range, name, annotation } }`, `decorator_list` of `Decorator { range }`; inside the body, each `StmtReturn { range }` and `ExprYield { range }` | the range of each test's `def` and of its name; for a refused test, the range of each untyped parameter, of the `return` of a value, of the `yield`, of `async` | `TestId { path, function }` and `Refused { test, reasons: Vec<Reason> }`, no range | [A] | [crates/selection/src/lib.rs#L30-L33](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/selection/src/lib.rs#L30-L33), [#L63-L66](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/selection/src/lib.rs#L63-L66), [#L161-L193](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/selection/src/lib.rs#L161-L193); [ruff_python_ast/src/generated.rs#L9301-L9310](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_ast/src/generated.rs#L9301-L9310) |
| assertion | each `StmtAssert { range, test, msg }`; `rewrite` reads `assert.test.range()` and `assert.range()` | the range of each assert and of its condition; the offset that the mark `_purba_ran(i, ` adds on that line | `asserts: Vec<String>`, the text of each assert | [A] | [crates/assertion/src/lib.rs#L63-L101](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/assertion/src/lib.rs#L63-L101); [generated.rs#L9478-L9482](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_ast/src/generated.rs#L9478-L9482) |
| assertion | `Ran::__call__(index, value)` at run time | the index of the last assert that ran, and whether it held | the same | [A] | [assertion/src/lib.rs#L120-L130](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/assertion/src/lib.rs#L120-L130) |
| execution | the `PyErr` of a failed load, call or lookup | the type's name, `str()`, `__notes__`, `__cause__` and `__context__` as a chain, `ExceptionGroup.exceptions`, and each traceback entry's `co_filename`, `co_name`, `tb_lineno` and `co_positions()[tb_lasti // 2]`; for a `SyntaxError`, `filename`, `lineno`, `offset`, `end_lineno`, `end_offset`, `text` | `err.to_string()`, one string | [A] the fields, from CPython; [B] the PyO3 calls that read them | [crates/execution/src/lib.rs#L90-L92](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/execution/src/lib.rs#L90-L92), [#L247-L249](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/execution/src/lib.rs#L247-L249) |
| execution | a failed assert: `Kind::Failed { assert }` | the `AssertionError`'s message, from `assert x, msg` | the assert's text only; the message is dropped | [A] | [execution/src/lib.rs#L242-L246](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/execution/src/lib.rs#L242-L246) |
| dispatch | `Handed::Test(T, Result<Outcome, Error>)`, `NotLoaded(PathBuf, Error)`, `Blocked(T, PathBuf)` | the module's path; nothing of the failure itself, which it hands on | the same | [A] | [crates/dispatch/src/lib.rs#L47-L56](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/dispatch/src/lib.rs#L47-L56) |
| command | `arguments: &[String]` | the argument text and its index for a usage error; for a read error, the path and the `io::Error` | `Error::Usage` holds no field; `Error::Read(PathBuf, io::Error)` | [A] | [crates/command/src/lib.rs#L12-L21](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/command/src/lib.rs#L12-L21), [#L70-L81](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/command/src/lib.rs#L70-L81) |
| reporting | each error as `impl Display`, joined to one line by `one_line` | nothing of its own; it reads what it takes | — | [A] | [crates/reporting/src/lib.rs#L136-L142](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/reporting/src/lib.rs#L136-L142) |
| fixtures | `Inputs {}` | nothing yet | — | [A] | [crates/fixtures/src/lib.rs#L1-L10](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/fixtures/src/lib.rs#L1-L10) |

### By field

| field | the part that can collect it | the form | note |
|---|---|---|---|
| file | command, parsing, assertion, execution | the path as the tester typed it; `co_filename` | — |
| start and end | parsing (each syntax error); selection (each `def`); assertion (each assert); execution (each frame, a `SyntaxError`) | a byte range from ruff; 0-based byte columns from `co_positions`; a `SyntaxError`'s 1-based offsets | the two traps below |
| source text | parsing, assertion | `Module::source()`, the text after a BOM is removed; the text of each assert | execution holds the rewritten text, not the source |
| label at the location | selection, as the text of each `Reason` | text | ruff's parser has no label |
| note | execution | `__notes__: list[str]` | — |
| ⚠️ help | none | — | no part, and neither ruff's parser nor CPython, has one |
| ⚠️ suggested fix | none | — | ruff's `Fix` belongs to its linter, which purba does not use |
| ⚠️ error code | none | — | ruff's `ParseErrorType` variant and CPython's class name are kinds; neither is a stable code |
| severity | none, as a field | purba's kind of failure plays the part: an outcome, an invocation error, a project error or an internal error | [A failure's kind is what the part that first sees it observes](https://github.com/kalonji-tools/purba/blob/b64bb14/docs/decisions/a-failures-kind-is-what-the-part-that-first-sees-it-observes.md) |
| related location | selection, for a refused test; execution, through a cause chain | a range; a chain of exceptions | ⚠️ none from the parser: ruff names two errors as two errors |
| cause | execution | `__cause__`, `__context__`, `ExceptionGroup.exceptions` | — |
| ⚠️ the values in a failed assert | none | — | `Ran::__call__` gets only the condition's value, and records whether it held |
| ⚠️ a semantic syntax error's ruff range | none | — | ruff's checker needs a `SemanticSyntaxContext`; CPython finds these at load, with the compiler's byte offsets |

### Two traps for execution

**A `SyntaxError`'s offset counts code points or bytes, by the stage that raised it.**
In purba, `compile()` in assertion's `run` raises both kinds.
Once #671 lands, parsing refuses each error that ruff finds before load. The errors that ruff passes, such as `'return' outside function` and `nonlocal` with no binding, come from CPython's compiler and symbol table, and that kind counts bytes.
The class and its fields are the same for both kinds. Only `msg` tells them apart.
P-purba-2 measured it on 3.12.14: `offset 13`, `end_offset 21`, the bytes `b'return 5'` and the characters `'urn 5\n'`. [A]

**CPython's positions on an assert's line point into the rewritten text.**
Assertion turns `assert 1 / 0 == 1` into `assert _purba_ran(0, 1 / 0 == 1)`, and it inserts no newline, so every line number is right.
On that line, a column after the mark is shifted by the length of `_purba_ran(0, `, which is 14 bytes for index 0.
`FrameSummary.line` reads the file on disk, which is the original.
So the positions and the line disagree: `colno 25 end_colno 30 line 'assert 1 / 0 == 1'` on 3.12.14 and 3.14.7, against `colno 11 end_colno 16` for the same text not rewritten.
CPython 3.12 then draws a caret line with no carets, and 3.14 draws an empty one (P-purba-1). [A]
Only assertion knows where it put each mark, so only assertion can give the map that moves such a column back.

```
the file           ····assert 1 / 0 == 1
                              ^^^^^          colno 11..16, in the source
the rewritten text ····assert _purba_ran(0, 1 / 0 == 1)
                                            ^^^^^          colno 25..30, which CPython reports
```

### The byte order mark

Parsing removes a leading BOM before it parses: `source.remove(0)`.
So every ruff offset that parsing collects is an offset into the text without the BOM, which is 3 bytes short of the file's own offset on every line.
`line_column` on that text counts no BOM, which matches the column CPython reports. [A]
Source: [crates/parsing/src/lib.rs#L136-L141](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/parsing/src/lib.rs#L136-L141); P-ruff-1 (`line_column` 1:5 on a file with a BOM).

## 7. The cost

| what a part keeps | what it costs | does it break "an outcome is data" | tier | source |
|---|---|---|---|---|
| a `ParseError`, in parsing | a `ParseErrorType` and a `TextRange` of 8 bytes. `ParseError` derives no serde, so a part copies it into its own type to encode it | no: an enum and two `u32` | [A] | [error.rs#L11-L15](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/error.rs#L11-L15) |
| a `LineIndex`, for a line and a column | one `u32` for each line, built in two linear scans. A lookup is a binary search, and O(1) for the column of an ASCII file. It needs `ruff_source_file` as a direct dependency | no: integers | [A] | [line_index.rs#L14-L66](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_source_file/src/line_index.rs#L14-L66), [#L249-L258](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_source_file/src/line_index.rs#L249-L258) |
| the source text, for a later stage to compute again | parsing already holds `source: String`, and command hands the rewritten `text: String` to dispatch for each module. rustc pays this through its `SourceMap`, which keeps "The complete source code." of each file for the session | no: a string | [A] | [rustc_span/src/lib.rs#L1926-L1959](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_span/src/lib.rs#L1926-L1959); [dispatch/src/lib.rs#L24-L29](https://github.com/kalonji-tools/purba/blob/b64bb14/crates/dispatch/src/lib.rs#L24-L29) |
| the ranges of each `def` and each assert | two `u32`s for each, read from a tree that selection and assertion already walk | no | [A] | Section 6 |
| the `PyErr`, the exception, its `__traceback__` or a frame | a traceback keeps each frame alive, and each frame keeps its locals: "The frame's locals then reference the exception, which references its own traceback, which references the locals of all frames caught in the traceback." | **yes**: a live Python object. [The gate](https://github.com/kalonji-tools/purba/blob/b64bb14/docs/decisions/a-gate-refuses-each-invariant-a-tool-can-see.md) asks that an outcome encode to bytes, and `pickle` refuses a traceback: `TypeError cannot pickle 'traceback' object` | [A] | [Doc/reference/datamodel.rst#L2057-L2064](https://github.com/python/cpython/blob/v3.15.2/Doc/reference/datamodel.rst#L2057-L2064); P-cpython-3 |
| a `TracebackException`, CPython's form "to ensure that no references are held" | on 3.12.14 it pickles. From 3.13.3, `FrameSummary` holds `_code`, a live code object, and `pickle` fails with `TypeError cannot pickle code objects`. It also holds the class in `_exc_type`, and the exception's own `__notes__` list | **yes** from 3.13.3; purba supports 3.12 and later | [A] | [Lib/traceback.py#L1095-L1098](https://github.com/python/cpython/blob/v3.15.2/Lib/traceback.py#L1095-L1098); `_code` is absent at [v3.13.2](https://github.com/python/cpython/blob/v3.13.2/Lib/traceback.py) and present at [v3.13.3](https://github.com/python/cpython/blob/v3.13.3/Lib/traceback.py); P-purba-3; [pyproject.toml#L10](https://github.com/kalonji-tools/purba/blob/b64bb14/pyproject.toml#L10) |
| scalars and strings copied from the `PyErr` while execution holds it | the walk over the chain and the frames, under the GIL; a character column needs the line's text as well | no | [A] | P-cpython-2 |
| a frame's locals | `repr` strings when copied, as `capture_locals=True` gives `{'x': "'😀'"}`; a `repr` runs the tester's code | no as strings; yes as objects | [A] | P-cpython-2 |

- **Only a live Python object breaks the invariant, and every tool that crosses a process copies out of it first.** pytest makes repr objects before `logreport`. `concurrent.futures` sets `__traceback__ = None`. `multiprocessing` sends the traceback as a string.
- **Data that a later stage computes again costs the source text in that stage.** rustc keeps every file's source in its `SourceMap` for the session. ruff shares one `Arc` of the text among every diagnostic of a file. In purba, the stage that finds the range holds the text, so it can compute the line and the column at once and pass on integers.
- **The cost of a field is paid only where it is found.** No later part can collect a frame's position, a cause chain or a `SyntaxError`'s stage, because no later part holds the `PyErr`.

## Could not establish

Each row below is [C].

- **Whether CPython's byte offsets for a compiler `SyntaxError` are a known issue.** Three searches of the tracker found none.
- **Whether mypy's native parser's double conversion is a known issue.** A search of python/mypy and mypyc/ast_serialize found none.
- **LSP's handling of a byte order mark and a tab.** The spec says nothing about either.
- **The unit of Open Test Reporting's `FilePosition` column.** Neither its XSD nor JUnit's `FilePosition` states one.
- **ruff's CLI output at 0.17.3.** The probes ran 0.17.2, and 0.17.3 rewrote the renderers. The struct fields read at 0.17.3 match the 0.17.2 output.
- **nextest at 0.9.150.** The probes ran 0.9.136. Only the panic heuristic was compared across the two versions, and it matched.
- **Where nextest checks `NEXTEST_EXPERIMENTAL_LIBTEST_JSON`.** The check was not found in `nextest-runner`.
- **dmypy and mypy's parallel workers.** Both were read from source and never run.
- **What consumes `DiagInner`'s `Encodable` derive in rustc.** The derive itself is [A].
- **Whether `LineIndex::line_column` panics for an offset inside the 3 bytes of a BOM.** Not probed. For purba this cannot happen, because parsing removes the BOM first.
- **Whether any pytest path leaves a live `ExceptionInfo` in `TestReport.longrepr`.** The type allows it, and no probe produced one.
- **The unit of vet's column.** It is [B], through `go/token`'s "(byte count)".
- **The exact PyO3 calls that execution would use to read a traceback.** Execution reaches each field through the Python object that its `PyErr` holds. No probe ran through PyO3.
- **Whether a compiler `SyntaxError`'s offset counts a BOM.** No fixture combined the two.

## Appendix A: liveness and the versions read

Liveness is the default branch's last commit and the latest release. It is measured with `gh api repos/O/R/commits/<branch> --jq .commit.committer.date` and `gh api repos/O/R/releases/latest --jq '.tag_name+" "+.published_at'`, or the tags API where a project has no releases.

| project | default branch, last commit | latest release | read at | ran |
|---|---|---|---|---|
| rust-lang/rust | `main` 2026-10-29T01:15:43Z | 1.99.1, 2026-10-16 | 1.99.1 | rustc 1.99.0 |
| rust-lang/cargo | `master` 2026-10-29T01:14:58Z | no release; rust 1.99.1 carries cargo at `5f94df47`, 2026-08-28 | `5f94df47` | cargo 1.99.0 (`5f94df478`) |
| astral-sh/ruff | `main` 2026-10-29T03:51:41Z | 0.17.3, 2026-10-27 | 0.17.0 (`5b27c1cc`, purba's pin, byte-identical to the registry copy) and 0.17.3 | ruff 0.17.2; a Rust probe on the `=0.0.17` crates |
| python/cpython | `main` 2026-10-29T11:27:14Z | no GitHub releases; newest stable tag v3.15.2, 2026-10-25 | v3.15.2 | 3.12.14, 3.13.15, 3.14.7, 3.15.0 |
| pytest-dev/pytest | `main` 2026-10-27T08:23:02Z | 9.2.0, 2026-10-27 | 9.2.0 | 9.2.0 on Python 3.13.15 |
| pytest-dev/pytest-xdist | `master` 2026-10-22T13:53:27Z | v3.8.0, 2025-07-01 | v3.8.0 | 3.8.0, execnet 2.1.2 |
| python/mypy | `master` 2026-10-28T14:11:37Z | no GitHub releases; newest tag v2.4.1, commit 2026-10-28 | v2.4.1 | 2.4.1 (ast_serialize 0.13.1); 1.20.1 |
| mypyc/ast_serialize | `main` 2026-10-16T23:34:23Z | no GitHub releases; tag v0.13.1 | v0.13.1 | 0.13.1 |
| nextest-rs/nextest | `main` 2026-10-29T11:53:49Z | cargo-nextest-0.9.150, 2026-10-26 | cargo-nextest-0.9.150 | 0.9.136 |
| nextest-rs/quick-junit | not measured | quick-junit-0.8.0, 2026-09-10 | quick-junit-0.8.0 | — |
| golang/go | `master` 2026-10-29T09:47:57Z | no GitHub releases; go1.27.3 on go.dev, tag commit 2026-10-20 | go1.27.3 | go1.27.2 |
| microsoft/language-server-protocol | `gh-pages` 2026-10-27T03:13:31Z | no GitHub releases; 3.18 is "Current", "3.18.0 (06/04/2026)" | `112d9789`, the 3.17 and 3.18 files | — |
| oasis-tcs/sarif-spec | `main` 2026-10-15T16:05:52Z | no releases; SARIF 2.1.0 errata01, "28 August 2023" | `1742a253`; the OASIS HTML | check-jsonschema 0.37.2 |
| apache/ant | `master` 2026-10-07T16:46:01Z | no releases; tag rel/1.10.18, 2026-09-03 | rel/1.10.18 | — |
| apache/maven-surefire | `master` 2026-10-28T07:26:59Z | surefire-3.6.1, 2026-10-26 | surefire-3.6.1 | — |
| jenkinsci/junit-plugin | `master` 2026-10-29T09:24:59Z | 1455.v5c1b_eed56292, 2026-10-29 | the same | — |
| ota4j-team/open-test-reporting | `main` 2026-10-29T07:17:48Z | r0.3.0, 2026-10-23 | r0.3.0 | — |
| junit-team/junit-framework | not measured | r6.1.3, 2026-08-07 | r6.1.3 | — |

Each probe ran from `/home/snregales/.claude/jobs/f70fe198/tmp/research710/<tool>/`, outside any checkout of purba.
Python ran with `-I`.
Each tool not pinned in purba's `mise.toml` ran through `nix-shell -p …`, or through a virtual environment in that directory for pytest 9.2.0 and mypy 2.4.1.

## Appendix B: the probes

Output is shown as the probe printed it. `…` marks lines left out.

### P-purba-1: the rewritten assert's columns

`test_div.py` (`cat -A`): `def test_div():$` and `    assert 1 / 0 == 1$`. The script compiles the rewritten text under that file name, runs `test_div`, and prints the last `FrameSummary`.

```
$ nix-shell -p python312 --run 'python3 -I shift2.py'     # and python314
lineno 2 colno 25 end_colno 30 line 'assert 1 / 0 == 1'
  File "test_div.py", line 2, in test_div
    assert 1 / 0 == 1

ZeroDivisionError: division by zero
3.12.14
```

On 3.12.14 the line after the source line holds 22 spaces and no caret; the repository's whitespace hook strips them here. On 3.14.7 the first line is the same, and that line is empty. The same script on the text that was not rewritten gives `colno 11 end_colno 16`.

### P-purba-2: a compiler `SyntaxError` on Python 3.12

`f6.py`: `x = "😀"; return 5` (bytes `x = " 360 237 230 200 " ;   r e t u r n   5 \n`).

```
$ nix-shell -p python312 --run 'python3 -I p6.py f6.py'
3.12.14 'return' outside function 13 21 chars: 'urn 5\n' bytes: b'return 5'
```

### P-purba-3: `pickle` of a `TracebackException`

A `ValueError` raised `from` a `KeyError`, with one note.

```
$ nix-shell -p python312 --run 'python3 -I p7.py'
pickle TracebackException: ok
FrameSummary has _code: False
after pickle: cause None tb None notes ['a note']
3.12.14
$ nix-shell -p python313 --run 'python3 -I p7.py'
pickle TracebackException: TypeError cannot pickle code objects
FrameSummary has _code: True
after pickle: cause None tb None notes ['a note']
3.13.15
```

`grep -c "'_code'" Lib/traceback.py` at each tag: v3.12.14 `0`, v3.13.0 `0`, v3.13.1 `0`, v3.13.2 `0`, v3.13.3 `1`, v3.13.15 `1`, v3.14.0 `1`.

### P-rustc-1: the column's unit

`rustc --error-format=json --crate-type lib --emit=metadata -o /dev/null <f>.rs 2>&1 | head -1 | jq -c '.spans[]'`, trimmed.

```
a_emoji: pub fn f() { let _s = "😀"; let _y: u8 = 1i32; }
{"byte_start":43,"byte_end":47,"line_start":1,"column_start":41,"column_end":45,"is_primary":true,"label":"expected `u8`, found `i32`",…}
{"byte_start":38,"byte_end":40,"line_start":1,"column_start":36,"column_end":38,"is_primary":false,"label":"expected due to this",…}
b_tab:   pub fn f() {\tlet _y: u8 = 1i32; }
{"byte_start":26,"byte_end":30,"line_start":1,"column_start":27,"column_end":31,"is_primary":true,…}
f_eacute: … "é" …
{"byte_start":41,"byte_end":45,"line_start":1,"column_start":41,"column_end":45,"is_primary":true,…}
```

### P-rustc-3: a BOM and `\r\n`

```
c_bom:  357 273 277 pub fn f() { let _y: u8 = 1i32; }
{"byte_start":29,"byte_end":33,"line_start":1,"column_start":27,"column_end":31,"is_primary":true,…,"text":[{"text":"pub fn f() { let _y: u8 = 1i32; }",…}]}
e_crlf: pub fn f() {\r\n let _y: u8 = 1i32; }\r\n
{"byte_start":28,"byte_end":32,"line_start":2,"column_start":15,"column_end":19,"is_primary":true,…,"text":[{"text":" let _y: u8 = 1i32; }",…}]}
```

### P-rustc-4: a secondary label, a note, a suggestion and a macro

`g_multi.rs`: `fn h(_a: u8) {}`, `pub fn k() { h(1i32); }`, `pub fn w() { let x = 1; }`, `macro_rules! m { () => { let _z: u8 = 1i32; } }`, `pub fn g() { m!(); }`.

```
{"message":"mismatched types","level":"error","code":{"code":"E0308"},"spans":[{"byte_start":31,"byte_end":35,"line_start":2,"column_start":16,"column_end":20,"is_primary":true,"label":"expected `u8`, found `i32`",…},{"byte_start":29,"byte_end":30,"line_start":2,"column_start":14,"column_end":15,"is_primary":false,"label":"arguments to this function are incorrect",…}],"children":[{"message":"function defined here","level":"note","spans":[{"line_start":1,"column_start":6,"column_end":12,"is_primary":false,"label":"",…},{"line_start":1,"column_start":4,"column_end":5,"is_primary":true,"label":null,…}]},{"message":"change the type of the numeric literal from `i32` to `u8`","level":"help","spans":[{"line_start":2,"column_start":17,"column_end":20,"is_primary":true,"label":null,"suggested_replacement":"u8","suggestion_applicability":"MachineApplicable"}]}]}
```

### P-rustc-7: cargo's wrapper

`cargo build --message-format=json` in a new library crate whose `src/lib.rs` is the `a_emoji` line.

```
$ jq -c 'select(.reason=="compiler-message") | keys' cargo.json
["manifest_path","message","package_id","reason","target"]
```

`cargo build -v --message-format=json` shows rustc run with `--error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat`.

### P-ruff-1: purba's pinned parser

A Rust program built `--offline` against `ruff_python_parser`, `ruff_python_ast`, `ruff_source_file` and `ruff_text_size` at `=0.0.17`. For each file it calls `parse_unchecked_source(&source, PySourceType::Python)` and prints each error and its positions.

```
== emoji.py        (s = "😀" ))
    Display: Expected a statement at byte range 11..12
    location: 11..12  line_column(start)=1:9  line_column(end)=1:10  utf8.col=12 utf16.col=10 utf32.col=9
== tab.py          (x =\t))
    location: 4..5  line_column(start)=1:5  line_column(end)=1:6  utf8.col=5 utf16.col=5 utf32.col=5
== bom.py          (357 273 277 s = ))
    location: 7..8  line_column(start)=1:5  line_column(end)=1:6  utf8.col=8 utf16.col=6 utf32.col=6
== bad1.py         (def test_ok(): pass, then def test_bad1(: on line 4)
parse_unchecked_source: errors=2 unsupported=0 has_valid_syntax=false
    Display: Expected a parameter or the end of the parameter list at byte range 39..40
    location: 39..40  line_column(start)=4:15  line_column(end)=4:16 …
    Display: Expected `)`, found newline at byte range 40..41
    location: 40..41  line_column(start)=4:16  line_column(end)=5:1 …
parse_module: Err(Expected a parameter or the end of the parameter list at byte range 39..40)
== semantic.py     (return 1, then def f(): nonlocal y)
parse_unchecked_source: errors=0 unsupported=0 has_valid_syntax=true
parse_module: Ok
```

### P-ruff-2: the linter on the same files

`nix-shell -I nixpkgs=channel:nixpkgs-unstable -p ruff --run 'ruff check --output-format concise --no-cache --isolated <f>'`

```
ruff 0.17.2
emoji.py:1:9: invalid-syntax: Expected a statement
bom.py:1:5: invalid-syntax: Expected an expression
bad1.py:4:15: invalid-syntax: Expected a parameter or the end of the parameter list
bad1.py:4:16: invalid-syntax: Expected `)`, found newline
semantic.py:1:1: F706 `return` statement outside of a function/method
semantic.py:4:14: PLE0117 Nonlocal name `y` found without binding
```

### P-ruff-3: a secondary annotation, in `full` and in JSON

`f811.py`: `import os`, `import os`, an empty line, `os.getcwd()`.

```
F811 [*] Redefinition of unused `os` from line 1
 --> f811.py:2:8
  |
1 | import os
  |        -- previous definition of `os` here
2 | import os
  |        ^^ `os` redefined here
…
help: Remove definition: `os`
json: "code": "F811", "location": {"column": 8, "row": 2}, "message": "Redefinition of unused `os` from line 1: `os` redefined here", "fix": {"message": "Remove definition: `os`", …}
```

### P-ruff-4: SARIF from ruff

```
$ ruff check --output-format sarif … f401.py emoji.py
"region": {"endColumn": 10, "endLine": 1, "startColumn": 9, "startLine": 1} …
$ grep -c columnKind out_formats.txt
0
```

### P-cpython-1: `SyntaxError` fields

`compile(bytes, path, 'exec')` on each file, on 3.13.15, 3.14.7 and 3.15.0, with the same output.

```
f1_emoji.py SyntaxError msg='invalid syntax' lineno=1 offset=11 end_lineno=1 end_offset=17 text='x = "😀" + import\n'
    text[offset-1:end_offset-1] as chars = 'import'; as utf8 bytes = b' + imp'
f3_tab.py … lineno=2 offset=12 end_lineno=2 end_offset=18 text='\ty = "é" + import\n'
f4_bom.py … lineno=1 offset=11 end_lineno=1 end_offset=17 text='x = "é" + import\n'
f5_two_errors.py SyntaxError msg='invalid syntax' lineno=1 offset=15 end_lineno=1 end_offset=16 text='def test_bad1(:\n'
f6_return.py SyntaxError msg="'return' outside function" lineno=1 offset=13 end_lineno=1 end_offset=21 text='x = "😀"; return 5\n'
    text[offset-1:end_offset-1] as chars = 'urn 5\n'; as utf8 bytes = b'return 5'
f7_nonlocal.py SyntaxError msg="no binding for nonlocal 'q' found" lineno=2 offset=13 end_lineno=2 end_offset=23 text='    é = 1; nonlocal q\n'
    text[offset-1:end_offset-1] as chars = 'onlocal q\n'; as utf8 bytes = b'nonlocal q'
f8_indent.py IndentationError msg="expected an indented block after 'if' statement on line 1" lineno=2 offset=1 end_lineno=2 end_offset=5 text='pass\n'
f10_unclosed.py SyntaxError msg="'(' was never closed" lineno=1 offset=5 end_lineno=1 end_offset=0 text='x = (1,\n'
```

### P-cpython-2: runtime positions

`r1_runtime.py`: `def f():`, `    x = "😀"; y = x + 1`, `f()`.

```
FrameSummary: lineno=2 end_lineno=2 colno=20 end_colno=25 name='f' line='x = "😀"; y = x + 1' locals=None
tb_lineno=2 tb_lasti=8 frame.f_lineno=2 co_positions[tb_lasti//2]=(2, 2, 20, 25)
raw[colno:end_colno] (bytes)=b'x + 1'  line[colno:end_colno] (chars)=' 1'
capture_locals FrameSummary.locals: {'x': "'😀'"}
--- -X no_debug_ranges
FrameSummary: … colno=None end_colno=None … co_positions[tb_lasti//2]=(2, 2, None, None)
```

### P-cpython-3: `pickle`

```
before: tb= traceback cause= KeyError('k') context= KeyError('k') suppress= True notes= ['a note']
after pickle: tb= None cause= None context= None suppress= False notes= ['a note']
pickle traceback: TypeError cannot pickle 'traceback' object
pickle frame: TypeError cannot pickle 'frame' object
pickle TracebackException: TypeError cannot pickle code objects
SyntaxError.args = ('invalid syntax', ('<s>', 1, 11, 'x = "é" + import\n', 1, 17))
AssertionError args= ('msg',) attrs= ['add_note', 'args', 'with_traceback']
bare assert args= ()
```

These ran on 3.13.15 and 3.15.0. P-purba-3 shows that on 3.12.14 the `TracebackException` pickles.

### P-cpython-5: CPython's own caret for a compiler error

```
$ python3 -I fix/f6_return.py        # 3.15.0
    x = "😀"; return 5
                 ^^^^^
SyntaxError: 'return' outside function
```

### P-pytest-1, P-pytest-2: what reaches `logreport`

A `conftest.py` logs the live `ExceptionInfo` in a `pytest_runtest_makereport` wrapper, and the report in `pytest_runtest_logreport`.

```
makereport: test_loc.py::test_assert_after_emoji: … entry.lineno(0b)=1 colno=16 end_colno=31 has_tb_frame=True rep.longrepr=ExceptionChainRepr
logreport:  test_loc.py::test_note: … reprcrash=ReprFileLocation(…, lineno=20, message='RuntimeError: base message\nnote one\nnote two')
logreport:  test_loc.py::test_chain: … len(chain)=2
    chain item: ReprTraceback entries=['ReprEntry'] crash.lineno=11 descr='The above exception was the direct cause of the following exception:'
    chain item: ReprTraceback entries=['ReprEntry'] crash.lineno=13 descr=None
logreport:  test_syntax.py: type(longrepr)=CollectErrorRepr location=('test_syntax.py', None, 'test_syntax.py') attrs=['longrepr']
interact:   test_syntax.py: live value=CollectError args0type=ExceptionChainRepr cause=SyntaxError lineno=2 offset=15 end_lineno=2 end_offset=15 text='    x = "😀" + )'
```

### P-pytest-4: across pytest-xdist

`python -I -m pytest -p xdist -n 2 -p no:cacheprovider test_loc.py test_syntax.py test_bom.py`

```
out/: … logreport.controller.txt logreport.gw0.txt logreport.gw1.txt makereport.gw0.txt makereport.gw1.txt
test_loc.py::test_chain: … type(longrepr)=ExceptionChainRepr
  extra attrs=['$report_type', 'item_index', 'node', 'testrun_uid', 'worker_id']
test_syntax.py: type(longrepr)=str location=('test_syntax.py', None, 'test_syntax.py') attrs=None
```

No `makereport.controller.txt` was written. The JSON on the worker and on the controller differs only in xdist's keys.

### P-pytest-5: JUnit XML

`test_fail` is defined on line 10.

```
xunit2: <testcase classname="test_j" name="test_fail" time="0.001">
          <failure message="AssertionError: assert '😀' == 'y'&#10;  &#10;  - y&#10;  + 😀">…</failure>
xunit1: <testcase classname="test_j" name="test_fail" file="test_j.py" line="9" time="0.001">
```

### P-pytest-6: a property that cannot be serialized

`record_property("thing", Thing())`, then `assert False`.

```
local: FAILED test_obj.py::test_obj_property - assert False
-n 1:  INTERNALERROR> E             execnet.gateway_base.DumpError: can't serialize <class 'test_obj.Thing'>
       no tests ran in 0.19s
```

### P-mypy-1, P-mypy-2, P-mypy-7: mypy's columns

`--show-column-numbers --show-error-end --no-incremental`. In `emoji.py`, `x: int = "😀"; y: str = 1`, the second `1` is at character 23, byte 26.

```
native (default)     emoji.py:1:21:1:21   bom.py:1:8:1:8
--no-native-parser   emoji.py:1:27:1:27   bom.py:1:10:1:10
-O json              {"file": "emoji.py", "line": 1, "column": 20, "end_line": 1, "end_column": 21, …}
```

### P-mypy-5, P-mypy-6: another location as text

```
redef.py:3:1:3:20: error: Name "f" already defined on line 1  [no-redef]
note2.py:2:1:2:6: error: Unexpected keyword argument "b" for "h"  [call-arg]
note2.py:2:1:2:6: note: "h" defined in "pkg.other"
```

### P-mypy-10: errors replayed from the cache

```
LOG:  Metadata fresh for emoji: file emoji.py
emoji.py:1:10: error: Incompatible types in assignment (expression has type "str", variable has type "int")  [assignment]
```

### P-nextest-1, P-nextest-2: a panic's column, and libtest JSON

```
thread 'emoji' (2607395) panicked at src/lib.rs:6:20:      # after 😀: bytes 22, UTF-16 20, scalars 19
thread 'tab' (2607397) panicked at src/lib.rs:16:20:       # two tabs: 14 for each unit
thread 'ascii' (2607391) panicked at src/lib.rs:21:19:
thread 'bom::bom_first_line' (2607392) panicked at src/bom.rs:1:31:

$ NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 cargo nextest run --no-fail-fast --message-format libtest-json-plus -E "test(=emoji) | test(=eq)" --retries 1
{"type":"test","event":"failed","name":"probe::probe$emoji#2","exec_time":0.003758663,"stdout":"\nthread 'emoji' (2608531) panicked at src/lib.rs:6:20:\nafter emoji\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n"}
```

### P-go-2, P-go-3, P-go-4: `go test`

```
$ go test -json ./fail | jq -c 'del(.Time)'
{"Action":"output","Package":"probe/fail","Test":"TestEmoji","Output":"    fail_test.go:11: after emoji 😀\n","OutputType":"error"}
{"Action":"output","Package":"probe/fail","Test":"TestEmoji","Output":"        second line\n","OutputType":"error-continue"}
$ go test ./build
build/build_test.go:6:19: undefined: undefinedA        # after 😀: bytes 19, scalars 16
build/build_test.go:8:20: undefined: undefinedC        # three tabs count 3
$ go test ./bom
bom/bom.go:1:25: undefined: undefinedBOM               # 22 with no BOM
```

### P-go-8: the bytes that `test2json` reads

```
$ fail.test -test.v=test2json -test.run TestEmoji | cat -v
^V=== RUN   TestEmoji
^O    fail_test.go:11: after emoji M-pM-^_M-^XM-^@
        second line^N
^V--- FAIL: TestEmoji (0.00s)
```

### P-sarif-1, P-sarif-2: the schema and `columnKind`

```
$ check-jsonschema --schemafile sarif-schema-2.1.0.json p1.sarif     # results, no columnKind
ok -- validation done
$ check-jsonschema --schemafile sarif-schema-2.1.0.json p2.sarif     # "columnKind":"bytes"
p2.sarif::$.runs[0].columnKind: 'bytes' is not one of ['utf16CodeUnits', 'unicodeCodePoints']
```
