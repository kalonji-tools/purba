{ config, pkgs, ... }:

# The development environment. `direnv` activates it through .envrc, and
# worktrunk's user-level post-switch hook enters it on every `wt switch`.
#
# ADMISSION RULE: a tool lands here in the same commit as the file that calls
# it, and its entry names that file. oxitest's list carried twelve packages, of
# which six had no caller in the tree by the end — `hyperfine` for a benchmark
# workflow #11 refused outright, `mdbook` and `mdbook-mermaid` for an internals
# book #28 replaced with MkDocs, and `cargo-insta`, `cargo-mutants` and `fzf`
# for testing and query recipes nothing on the map has admitted. A package with
# no caller is not inert: it is a claim that the project does that thing.
#
# What that rule defers, and to where:
#   `just`       -> Write the justfile (#40)
#   `prek`       -> Write the prek config (#41)
#   `uv`         -> Write the prek config (#41), if it is needed at all
#   `git-cliff`  -> Write the release workflow and git-cliff config (#43)
#   `actionlint` -> Write the quality workflow (#36)
#
# uv is deferred rather than dropped, and the reason is a version, not a
# preference. Its only prospective caller is prek's hook environment, and
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
