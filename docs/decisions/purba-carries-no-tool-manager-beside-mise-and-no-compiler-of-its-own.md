# purba carries no tool manager beside mise, and no compiler of its own

## Context and Problem Statement

purba needs one answer to the question of which version of a tool it uses.

The prototype purba succeeds had two answers and did not know it.
Its development shell resolved the Rust toolchain from a toolchain file.
Every one of its seven workflows resolved that toolchain again through rustup.
The file was the only thing holding the two readers in agreement, and a floating channel breaks that agreement silently.

Both candidate managers were asked for `nightly` on one machine inside one hour.

| reader | rustc | cargo |
|---|---|---|
| devenv, pinned by `devenv.lock` | `1.100.0-nightly (0fc141305 2026-09-11)` | `1.100.0-nightly (3c0b53475 2026-09-04)` |
| mise, pinned by `mise.lock` | `1.100.0-nightly (574ff7d98 2026-09-14)` | `1.100.0-nightly (7941be6fb 2026-09-11)` |

They disagree by four days on the compiler and by a week on cargo.

Three facts shape the rest of the choice.

- **A wheel is built per platform.** purba ships an abi3 extension, so the environment has to stand up on every architecture and operating system the wheel matrix covers.
- **Every repository here is driven by worktrees.** A per-worktree cost is paid every working day rather than once.
- **The two managers hold a toolchain by different mechanisms.** devenv wins the PATH. mise sets `RUSTUP_TOOLCHAIN`, which no PATH order can defeat.

That last difference decides how each one fails.
A gap in devenv's PATH coverage falls through to whatever rustup the developer already has.
The shell keeps working with the wrong compiler in it.
This was reproduced during the measurement.
A shell served a stable rustc beside a nightly rustfmt, with no warning, on a version the pinned parser crates cannot build.

## Considered Options

Four arrangements were built and run against the same criterion, a wheel that a Python interpreter then loads, on two architectures and three operating systems.

| arrangement | platforms | cold range |
|---|---|---|
| devenv alone | 3 of 6 | 53 s to 217 s |
| mise for versions, devenv for the system layer | 3 of 6 | 53 s to 217 s |
| mise with zig supplying the compiler | 6 of 6 | 21 s to 195 s |
| **mise alone** | **6 of 6** | **18 s to 69 s** |

- **devenv alone.** Rejected on reach and on cost. Windows is reachable only through WSL2, and nixpkgs 26.11 dropped x86_64-darwin outright, so Intel macOS fails at evaluation rather than at compilation. It is the slowest arrangement everywhere it does run, and it writes a directory into every worktree.

- **mise for tool versions, devenv for the system layer.** Rejected, though it works. It keeps the reach problem above, because the system layer is the half that cannot leave nix. It adds a second lockfile and a rule about which one owns what.

- **mise with zig supplying the compiler.** Rejected after being built and priced. zig reaches the same six platforms only by a fallback to the host compiler on three of them, and that fallback is silent. Forcing build scripts through zig broke aarch64 Linux on a Cortex-A53 linker argument that rustc emits by default. It cost the very wheel floor zig was bought for. On macOS zig produces exactly the tags the host toolchain produces unaided. Its one real gain is the Linux floor, `manylinux_2_17` against `manylinux_2_34`.

- **mise alone.** Chosen. Same reach as the zig arrangement and faster on every platform, with nothing in the repository beyond a lockfile. The C toolchain comes from the host, which every runner and every ordinary developer machine already carries.

## Decision Outcome

**purba carries no tool manager beside mise, and no compiler of its own.**
mise names the version of every package `mise.toml` declares, in one committed lockfile.
[One manifest declares each package](one-manifest-declares-each-package.md) says which packages `mise.toml` declares.

`mise.toml` names what purba accepts and `mise.lock` records what those names resolved to.
Both are committed.
A lockfile is generated rather than authored, so `.gitattributes` marks it `linguist-generated` and a reviewer is not shown its diff.
`Cargo.lock` and `mise.lock` both carry that mark.
mise chooses the platform list itself rather than being given one.

**purba requires a C toolchain on the host and does not supply one.**
This is a stated requirement rather than an omission, and `README.md` carries it.
That is the location a reader who does not yet know purba arrives at.
Every continuous integration runner already carries one.
A NixOS machine does not, and `pkgs.gcc` supplies both `cc` and `ld` there.

The Linux wheel floor is whatever the host provides.
Buying a lower floor is deferred to whatever publishes wheels, and zig is the measured way to buy it.

**Downside:**

- **A machine with no C compiler does not build purba at all.** Nothing detects that before the first build script fails.
- **The Linux floor is glibc 2.34 rather than 2.17.** It costs nothing today because nobody installs these wheels, and will cost something the day somebody does.
- **The toolchain's entry records a version and verifies none.** `core:rust` downloads no artifacts and so carries no checksum.
- **The entry for `bats` verifies nothing either.** `mise lock` records a URL for it and no checksum.
- **Nothing moves mise's own pinned version.** mise cannot pin itself, and no bot reads a workflow input.
- **The lockfile is not complete by default.** `mise lock` skips what it cannot fetch and reports success anyway, and an unauthenticated GitHub rate limit is enough to cause that. An entry carrying a stale tool option splits in two, and then fails on the platform that produced it.

## Confirmation

`mise install --locked`, run against an empty store.

It reproduces every tool from the committed lockfile, the Rust toolchain included.
The lockfile holds no Windows entry for `bats`.
The install on Windows skips that tool and reports nothing.
Run against a store that already holds them it reports "already installed" and resolves nothing, so a local pass there proves nothing at all.

⚠️ **A floating version name cannot be held by this lockfile.**
`core:rust` finishes an install with a symlink from its install path to `CARGO_HOME/bin`.
mise reads an absolute symlink outside its own directories as one a person made.
Such a version outranks the lockfile.
After the first rust install the lockfile is not consulted, and a floating name resolves against the channel again.
Two runs six minutes apart, on one commit and one lockfile, installed `nightly-2026-09-22` and then `nightly-2026-09-25`.

⚠️ **`locked = true` does not refuse that.**
Its not-in-lockfile error is withheld under the same condition, so an unlocked resolution is a silent difference rather than a failure.
**The setting covers a tool only while that tool's backend installs into a real directory.**
Every other tool in `mise.toml` satisfies that by accident, and no lockfile entry reveals it.

The toolchain is therefore named by a date, which resolves only to itself.
[The nightly is named by a date](the-nightly-is-named-by-a-date.md) holds that choice.

| property | check |
|---|---|
| the extension loads, rather than merely linking | ✅ `.github/workflows/build.yml`, on three operating systems and on every interpreter above the floor |
| a host compiler is present | nothing checks this. The build fails at the first build script, loudly |

The `build.yml` check is worth naming precisely.
`import purba` reaches a package whose first line is a star import of the extension.
The package's own file attribute reports the `__init__.py` and proves nothing.
A check written that way passes for a package with no Rust in it.
⚠️ **maturin writes that wrapper into every wheel it builds**, so this is the shape today and not a future one.
The check reads `purba.purba`, and it refuses if the wrapper is ever absent.
