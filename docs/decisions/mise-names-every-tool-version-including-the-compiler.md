# mise names every tool version, including the compiler

## Context and Problem Statement

purba needs one answer to the question of which version of a tool it uses.

The prototype purba succeeds had two answers and did not know it.
Its development shell resolved the Rust toolchain from a toolchain file.
Every one of its seven workflows resolved that toolchain again through rustup.
The file was the only thing holding the two readers in agreement, and a floating channel breaks that agreement without saying so.

Both managers were asked for `nightly` on one machine inside one hour.

| reader | rustc | cargo |
|---|---|---|
| devenv, pinned by `devenv.lock` | `1.100.0-nightly (0fc141305 2026-09-11)` | `1.100.0-nightly (3c0b53475 2026-09-04)` |
| mise, pinned by `mise.lock` | `1.100.0-nightly (574ff7d98 2026-09-14)` | `1.100.0-nightly (7941be6fb 2026-09-11)` |

The two disagree by four days on the compiler and by a week on cargo.

Three further facts shape the choice.

- **A wheel is built per platform.** purba ships an abi3 extension, so the environment has to stand up on every architecture and operating system the wheel matrix covers.
- **Every repository here is driven by worktrees.** A per-worktree cost is paid every working day, not once.
- **The two managers hold a toolchain by different mechanisms.** devenv wins the PATH. mise sets `RUSTUP_TOOLCHAIN`, which no PATH order can defeat.

That last difference decides how each one fails.
A gap in devenv's PATH coverage falls through to whatever rustup the developer already has, and the shell keeps working with the wrong compiler in it.
This was reproduced while measuring: a shell served a stable rustc beside a nightly rustfmt, with no warning, on a version the pinned parser crates cannot build.

## Considered Options

- **devenv alone.** Rejected on reach and on cost. It builds on three of the six platforms the wheel matrix needs. Windows is reachable only through WSL2, and nixpkgs 26.11 has dropped x86_64-darwin outright, so Intel macOS fails at evaluation rather than at compilation. It is also the slower environment everywhere it does run, and it writes a directory into every worktree.

- **mise for tool versions, devenv for the system layer.** Rejected, though it works. It keeps the exact reach problem above, because the system layer is the half that cannot leave nix. It adds a second lockfile and a rule about which one owns what. It buys a compiler on the three platforms that already have one, and supplies none on the three it cannot reach.

- **mise alone, with zig as the compiler.** Chosen. It builds and imports on all six platforms. The compiler stops being a layer and becomes an entry in the same lockfile as every other tool, which is what removes the second manager rather than merely shrinking it.

## Decision Outcome

<!--
  NOT WRITTEN BY AN AGENT.

  The sign-off workflow states on every record-touching pull request: "The
  Decision Outcome is written by a person, or the change does not merge. An
  agent writing it satisfies the letter and voids the rule."

  What this section has to state, with the measurements already in hand:

  1. The decision in one sentence. mise names every tool version, the compiler
     included, and purba carries no second environment manager.
  2. `mise.lock` is the pin, and it is committed.
  3. zig supplies the C toolchain through a `cc` shim the repository ships,
     because `--zig` redirects the target build and not the host one.
  4. Where zig declines a platform the build falls back to the host toolchain,
     and that fallback is silent today.

  Measured downsides for the **Downside:** line, which is required:

  - zig ran on three platforms of six. It declined arm64 Linux for reasons not
    established, and it declines both Windows targets because rustc drives the
    MSVC linker directly and never consults `cc`. The build still succeeds on
    the host compiler, so the arm passes while its distinguishing feature is
    absent. Nothing reports that today.
  - The wheel floor therefore differs by platform. Where zig runs it drops by
    years; where zig declines it is whatever the host provides.
  - The repository ships a `cc` shim, which is a file that exists to paper over
    a gap in another tool.
  - `core:rust` records a version and no artifact checksum, because it
    delegates to rustup. The date is pinned. The download is not verified.
  - The lockfile has to be generated with an explicit platform list. It is not
    complete by default, and an entry carrying a stale tool option splits in two
    and then fails on the platform that produced it.
-->

## Confirmation

`mise install --locked`, run against an empty store.

It reproduces the pinned toolchain and every other tool from the committed lockfile.
Run against a store that already holds them it reports "already installed" and resolves nothing, so a local pass there proves nothing at all.

Two properties are checked today only by hand, and the workflows that would run them are not written yet.

| property | check |
|---|---|
| the toolchain is the same everywhere | `rustc --version` agrees on every platform in the matrix |
| the extension loads, rather than merely linking | `import purba.purba` and assert its file ends in `.so`, `.pyd` or `.dylib` |

The second check is worth naming precisely.
`import purba` reaches a package whose first line is a star import of the extension, so the package's own file attribute reports the `__init__.py` and proves nothing.
A check written that way passes for a package with no Rust in it.
