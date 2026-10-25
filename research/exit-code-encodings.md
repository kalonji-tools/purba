# How checking tools name more than pass and fail in an exit code

This file answers [#653](https://github.com/kalonji-tools/purba/issues/653).
It feeds [#609](https://github.com/kalonji-tools/purba/issues/609), and builds on [`research/exit-codes.md`](https://github.com/kalonji-tools/purba/blob/9b3fcc44897ffff09a876f00034adf6fe88a81dd/research/exit-codes.md) of [#616](https://github.com/kalonji-tools/purba/issues/616), called "#616" below.
It is never merged.

## Answer

Checking tools use six ways to say more than pass and fail.
Most linters use three codes: clean, problems found, and the tool broke.
A few switch the meaning of a code with a flag, or use one bit for each kind, or keep a long table.
No tool in this survey puts a phase and a kind into the digits of one exit code, as the #609 sketch does.
Three tools group their codes in loose ranges, and promise nothing about a digit.
The two digit schemes found are protocol replies, HTTP and SMTP, and SMTP ran out of digits.
Every encoding that gives a non-zero code to a success has a documented case where an integrator read it as a failure.
Each documented break fell on a reader that compared an exact code, such as `== 1` or `< 8`.
The tools that explained a split told readers to test zero or not zero, or promised that every non-zero code stays an error.
The one tool that names a category and a kind for each failure, Bazel, puts them in a report beside the code.
Its report holds 576 kinds, and the exit code holds 17 values.
So the survey says: the exit code carries a coarse class that zero-or-not-zero readers get right, and a report carries the kind.
An exit code has 8 bits on Linux and in the shell, and Python wraps any larger integer, so `sys.exit(2**32)` exits 0.

| encoding | tools | what a zero-or-not-zero integrator reads | documented cost |
|---|---|---|---|
| three codes: clean, found, the tool broke | ruff, ESLint, mypy, shellcheck (with two usage codes) | right on every code | the tools draw the line between "found" and "broke" in different places |
| a flag that changes what a code means | `terraform plan -detailed-exitcode`, `git diff --exit-code` | a success with changes reads as a failure | a CI wrapper that passed `plan`'s 2 also passed `fmt`'s error 2 |
| a table of many codes | curl, rsync, git, cargo-nextest, Bazel, ansible | right, if no success is non-zero | renumbered and reused codes; a table that drifted from the source |
| one bit for each kind | pylint, fsck, robocopy, part of ansible | a success can read as a failure | shims (`pylint-exit`, `-lt 8`); a foreign 2 read as a bit |
| a range kept for the tool itself | `sysexits.h`, LSB, systemd, coreutils' `timeout`, Python's 120 | right | a wrapper that passes on its child's code collides with its own |
| a report beside the code | Bazel, Terraform, nextest, and the JSON of every linter | right | the report needs its own promise of stability |

[A] for each row, from the sections below.

## 1. The encodings

### 1.1 Three codes: clean, found, the tool broke

| tool | 0 | 1 | 2 | more | tier | source |
|---|---|---|---|---|---|---|
| ruff 0.15.14 | no violation | violations found | "invalid configuration, invalid CLI options, or an internal error" | — | [A] | [linter.md](https://github.com/astral-sh/ruff/blob/0.17.2/docs/linter.md?plain=1#L741-L759); [`ExitStatus`](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff/src/lib.rs#L38-L55); M1 |
| ESLint 10.13.0 | no error | an error, or more warnings than `--max-warnings` | "a configuration problem or an internal error" | — | [A] | [CLI docs](https://github.com/eslint/eslint/blob/v10.13.0/docs/src/use/command-line-interface.md?plain=1#L1037-L1043); M4 |
| mypy 1.20.1 | no error, or only notes | type errors | a blocking error: a syntax error, a missing file, a bad flag, or a crash | — | [A] | [`main.py`](https://github.com/python/mypy/blob/v2.4.0/mypy/main.py#L184-L187); [`errors.py`](https://github.com/python/mypy/blob/v2.4.0/mypy/errors.py#L1398-L1400); M1 |
| shellcheck 0.11.0 | no issue | issues found | "Some files could not be processed" | 3 bad syntax of the call, 4 bad options | [A] | [man page](https://github.com/koalaman/shellcheck/blob/v0.11.0/shellcheck.1.md?plain=1#L340-L348); [`statusToCode`](https://github.com/koalaman/shellcheck/blob/v0.11.0/shellcheck.hs#L200-L206); M1 |

- ruff names the convention: "This convention mirrors that of tools like ESLint, Prettier, and RuboCop." [A]
- POSIX `grep`, `diff`, `cmp` and `test` have the same shape: 0 yes, 1 no, more than 1 an error [B] (#616 item 5).
- The tools put the same case on different sides of the line (M1, M4) [A]:

| case | ruff | ESLint | mypy | shellcheck | pylint |
|---|---|---|---|---|---|
| the checked file does not parse | 1 | 1 | 2 | 1 | 2 (the "error" bit) |
| the checked file does not exist | 1 (`E902` as a violation) | 2 | 2 | 2 | 1 (the "fatal" bit) |
| unknown flag | 2 | 2 | 2 | 3 | 32 |
| config the tool cannot read | 2 | 2 | 0 (two warnings, then "Success") | — | — |

- ESLint keeps a file that does not parse on 1 unless asked: "Without this option, ESLint reports fatal parsing errors as rule violations" ([`--exit-on-fatal-error`](https://github.com/eslint/eslint/blob/v10.13.0/docs/src/use/command-line-interface.md?plain=1#L883-L885)) [A]. With the flag, it exits 2 (M4) [A].
- shellcheck combines the status of each file with `max` over its own order, not over the number ([`Semigroup Status`](https://github.com/koalaman/shellcheck/blob/v0.11.0/shellcheck.hs#L59-L72)) [A]. A missing file and a file with issues exit 2, not 1 (M1) [A].

### 1.2 A flag that changes what a code means

| tool | the flag | 0 | 1 | 2 | 3 | tier | source |
|---|---|---|---|---|---|---|---|
| `terraform plan` 1.17.0 | `-detailed-exitcode` | no changes | error | "Succeeded, there is a diff" | — | [A] | [help text](https://github.com/hashicorp/terraform/blob/v1.17.0/internal/command/plan.go#L277-L281); [`return 2`](https://github.com/hashicorp/terraform/blob/v1.17.0/internal/command/plan.go#L139-L146); M5 |
| `terraform plan` | none | success, with or without changes | error | — | — | [A] | M5 |
| `terraform fmt -check` | `-check` | formatted | bad flag | a file or directory it cannot read | not formatted | [A] | [`fmt.go`](https://github.com/hashicorp/terraform/blob/v1.17.0/internal/command/fmt.go#L64-L112); M5 |
| `git diff` 2.54.0 | `--exit-code` | no differences | differences | — | — | [A] | [diff-options.adoc](https://github.com/git/git/blob/v2.56.0/Documentation/diff-options.adoc?plain=1#L816-L818); M2 |
| `git ls-remote` | `--exit-code` | refs found | — | no matching ref | — | [A] | [git-ls-remote.adoc](https://github.com/git/git/blob/v2.56.0/Documentation/git-ls-remote.adoc?plain=1#L51-L54); M2 |
| LSB init script | the `status` verb | running | dead, pid file exists | dead, lock file exists | not running | [A] | [LSB 5.0 init script actions](https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/iniscrptact.html) |
| LSB init script | any other verb | success | generic error | invalid argument | unimplemented | [A] | same |

- In one tool, 2 is a success for `plan -detailed-exitcode` and an error for `fmt` (M5) [A].
- `fmt -check` documents only "non-zero otherwise" ([L577-L578](https://github.com/hashicorp/terraform/blob/v1.17.0/internal/command/fmt.go#L577-L578)); it exits 3 (M5) [A].

### 1.3 A table of many codes

| tool | codes | how they are grouped | its own usage error | tier | source |
|---|---|---|---|---|---|
| curl 8.22.0 | 87 listed, 0 to 100 | none; the tool's code is libcurl's error enum | 2, "Failed to initialize." | [A] | [`_EXITCODES.md`](https://github.com/curl/curl/blob/3163ec065643ba20709f190111f42c3e1fe95273/docs/cmdline-opts/_EXITCODES.md?plain=1#L3-L12); [`tool_operate.c`](https://github.com/curl/curl/blob/3163ec065643ba20709f190111f42c3e1fe95273/src/tool_operate.c#L2456-L2492); M2 |
| rsync 3.5.0 | 22 in the man page, 4 more in the source | blank lines in `errcode.h`: 1–5 start-up, 10–16 I/O and children, 19–25 signals and partial transfers, 30 and 35 timeouts | 1 | [A] | [rsync.1.md](https://github.com/RsyncProject/rsync/blob/v3.5.1/rsync.1.md?plain=1#L4947-L4972); [`errcode.h`](https://github.com/RsyncProject/rsync/blob/v3.5.1/errcode.h#L20-L64); M2 |
| git 2.54.0 | 0, 1 for "no", 128 for any fatal error, 129 for usage; more in some commands | by role, not by number | 129 from the option parser, 128 from a revision parser | [A] | [`usage.c`](https://github.com/git/git/blob/v2.56.0/usage.c#L60-L88); [`parse-options.c`](https://github.com/git/git/blob/v2.56.0/parse-options.c#L629); M2 |
| cargo-nextest 0.9.149 | 22 named constants, plus 1 and 2 | 4, 5 and 10 "advisory"; 80–96 update and setup; 100–110 build, run and output | 2 | [A] | [`exit_codes.rs`](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-metadata/src/exit_codes.rs#L4-L89) |
| Bazel 9.3.1 | 0–9, 32–39, 45, 48, some per command | 0–9 for the source and the command line; 32–39 for the environment and Bazel itself; 34, 35, 41–44, 47, 49 "Reserved for Google-internal use" | 2 | [A] | [scripts.md](https://github.com/bazelbuild/bazel/blob/9.3.1/site/en/run/scripts.md?plain=1#L43-L93); [`ExitCode.java`](https://github.com/bazelbuild/bazel/blob/9.3.1/src/main/java/com/google/devtools/build/lib/util/ExitCode.java#L23-L72) |
| ansible 2.21.5 | 0, 1, 2, 4, 5, 6, 99, 250 | 1, 2, 4 "must be bit-flag safe"; 250 an unexpected exception | 5 | [A] | [`ExitCode`](https://github.com/ansible/ansible/blob/v2.21.5/lib/ansible/errors/__init__.py#L24-L33); [`cli`](https://github.com/ansible/ansible/blob/v2.21.5/lib/ansible/cli/__init__.py#L663-L680) |

- Bazel names the split it makes: "Bazel attempts to differentiate failures due to the source code under consideration from external errors that prevent Bazel from executing properly." [A]
- ansible mixes two encodings in one space, and they collide: `PARSER_ERROR = 4  # FIXME: CLI-sourced, conflicts with HOST_UNREACHABLE` [A].
- git gives one mistake two codes: an unknown flag to `commit` exits 129, and to `log` exits 128 (M2) [A].
- curl, rsync, nextest and Bazel group codes, but none of them gives a digit a meaning. Nothing in their documents says that a code's tens digit names a phase [A].

### 1.4 One bit for each kind

| tool | bits | combined by | 0 is the only success? | tier | source |
|---|---|---|---|---|---|
| pylint 4.0.5 | 1 fatal, 2 error, 4 warning, 8 refactor, 16 convention, 32 usage | OR over every message | yes, so a convention message alone exits 16 | [A] | [run.rst](https://github.com/pylint-dev/pylint/blob/v4.1.3/doc/user_guide/usage/run.rst?plain=1#L176-L194); [`MSG_TYPES_STATUS`](https://github.com/pylint-dev/pylint/blob/v4.1.3/pylint/constants.py#L70); [`msg_status \|=`](https://github.com/pylint-dev/pylint/blob/v4.1.3/pylint/lint/pylinter.py#L1361); M1 |
| fsck (util-linux 2.42.4) | 1 corrected, 2 reboot, 4 left uncorrected, 8 operational error, 16 usage, 32 cancelled, 128 shared-library error | "the sum", and a bit-wise OR over file systems | no: 1 is "Filesystem errors corrected" | [A] | [fsck.8.adoc](https://github.com/util-linux/util-linux/blob/v2.42.4/disk-utils/fsck.8.adoc?plain=1#L29-L48) |
| robocopy | 1 copied, 2 extra files, 4 mismatched, 8 failure | the sum | no: "Any value equal to or greater than **8** indicates that there was at least one failure" | [A] | [robocopy.md](https://github.com/MicrosoftDocs/windowsserverdocs/blob/0f5dbfad59cfb66e3dfb348e29f94cb479ed7256/WindowsServerDocs/administration/windows-commands/robocopy.md?plain=1#L195-L209) |

pylint, measured (M1) [A]:

| case | code | bits |
|---|---|---|
| a convention message | 16 | convention |
| a convention, a warning and an error | 22 | 16 + 4 + 2 |
| a file that does not parse | 2 | error |
| a file that does not exist (`F0001`) | 1 | fatal |
| a checker that raises inside pylint (`F0002`) | 1 | fatal |
| an unknown `--output-format` | 1 | an uncaught traceback; the interpreter exits 1, which reads as "fatal" |
| score below `--fail-under`, no message | 1 | fatal: `sys.exit(self.linter.msg_status or 1)` ([run.py](https://github.com/pylint-dev/pylint/blob/v4.1.3/pylint/lint/run.py#L259-L274)) |
| unknown flag | 32 | usage |

- The "fatal" bit holds a missing file, a defect inside pylint, an uncaught exception and a low score [A].
- fsck's bit 128 lands in the shell's 128 + n range: fsck's 130 (128 + 2) is also the shell's code for SIGINT [A] (arithmetic on the two tables).
- pylint's six bits sum to at most 63, inside 0–125 [A].

### 1.5 A range kept for the tool itself

| range | owner | meaning | tier | source |
|---|---|---|---|---|
| 64–78 | `sysexits.h` | `EX_USAGE` … `EX_CONFIG`; the man page calls the interface deprecated | [B] | #616 item 5 |
| 0–7, then 8–99, 100–149, 150–199, 200–254 | LSB init scripts | the codes, then "reserved for future LSB use", "distribution use", "application use", "reserved" | [A] | LSB 5.0 init script actions |
| 200–246 | systemd v262.1 | systemd's own failure before `execve`, "under the assumption that they hence are unused by init scripts" | [A] | [`exit-status.h`](https://github.com/systemd/systemd/blob/v262.1/src/shared/exit-status.h#L13-L76); [systemd.exec.xml](https://github.com/systemd/systemd/blob/v262.1/man/systemd.exec.xml#L4808-L4818) |
| 124–127, 137 | GNU `timeout`, `env` (coreutils 9.12) | 124 timed out, 125 "itself fails", 126 cannot invoke, 127 not found; otherwise the child's code | [A] | [coreutils.texi](https://github.com/coreutils/coreutils/blob/v9.12/doc/coreutils.texi#L18455-L18470) and [L17708-L17716](https://github.com/coreutils/coreutils/blob/v9.12/doc/coreutils.texi#L17708-L17716); M3 |
| 124–127 | rsync, for its remote shell | "Although it doesn't seem to be specified anywhere", it copies the shell's values | [A] | [`errcode.h`](https://github.com/RsyncProject/rsync/blob/v3.5.1/errcode.h#L50-L64) |
| 125, 128 and above | `git bisect run` | 125 skips a commit; 128 or more aborts | [B] | #616 item 4 |
| 120 | CPython | "the exit status is changed to 120" when flushing the standard streams fails at exit | [A] | [sys.rst](https://github.com/python/cpython/blob/v3.15.1/Doc/library/sys.rst?plain=1#L552-L578); M3 |
| 259 | Windows | `STILL_ACTIVE`; "an application should not use STILL_ACTIVE (259) as an error code" | [A] | [GetExitCodeProcess](https://github.com/MicrosoftDocs/sdk-api/blob/656e9656bfaef34dd29cd7ee9e1146aec17e924c/sdk-api-src/content/processthreadsapi/nf-processthreadsapi-getexitcodeprocess.md?plain=1#L91-L102) |
| 4 | cargo-nextest | "4 is the smallest unreserved code" | [A] | [discussion #1646](https://github.com/nextest-rs/nextest/discussions/1646) |

- A wrapper that passes on its child's code collides with its own range. `timeout 5 sh -c 'exit 125'` exits 125, as `timeout` does when it fails itself (M3) [A].
- coreutils says so for 137: "these cases cannot be distinguished" [A].
- No source tree in this survey names `sysexits` (a grep of each clone) [A]. nextest's 70 equals `EX_SOFTWARE` [B] (#616), and systemd names the range in a comment [A].
- Two conventions meet at 125: coreutils' wrappers give it to "the tool itself fails", and `git bisect run` reads it as "this commit cannot be tested" [A] (both sources above; the link between them is my reading).

### 1.6 A report beside the code

| tool | the report | its link to the code | tier | source |
|---|---|---|---|---|
| Bazel | `FailureDetail` in the Build Event Protocol: 64 categories, each with an enum of kinds | each kind carries `exit_code` metadata: 576 kinds map onto 17 codes | [A] | [`failure_details.proto`](https://github.com/bazelbuild/bazel/blob/9.3.1/src/main/protobuf/failure_details.proto#L15-L89) |
| Terraform | `-json` output, and `terraform show -json` of a saved plan | both are in the v1 compatibility promise | [A] | [v1-compatibility-promises.mdx](https://github.com/hashicorp/web-unified-docs/blob/9c30178b0eaa2d78f8930f259bfd7b420fbb4a3f/content/terraform/v1.17.x/docs/language/v1-compatibility-promises.mdx?plain=1#L154-L157); M5 |
| cargo-nextest | machine-readable output, and `NextestExitCode` in the `nextest-metadata` library | the library follows semver | [A] | [stability policy](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/stability/index.md?plain=1#L10-L24) |
| curl | `-w '%{exitcode} %{errormsg}'` | the same number, with its text | [A] | M2: `22 404 The requested URL returned error: 404` |
| pylint, mypy, ruff, ESLint, shellcheck | `--output-format=json`, `-O json`, `--output-format=json`, `-f json`, `-f json1` | the code is the same with or without the report | [A] | M1, M4 |

- Bazel states the purpose: "The intent is to provide more detail to a Bazel client than is conveyed with an exit code, to help those clients decide how to respond to, or classify, a failure." [A]
- Bazel reduces a failure to "a pair of integers": the category, then the kind. It also warns: "This functionality is experimental and should not be relied on at this time." [A]
- Bazel keeps a fault of its own apart from a fault of the build, inside one kind: "Spawn OUT_OF_MEMORY leads to a BUILD_FAILURE exit_code because the build tool itself did not run out of memory." ([L218-L220](https://github.com/bazelbuild/bazel/blob/9.3.1/src/main/protobuf/failure_details.proto#L218-L220)) [A]

### 1.7 Digits that carry a class

No exit code in this survey gives a digit a fixed meaning [A] for the tools above.
I could not establish that no exit code anywhere does [C].
The digit schemes I found are protocol replies:

| scheme | first digit | second digit | rule for an unknown code | tier | source |
|---|---|---|---|---|---|
| HTTP, RFC 9110 | the class: 1xx to 5xx | "The last two digits do not have any categorization role." | "treat an unrecognized status code as being equivalent to the x00 status code of that class" | [A] | [RFC 9110 §15](https://www.rfc-editor.org/rfc/rfc9110#section-15) |
| SMTP, RFC 5321 | 2 success, 4 transient, 5 permanent | "The second digit encodes responses in specific categories": syntax, information, connections, mail system; x3z and x4z "Unspecified" | — | [A] | [RFC 5321 §4.2.1](https://www.rfc-editor.org/rfc/rfc5321#section-4.2.1) |
| SMTP enhanced codes, RFC 3463 | class.subject.detail, written in the reply text beside the 3-digit code | — | — | [A] | [RFC 3463 §1](https://www.rfc-editor.org/rfc/rfc3463#section-1) |

- HTTP gives meaning to the first digit only, and every client must apply the class rule [A].
- SMTP's second digit ran out. RFC 3463: "The most critical example is the existence of only 5 remaining codes for mail system errors." [A]
- Its fix was a structured code beside the old one, because "A revision to the SMTP theory of reply codes to better distribute the error conditions in the number space will necessarily be incompatible with SMTP." [A]
- RFC 3463 also names the damage: "SMTP suffers some scars from history, most notably the unfortunate damage to the reply code extension mechanism by uncontrolled use." [A]

## 2. What limits the range

| limit | what happens | tier | source |
|---|---|---|---|
| POSIX `exit()` | "only the least significant 8 bits (that is, status & 0377) shall be available from wait() and waitpid(); the full value shall be available from waitid()" | [A] | [POSIX.1-2024 `exit`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/exit.html) |
| Linux 7.3 | the kernel masks at the system call: `do_exit((error_code & 0xff) << 8);`, so `waitid` gets 8 bits too | [A] | [`kernel/exit.c`](https://github.com/torvalds/linux/blob/v7.3/kernel/exit.c#L1094-L1097); M3: `os._exit(300)`, then `waitid` gives `si_status 44` |
| the shell | 126 not executable, 127 not found, 128 + n killed by signal n | [A] | M3: 126, 127, SIGTERM 143, SIGINT 130, SIGKILL 137; POSIX [B] (#616 item 5) |
| the shell's `exit N` | N modulo 256: `exit 256` gives 0, `exit 300` gives 44, `exit -1` gives 255 | [A] | M3 |
| Windows | the code is the value passed to `ExitProcess` or returned by `main`; Rust's docs say `process::exit(0x0100)` gives "`0` on Linux, but `256` on Windows" | [A] the docs; [C] not measured, no Windows host | [GetExitCodeProcess](https://github.com/MicrosoftDocs/sdk-api/blob/656e9656bfaef34dd29cd7ee9e1146aec17e924c/sdk-api-src/content/processthreadsapi/nf-processthreadsapi-getexitcodeprocess.md?plain=1#L91-L98); [Rust `process.rs`](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/process.rs#L2528-L2540) |
| Rust's `ExitCode` | built only `From<u8>`; "Numeric values used in this type don't have portable meanings, and different platforms may mask different amounts of them." | [A] | [process.rs](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/process.rs#L2171-L2177) and [L2238-L2244](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/process.rs#L2238-L2244) |
| CPython's docs | "Most systems require it to be in the range 0--127, and produce undefined results otherwise." "Unix programs generally use 2 for command line syntax errors and 1 for all other kinds of errors." | [A] | [sys.rst](https://github.com/python/cpython/blob/v3.15.1/Doc/library/sys.rst?plain=1#L556-L567) |
| CPython's code | casts the integer to a C `int`; on overflow it uses -1 "to match historical Python behavior" | [A] | [`parse_exit_code`](https://github.com/python/cpython/blob/v3.15.1/Python/pythonrun.c#L600-L622) |

`sys.exit(N)` on CPython 3.13.15, Linux (M3) [A]:

| N | the shell reads | why |
|---|---|---|
| 255 | 255 | — |
| 256 | 0, a success | N modulo 256 |
| 300, 404 | 44, 148 | N modulo 256; 148 also reads as "killed by signal 20" |
| -1 | 255 | — |
| 2³¹ − 1 | 255 | — |
| 2³¹ | 0, a success | the cast to `int` gives −2³¹ |
| 2³² | 0, a success | the cast gives 0 |
| 2³² + 1 | 1 | the cast gives 1 |
| 2⁶³, 2⁶⁴ | 255 | overflow gives −1 |
| `"text"`, `1.5` | 1 | printed to stderr |
| `None` | 0 | — |
| `True` | 1 | `bool` is an `int` |

- `os._exit` gives the same low 8 bits: 256 gives 0, 300 gives 44 (M3) [A].
- A print to `/dev/full` that fails at exit gives 120 (M3) [A].
- purba returns a `u8` from Rust to Python, so it cannot pass a code above 255 ([`fn main(...) -> u8`](https://github.com/kalonji-tools/purba/blob/efb4c9c/crates/purba/src/lib.rs#L24)) [A].
- A console script passes the function's integer to `sys.exit` [B] ([the exit-code record](https://github.com/kalonji-tools/purba/blob/efb4c9c/docs/decisions/a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md?plain=1#L13)).

## 3. How each tool tells its integrator what a code means

| tool | a documented table | a promise about the codes | a structured report | tier |
|---|---|---|---|---|
| pylint | yes, in run.rst | none found | JSON | [A] table and report; [C] promise |
| mypy | none in the docs at v2.4.0; the meaning is in `main.py` and `errors.py` | none found | `-O json` | [A] grep of `docs/source`; [C] promise |
| ruff | yes, for `check` and for `format` | none found | JSON, SARIF and others | [A]; [C] promise |
| ESLint | yes, in the CLI page | each change is in a migration guide (item 4) | `-f json` | [A] |
| shellcheck | yes, in the man page | none found | `-f json1`, `checkstyle` | [A]; [C] promise |
| Terraform | in the help text and the docs | "The supported ways for external software to interact with Terraform are via the JSON output modes offered by some commands and via exit status codes." `-detailed-exitcode` is on the v1 list | `-json`, `show -json` | [A] [v1 promises](https://github.com/hashicorp/web-unified-docs/blob/9c30178b0eaa2d78f8930f259bfd7b420fbb4a3f/content/terraform/v1.17.x/docs/language/v1-compatibility-promises.mdx?plain=1#L390-L412) |
| curl | yes, 87 codes | "More error codes might appear here in future releases. The existing ones are meant to never change." | `-w '%{exitcode}'` | [A] [L201-L203](https://github.com/curl/curl/blob/3163ec065643ba20709f190111f42c3e1fe95273/docs/cmdline-opts/_EXITCODES.md?plain=1#L201-L203) |
| rsync | yes, in the man page; the source asks to keep it in step | none found | none found; the error line names the code and the source line, `(code 23) at main.c(1394)` | [A] table and line (M2); [C] promise and report |
| git | no single table; six pages have an EXIT STATUS section | none found | `--porcelain` outputs | [A] grep of `Documentation/*.adoc`; [C] promise |
| cargo-nextest | as constants in a library | the binary's public API is "append-only" within 0.9.x; a behaviour change gets a notice and "at least 3 months" | JUnit and libtest JSON | [A] [stability policy](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/stability/index.md?plain=1#L22-L47) |
| Bazel | yes | "Future Bazel versions may add additional exit codes, replacing generic failure exit code `1` with a different non-zero value with a particular meaning. However, all non-zero exit values will always constitute an error." | `FailureDetail` | [A] [scripts.md L91-L93](https://github.com/bazelbuild/bazel/blob/9.3.1/site/en/run/scripts.md?plain=1#L91-L93) |
| systemd | yes, in systemd.exec(5) | a retired code stays retired: "used to be tcpwrap, don't reuse!" | the journal | [A] code and table; [C] the journal not read |

- Bazel says why a code must stay put: "users, in particular infrastructure users, will use the exit code to decide whether the request should be retried or not." [A]
- Bazel's promise lets a code split only out of 1, and keeps every split non-zero [A].

## 4. Where a tool added, split or renumbered a code

| case | the change | what broke, or what the tool told its readers | tier | source |
|---|---|---|---|---|
| ESLint v5, 2018 | a fatal error moved from 1 to 2 | asked for in #9384: "eslint give the same exit code when some lint rules are not respected and when the binary is failing"; the guide said: "If you have an integration that detects all problems with linting runs by checking whether the exit code is equal to 1, update it to check whether the exit code is nonzero instead." | [A] | [migrating-to-5.0.0.md](https://github.com/eslint/eslint/blob/v10.13.0/docs/src/use/migrating-to-5.0.0.md?plain=1#L256-L267); [eslint#9384](https://github.com/eslint/eslint/issues/9384) |
| ESLint v5, 2018 | a missing file or an empty glob became fatal (2); v4 "exits without any errors" | a company's shared runner failed on projects with no `test` folder: "We don't want to implement custom code to check for pattern existence"; ESLint added `--no-error-on-unmatched-pattern` | [A] | [L100-L115](https://github.com/eslint/eslint/blob/v10.13.0/docs/src/use/migrating-to-5.0.0.md?plain=1#L100-L115); [eslint#10587](https://github.com/eslint/eslint/issues/10587) |
| ESLint v9, 2024 | no pattern now lints `.`; before, it linted nothing and exited 0 | "This was confusing because it wasn't clear that nothing had actually happened." | [A] | [migrate-to-9.0.0.md](https://github.com/eslint/eslint/blob/v10.13.0/docs/src/use/migrate-to-9.0.0.md?plain=1#L177-L182) |
| pylint 2.15, 2022 | a bad argument let argparse exit 2, the "error" bit; now 32 | "We were bitten by this causing us to think that pylint was completing successfully when in fact a typo in a configuration file was causing argparse to exit with exit code 2." | [A] | [pylint#7931](https://github.com/pylint-dev/pylint/pull/7931); [2.15 notes](https://github.com/pylint-dev/pylint/blob/v4.1.3/doc/whatsnew/2/2.15/index.rst?plain=1#L93-L96) |
| pylint and `pylint-exit` | `pylint-exit` 1.2.0 (2020) documents an unknown `--output-format` as 32 | pylint 4.0.5 exits 1, the "fatal" bit, for it (M1); `pylint-exit` maps 1 to failure, so the step still fails | [A] both; [C] when and why it changed | [README](https://github.com/jongracecox/pylint-exit/blob/v1.2.0/README.md?plain=1#L120-L129) |
| curl 7.62.0, 2018 | `CURLE_PEER_FAILED_VERIFICATION` moved from 51 to 60; 51 became `CURLE_OBSOLETE51` | commit "ssl: deprecate CURLE_SSL_CACERT in favour of a unified error code"; a script that tests 51 no longer matches | [A] the change; [C] a report of a broken script | [commit 3f3b26d](https://github.com/curl/curl/commit/3f3b26d6feb0667714902e836af608094235fca2); [curl.h](https://github.com/curl/curl/blob/3163ec065643ba20709f190111f42c3e1fe95273/include/curl/curl.h#L580-L590) |
| curl 7.24.0 and 7.38.0 | retired codes 10, 12 and 16 were given new meanings; 16 "was obsoleted in August 2007 for 7.17.0, reused in July 2014 for 7.38.0" | against its own page: "The existing ones are meant to never change." | [A]; [C] a report of a broken script | [curl.h L543-L545](https://github.com/curl/curl/blob/3163ec065643ba20709f190111f42c3e1fe95273/include/curl/curl.h#L543-L545), [L661-L666](https://github.com/curl/curl/blob/3163ec065643ba20709f190111f42c3e1fe95273/include/curl/curl.h#L661-L666) |
| git 2.56.0, 2026 | `-h` exits 0, "instead of 129" | 2.54.0 still exits 129 (M2) | [A]; [C] a report of a broken script | [RelNotes 2.56.0](https://github.com/git/git/blob/v2.56.0/Documentation/RelNotes/2.56.0.adoc?plain=1#L46-L49) |
| git 2.34.0, 2021 | — | "The xxdiff difftool backend can exit with status 128, which the difftool-helper that launches the backend takes as a significant failure, when it is not significant at all." | [A] | [RelNotes 2.34.0](https://github.com/git/git/blob/v2.56.0/Documentation/RelNotes/2.34.0.adoc?plain=1#L369-L372) |
| nextest 0.9.74 → 0.9.85, 2024 | no test run went from 0 to the advisory 4, after a `warn` default and a three-month notice | "some CI might break but it'll be correctly so" | [A] | [discussion #1646](https://github.com/nextest-rs/nextest/discussions/1646); [changelog](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/changelog.md?plain=1#L1217-L1224) |
| nextest 0.9.148, 2026 | a stress run exits non-zero if any iteration failed | before, "a stress run with failures in earlier iterations exited with code 0 if the last iteration passed" | [A] | [changelog L40](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/changelog.md?plain=1#L40) |
| rsync 3.0.6, 2009 | — | "a vanished directory could turn into an exit code 23 instead of the proper exit code 24"; rsync ships `rsync-no-vanished`, which turns 24 into 0 | [A] | [NEWS.md L2804-L2805](https://github.com/RsyncProject/rsync/blob/v3.5.1/NEWS.md?plain=1#L2804-L2805); [support script](https://github.com/RsyncProject/rsync/blob/v3.5.1/support/rsync-no-vanished) |
| rsync 3.4.3, 2026 | the table, not the codes | "Fixed the EXIT VALUES table: removed nonexistent code 6, added missing codes 15/16/19, corrected SIGUSR1 classification." | [A] | [NEWS.md L721](https://github.com/RsyncProject/rsync/blob/v3.5.1/NEWS.md?plain=1#L721) |
| setup-terraform, 2020–2023 | the GitHub Action's wrapper first turned every non-zero into 1, then passed 0 and 2 | #9: `plan -detailed-exitcode` printed 1, not 2; #328: `fmt`'s 2 is an error, so "a GitHub action running that command will succeed (if you use the wrapper) even though the command failed" | [A] the reports, the wrapper source, and `fmt`'s 2 (M5); [C] not run in Actions | [setup-terraform#9](https://github.com/hashicorp/setup-terraform/issues/9); [#328](https://github.com/hashicorp/setup-terraform/issues/328); [`terraform.js`](https://github.com/hashicorp/setup-terraform/blob/v4.0.1/wrapper/terraform.js#L46-L55) |
| Terraform, 2021 | a proposal to give each change kind a bit, "inspired by Unix file permissions" | closed as not planned; the maintainer pointed to the JSON plan for "I want my automation to behave differently depending on the content of the plan" | [A] | [terraform#30146](https://github.com/hashicorp/terraform/issues/30146) |
| robocopy in Azure Pipelines, 2016 | — | a copy that succeeded failed the task with exit code 1; a reply on the issue: "In cases where you need custom error handling in your task (0 == success is not sufficient) then handling in your task/ad-hoc script is the proper place to do it." | [A] | [azure-pipelines-tasks#1605](https://github.com/microsoft/azure-pipelines-tasks/issues/1605) |
| pytest 5, `unittest` 5, Jest | a new "no test" code broke CI, `make` and a pre-commit hook | the runner kept or eased its rule | [B] | #616 item 6 |

- Every change above kept 0 for success [A].
- Two tools told their readers how to survive a split: ESLint said to test non-zero, and Bazel promises that every non-zero code stays an error [A]. I found no such advice with the changes of curl, pylint or git [C].
- The breakage falls on readers of an exact code: a wrapper that tests `== 1`, `== 2` or `< 8` [A] (ESLint v5, setup-terraform, robocopy).
- A non-zero success broke a zero-or-not-zero reader for Terraform's 2 (setup-terraform#9) and robocopy's 1 (#1605) [A]. pylint's bits and rsync's 24 each got a shim that maps them back to 0 [A].

## What this means for purba

The #616 facts are: every integrator reads only zero or not zero, and `git bisect run` gives 125 and 128 or more their own meaning [B].

| encoding | a zero-or-not-zero integrator | `git bisect run` | the documented cost | with purba's record |
|---|---|---|---|---|
| three codes: clean, found, the tool broke | right | a tool fault reads as "bad", and bisect can blame the wrong commit | the line between "found" and "broke" moves from tool to tool (item 1.1) | the record already has five codes, and keeps a fault (3) apart from a failure (1) |
| a flag that changes what a code means | wrong for the non-zero success | the non-zero success reads as "bad" | a wrapper for one command broke another (setup-terraform#328) | the record gives 0 to a full pass only, so it needs no such flag |
| a table of many codes | right | right, below 125 | renumbering (curl 51 → 60), reuse (curl 16), a table that drifted (rsync 3.4.3) | — |
| one bit for each kind | wrong for every success that sets a bit | wrong for those too | shims; a foreign 2 read as a bit (pylint#7931); the "fatal" bit holds four causes | bits cannot hold "the worst of the run", which reporting renders |
| a range kept for the tool itself | right | right only for 125 and 128 or more | a wrapper that passes on its child's code collides (M3) | — |
| a report beside the code | right; the report is extra | right | the report needs its own promise | the data form belongs to reporting, out of the map's scope |
| digits for phase and kind, the #609 sketch | right | every 1x to 9x reads as "bad"; none reads as "skip" | no exit code in the survey does it; SMTP's digits ran out (RFC 3463) | fits in 8 bits, with the limits below |

Facts that bear on #609's sketch:

- **No tool in the survey gives a digit a meaning.** rsync, nextest and Bazel group codes in ranges, and leave gaps. None documents a digit rule an integrator could apply [A].
- **A digit class works in HTTP because every client must apply it.** RFC 9110 makes clients treat an unknown 471 as 400. No integrator in #616 applies a class rule to an exit code [B].
- **Ten kinds for each phase can run out.** SMTP had "only 5 remaining codes for mail system errors", and a fix in the digits "will necessarily be incompatible" [A].
- **The range is smaller than 0–125.** Python's interpreter owns 120 for a failed flush [A] (M3), and 1 for an uncaught exception [B] (the exit-code record). `sys.exit` wraps any larger code, and 2³² reads as success [A] (M3).
- **A phase 9x for "inside purba" reads as "bad" in `git bisect run`.** Two conventions give 125 to "the tool itself failed": coreutils' wrappers and bisect's skip [A]. The sketch gives no code to 125.
- **Two failures in one run need a rule.** shellcheck takes the worst by its own order, pylint ORs bits, and a Bazel `FailureDetail` holds one category [A]. The exit code already "names the worst of the run" ([reporting's record](https://github.com/kalonji-tools/purba/blob/efb4c9c/docs/decisions/reporting-renders-the-exit-code-and-command-returns-it.md?plain=1#L8)), so phases need an order [A].
- **Readers break when a code splits, unless they test zero or not zero.** ESLint said so when it split 1. Bazel promises that a split comes only out of 1 and stays non-zero [A].
- **The record's 2 means `unknown`.** ruff, ESLint and mypy use 2 for "the tool broke", and CPython's docs say "Unix programs generally use 2 for command line syntax errors" [A]. #616 found pytest's 2 is "interrupted" [B].
- **The kind can travel beside the code.** Bazel names 576 kinds in its report and exits with 17 codes. Terraform's maintainer refused a bit for each kind, and pointed to the JSON plan [A].

## Appendix A. Commands

Sources (S) and searches (C) were fetched this session, 2026-10-25.
`$P` is `/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/16dca681-fc06-4d1d-a235-dc886367a8a3/scratchpad/research-r1`.
`clone R T` is `git clone -q --depth 1 --branch T https://github.com/R.git $P/src/<name>`; each clone and download has its own directory.

| key | command |
|---|---|
| S1 | `clone pylint-dev/pylint v4.1.3`; `clone jongracecox/pylint-exit v1.2.0`; `gh api repos/pylint-dev/pylint/issues/7931` |
| S2 | `clone python/mypy v2.4.0`; `grep -rn -i "exit code\|exit status" docs/source` |
| S3 | `clone astral-sh/ruff 0.17.2` |
| S4 | `clone eslint/eslint v10.13.0`; `gh api repos/eslint/eslint/issues/{9384,10587}` |
| S5 | `clone koalaman/shellcheck v0.11.0` |
| S6 | `clone hashicorp/terraform v1.17.0`; `clone hashicorp/setup-terraform v4.0.1`; `curl -sfL https://raw.githubusercontent.com/hashicorp/web-unified-docs/9c30178b0eaa2d78f8930f259bfd7b420fbb4a3f/content/terraform/v1.17.x/docs/{cli/commands/plan.mdx,language/v1-compatibility-promises.mdx}`; `gh api repos/hashicorp/setup-terraform/issues/{9,328}`; `gh api repos/hashicorp/terraform/issues/30146` and `.../comments` |
| S7 | `clone curl/curl curl-8_23_0` (commit `3163ec0`); `gh api repos/curl/curl/pulls/2901`; `gh api repos/curl/curl/commits/3f3b26d6feb0` |
| S8 | `clone RsyncProject/rsync v3.5.1` |
| S9 | `clone git/git v2.56.0`; `grep -rln "^EXIT STATUS" Documentation/*.adoc` |
| S10 | `clone nextest-rs/nextest cargo-nextest-0.9.149`; `gh api graphql` for discussion 1646 (`repository{discussion(number:1646){title createdAt body}}`) |
| S11 | `clone systemd/systemd v262.1`; `curl -sfL https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/iniscrptact.html` |
| S12 | `curl -sfL https://raw.githubusercontent.com/coreutils/coreutils/v9.12/doc/coreutils.texi` |
| S13 | `curl -sfL https://raw.githubusercontent.com/bazelbuild/bazel/9.3.1/{site/en/run/scripts.md,src/main/java/com/google/devtools/build/lib/util/ExitCode.java,src/main/protobuf/failure_details.proto}`; counts: `grep -o "exit_code: [0-9]*" failure_details.proto \| sort \| uniq -c` |
| S14 | `clone ansible/ansible v2.21.5`; `clone util-linux/util-linux v2.42.4` |
| S15 | `curl -sfL https://raw.githubusercontent.com/MicrosoftDocs/sdk-api/656e9656bfaef34dd29cd7ee9e1146aec17e924c/sdk-api-src/content/processthreadsapi/{nf-processthreadsapi-exitprocess.md,nf-processthreadsapi-getexitcodeprocess.md}`; `gh api "repos/MicrosoftDocs/windowsserverdocs/contents/WindowsServerDocs/administration/windows-commands/robocopy.md?ref=0f5dbfad59cfb66e3dfb348e29f94cb479ed7256"`; `gh api repos/microsoft/azure-pipelines-tasks/issues/1605` and `.../comments` |
| S16 | `curl -sfL https://pubs.opengroup.org/onlinepubs/9799919799/functions/exit.html`; `curl -sfL https://raw.githubusercontent.com/torvalds/linux/v7.3/kernel/exit.c` |
| S17 | `curl -sfL https://raw.githubusercontent.com/python/cpython/v3.15.1/{Doc/library/sys.rst,Python/pythonrun.c}`; `curl -sfL https://raw.githubusercontent.com/rust-lang/rust/1.99.1/library/std/src/process.rs` |
| S18 | `curl -sfL https://www.rfc-editor.org/rfc/rfc{9110,5321,3463}.txt` |
| S19 | the purba tree at `efb4c9c`: `crates/purba/src/lib.rs`, `crates/command/src/lib.rs`, the exit-code record |
| C1 | `gh search issues "detailed-exitcode" --repo hashicorp/{setup-terraform,terraform}`; `gh search prs "CURLE_SSL_CACERT" --repo curl/curl`; `gh search commits "CURLE_PEER_FAILED_VERIFICATION" --repo curl/curl`; `gh search issues "exit code 51 60"` and `"error 51"` `--repo curl/curl`; `gh search issues "robocopy exit code" --repo microsoft/azure-pipelines-tasks`; `gh search issues "exit code 2" --repo eslint/eslint --created 2018-06-01..2019-06-30`; `gh search issues "exit code" --repo eslint/eslint --created 2017-09-01..2018-04-30` |
| C2 | `grep -rl sysexits` over the source directories of the pylint, mypy, ruff, ESLint, shellcheck, Terraform, curl, rsync, git, nextest and ansible clones: no match |
| M1 | `nix-shell -p 'python3.withPackages(ps: [ps.pylint ps.mypy])' ruff shellcheck --run 'bash py.sh $P/run/py'`; each case runs in its own fixture directory with `HOME`, caches and `PYLINTHOME` in scratch; plus `pylint --fail-under=11 m.py` on a module with one statement and no message |
| M2 | `nix-shell -p python3 curl rsync git --run 'bash tables.sh $P/run/tables'`; curl hits only a local `python -m http.server` on 127.0.0.1, `127.0.0.1:1` and `file://`; git runs with `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_NOSYSTEM=1` and git's repo variables unset |
| M3 | `nix-shell -p python3 coreutils --run 'bash range.sh $P/run/range'`: `sh -c "exit N"`, `python -I -c "import sys; sys.exit(N)"`, `os._exit(N)`, `os.waitid` after `os._exit(300)`, a print to `/dev/full`, the shell's 126, 127 and signals, `timeout` and `env` |
| M4 | `nix-shell -p nodejs --run 'bash js.sh $P/run/js'`; `npm install --no-save --no-audit --no-fund eslint@10.13.0` in `$P/run/js/pkgs`; flat config with `no-unused-vars: error` and `no-console: warn` |
| M5 | `curl -sfLO https://releases.hashicorp.com/terraform/1.17.0/terraform_1.17.0_{linux_amd64.zip,SHA256SUMS}`, `sha256sum -c`, then `bash tf.sh <terraform> $P/run/tf` with `CHECKPOINT_DISABLE=1`, `TF_CLI_CONFIG_FILE=/dev/null`; only the built-in `terraform_data` resource, so `init` downloads nothing |
| L | `gh api repos/O/R/commits/<branch> --jq .commit.committer.date`; `gh api repos/O/R/releases/latest --jq '.tag_name+" "+.published_at'`; with no releases, `git ls-remote --tags --refs`, then `gh api repos/O/R/commits/<tag> --jq .commit.committer.date` |

## Appendix B. Versions measured

Sources were read at the newest tag; measurements ran the version nixpkgs, npm or the vendor gave.

| tool | source read | version measured |
|---|---|---|
| pylint | v4.1.3 | 4.0.5, CPython 3.13.15 |
| mypy | v2.4.0 | 1.20.1 |
| ruff | 0.17.2 | 0.15.14 |
| ESLint | v10.13.0 | 10.13.0, node 24.21.0 |
| shellcheck | v0.11.0 | 0.11.0 |
| Terraform | v1.17.0 | 1.17.0, the release binary |
| curl | curl-8_23_0 | 8.22.0 |
| rsync | v3.5.1 | 3.5.0 |
| git | v2.56.0 | 2.54.0 |
| cargo-nextest | 0.9.149 | not run; #616 measured 0.9.136 |
| CPython | v3.15.1 | 3.13.15 |
| bash, coreutils | coreutils v9.12 | bash 5.3.9; coreutils from nixpkgs |
| Linux | v7.3 | the host kernel, 6.18 |
| Bazel, systemd, ansible, util-linux, robocopy | 9.3.1, v262.1, v2.21.5, v2.42.4, docs at `0f5dbfa` | not run |

## Appendix C. Liveness

| project | default branch, last commit | latest release or tag | date |
|---|---|---|---|
| pylint-dev/pylint | `main`, 2026-10-25 | release v4.1.3 | 2026-10-11 |
| jongracecox/pylint-exit | `master`, 2022-07-14 | release v1.2.0 | 2020-07-15 |
| python/mypy | `master`, 2026-10-25 | tag v2.4.0 (no GitHub releases) | 2026-10-01 (tag commit) |
| astral-sh/ruff | `main`, 2026-10-25 | release 0.17.2 | 2026-10-22 |
| eslint/eslint | `main`, 2026-10-25 | release v10.13.0 | 2026-10-16 |
| koalaman/shellcheck | `master`, 2026-10-19 | release v0.11.0 | 2025-08-04 |
| hashicorp/terraform | `main`, 2026-10-23 | release v1.17.0 | 2026-10-21 |
| hashicorp/setup-terraform | `main`, 2026-09-07 | release v4.0.1 | 2026-05-12 |
| hashicorp/web-unified-docs | `main`, 2026-10-23 | — (docs site) | — |
| curl/curl | `master`, 2026-10-25 | release curl-8_23_0 | 2026-10-14 |
| RsyncProject/rsync | `master`, 2026-10-23 | release v3.5.1 | 2026-09-21 |
| git/git | `master`, 2026-10-22 | tag v2.56.0 (no GitHub releases) | 2026-09-28 (tag commit) |
| nextest-rs/nextest | `main`, 2026-10-25 | release cargo-nextest-0.9.149 | 2026-10-20 |
| bazelbuild/bazel | `master`, 2026-10-24 | release 9.3.1 | 2026-10-21 |
| ansible/ansible | `devel`, 2026-10-23 | release v2.21.5 | 2026-10-05 |
| systemd/systemd | `main`, 2026-10-25 | release v262.1 | 2026-10-16 |
| coreutils/coreutils (public mirror) | `master`, 2026-10-25 | tag v9.12 | 2026-09-14 (tag commit) |
| util-linux/util-linux | `master`, 2026-10-21 | tag v2.42.4 (no GitHub releases) | 2026-09-21 (tag commit) |
| torvalds/linux | `master`, 2026-10-25 | tag v7.3 (no GitHub releases) | 2026-10-18 (tag commit) |
| python/cpython | `main`, 2026-10-25 | tag v3.15.1 (no GitHub releases) | 2026-10-21 (tag commit) |
| rust-lang/rust | `main`, 2026-10-25 | release 1.99.1 | 2026-10-16 |
| MicrosoftDocs/sdk-api | `docs`, 2026-10-09 | — (docs site) | — |
| MicrosoftDocs/windowsserverdocs | `main`, 2026-10-23 | — (docs site) | — |
| microsoft/azure-pipelines-tasks | `master`, 2026-10-23 | — (not read for a release) | — |
| LSB 5.0 (refspecs.linuxbase.org) | — | — | — |
| POSIX.1-2024 | — (The Open Group, Issue 8) | — | — |
| RFC 9110, RFC 5321, RFC 3463 | — (RFC Editor) | — | — |
