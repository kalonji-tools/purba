{ config, pkgs, ... }:

# The development environment. `direnv` activates it through .envrc, and
# worktrunk's user-level post-switch hook enters it on every `wt switch`.
#
# ADMISSION RULE: a tool lands here in the same commit as the file it ACTS ON,
# and its entry names that file.
#
# ACTS ON, not calls — the two admissions are different, and conflating them is
# how a package list fills up. A tool enters this FILE when the thing it reads
# is in the tree. A check enters a GATE separately, by #21's four questions:
# valuable, no prek builtin, trigger lifetime, and decidable.
#
# `actionlint` is the worked example of the difference. `signoff.yml` exists,
# so its subject is here — but run against it today actionlint exits 1 on one
# SC2016 shellcheck finding about `'.[].filename'`, a jq filter that is
# single-quoted on purpose. A tool worth having; not yet a gate anyone should
# believe.
#
# oxitest's list carried twelve packages and six had no consumer in the tree by
# the end: `hyperfine` for a benchmark workflow #11 refused outright, `mdbook`
# and `mdbook-mermaid` for an internals book #28 replaced with MkDocs, and
# `cargo-insta` and `fzf` for snapshots and query recipes that were never
# written. A package with no consumer is not inert — it is a claim that the
# project does that thing.
#
# What the rule defers, and to where. The first four have no subject in the
# tree at all; the last three have one that arrived in an earlier commit, so
# their moment passed and the ticket that owns the subject owns the tool:
#   `just`          -> Write the justfile (#40)
#   `prek`          -> Write the prek config (#41)
#   `uv`            -> Write the prek config (#41), if it is needed at all
#   `git-cliff`     -> Write the release workflow and git-cliff config (#43)
#   `actionlint`    -> Write the quality workflow (#36), which owns `signoff.yml`
#   `bacon`         -> whichever ticket first watches `src/`, created by #35
#   `cargo-mutants` -> unowned; no ticket on the map admits mutation testing
#
# uv is deferred rather than dropped, and the reason is a version, not a
# preference. Its only prospective consumer is prek's hook environment, and
# nixpkgs already supplies all three tools those hooks need — measured here at
# ruff 0.16.3, ty 0.0.73, codespell 2.4.3 — so uv is not required to write
# them. What nixpkgs cannot do is pin ruff independently of nixpkgs: the CLI
# there is 0.16.3 while the parser crate this project builds on is `=0.0.12`,
# which is ruff 0.16.6. If #41 decides the linter must match the parser, a uv
# dependency group is how that is expressed, and uv arrives with it.

{
  languages.rust = {
    enable = true;

    # rust-toolchain.toml is the one place that names the toolchain, and it
    # resolves through rust-overlay rather than nixpkgs — nixpkgs *stable*
    # ships rustc 1.95.0, which cannot build ruff's crates at `=0.0.12`
    # (`rust-version = "1.96"`, #26).
    toolchainFile = ./rust-toolchain.toml;

    # devenv derives RUST_SRC_PATH from `toolchain.rust-src` and falls back to
    # nixpkgs' rustLibSrc when it is unset — and `toolchainFile` never sets it,
    # so rust-analyzer would index a standard library from a different release
    # than the rustc beside it. rust-toolchain.toml carries `rust-src` for
    # exactly this reason, so point it back at the pinned toolchain.
    #
    # This line TRUSTS rust-toolchain.toml to keep `rust-src` in `components`.
    # Drop it there and RUST_SRC_PATH is still set, to a directory that does
    # not exist — measured, with no error and no warning. Nothing checks that
    # the two files agree.
    toolchain.rust-src = config.languages.rust.toolchainPackage;
  };

  # Python is here for the Rust build, not for Python code — there is none.
  # `extension-module` is a non-default feature (#28), so a plain `cargo build`
  # links libpython, and without an interpreter pyo3-ffi's build script fails:
  # `error: no Python 3.x interpreter found`. Measured, not assumed.
  #
  # No `withPackages`, so no pip. That is a real capability dropped, not a
  # no-op: `python3 -m pip` reports no module, and `maturin develop` needs pip
  # or uv to install into an environment. `maturin build` does not, and there
  # is nothing to install yet. The installer arrives with the first ticket that
  # has something to install.
  #
  # It changes nothing for the Rust build. pyo3-build-config reads `lib_dir`
  # from the underlying CPython derivation, and all three wrappers seen while
  # writing this file resolved to the same one.
  #
  # 3.12 is inherited from oxitest's substrate, not decided. Whether the
  # development interpreter should instead sit on the abi3 floor
  # (`requires-python = ">=3.11"`) belongs to the first ticket that writes
  # Python.
  languages.python = {
    enable = true;
    package = pkgs.python312;
  };

  packages = with pkgs; [
    # The build backend pyproject.toml already names (`maturin>=1.9.3,<2`).
    maturin

    # Nix tooling. Its subject is this file and devenv.yaml, both of which
    # arrive in the same commit, which is what admits it. The set matches
    # SNROS, the one repo here that is Nix rather than merely built with it.
    #
    # All three were run against this tree before being added and all three
    # pass: `nixfmt --check`, `deadnix --fail` and `statix check` each exit 0.
    # They are admitted on a green baseline, so the first failure any of them
    # reports will be a real regression rather than inherited debt.
    #
    # None of them GATES anything yet — there is no prek hook and no CI step.
    # That is #41's and #36's to admit under #21's four questions, and SNROS
    # already shows the shape: `nixfmt --check`, `deadnix --fail`, and
    # `statix check` with `pass_filenames = false`.
    nixfmt
    deadnix
    statix

    # The Nix language server. It acts on the same files; it gates nothing and
    # never will, which is why no ticket owns it.
    nil
  ];

  # PYO3_PYTHON is deliberately absent. It named a second interpreter, and
  # measurement showed it bought nothing: with the variable set and unset,
  # pyo3-build-config resolved the same `lib_dir`, `lib_name`, `version` and
  # `shared`. Only `executable` differed, between two wrappers of one CPython.
  # Setting it is how the two could ever disagree, so PATH is the only source
  # and they cannot — which is #28's rule that a design where drift cannot
  # happen beats a check that detects it.
  env.RUST_BACKTRACE = "1";
}
