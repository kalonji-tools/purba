{ config, pkgs, ... }:

# Options and their defaults: https://devenv.sh/reference/options/
#
# A tool lands here in the same commit as the file it acts on, and its entry
# names that file. Acting on is not calling: a tool enters when its subject is
# in the tree, and a check enters a gate separately. A tool with no single
# subject names none.

{
  languages.rust = {
    enable = true;

    toolchainFile = ./rust-toolchain.toml;

    # rust-toolchain.toml carries rust-src, and this aims RUST_SRC_PATH at it
    # rather than at nixpkgs' rustLibSrc.
    toolchain.rust-src = config.languages.rust.toolchainPackage;
  };

  # Python is here for the Rust build, and the tree holds no Python code:
  # extension-module is not a default feature, so a plain `cargo build` links
  # libpython. https://pyo3.rs/v0.29.2/building-and-distribution.html
  languages.python = {
    enable = true;
    package = pkgs.python312;
  };

  packages = with pkgs; [
    # pyproject.toml
    maturin

    # devenv.nix, devenv.yaml
    nixfmt
    deadnix
    statix
    nil

    # No single subject: both are used throughout the project.
    git
    gh
  ];

  env.RUST_BACKTRACE = "1";
}
