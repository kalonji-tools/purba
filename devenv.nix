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
#   `git-cliff`  -> Write the release workflow and git-cliff config (#43)
#   `actionlint` -> Write the quality workflow (#36)

let
  # Inherited from oxitest's substrate, not decided here. With zero Python
  # source in the tree there is nothing for a floor to protect, so this is the
  # interpreter PyO3 links against and nothing more. Whether the development
  # interpreter should instead sit on the abi3 floor (`requires-python =
  # ">=3.11"`) belongs to the first ticket that writes Python.
  python = pkgs.python312;
  pythonEnv = python.withPackages (ps: [ ps.pip ]);
in
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
    toolchain.rust-src = config.languages.rust.toolchainPackage;
  };

  languages.python = {
    enable = true;
    package = pythonEnv;

    # uv is the package manager, not a synchronised environment: `sync.enable`
    # installs the declared dependency groups, and pyproject.toml declares
    # none yet. It arrives with the first group, which is #41's — prek's hooks
    # find ruff, ty and codespell on PATH from this venv.
    uv.enable = true;
  };

  packages = with pkgs; [
    # The build backend pyproject.toml already names (`maturin>=1.9.3,<2`).
    maturin
  ];

  env = {
    RUST_BACKTRACE = "1";

    # Tell PyO3 which Python to link against.
    PYO3_PYTHON = "${pythonEnv}/bin/python3";
  };

  enterShell = ''
    # Put the uv tool bin and the devenv venv bin on PATH. Nothing in the tree
    # reads them yet; they are here because $UV_PROJECT_ENVIRONMENT is only
    # defined inside this shell, so the export cannot live anywhere else.
    export PATH="$HOME/.local/bin:$UV_PROJECT_ENVIRONMENT/bin:$PATH"
  '';
}
