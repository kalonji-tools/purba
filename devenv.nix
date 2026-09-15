{ config, pkgs, ... }:

# A tool lands here in the same commit as the file it acts on, and its entry
# names that file. Acting on is not calling: a tool enters this file when its
# subject is in the tree, and a check enters a gate separately.

{
  languages.rust = {
    enable = true;

    toolchainFile = ./rust-toolchain.toml;

    # devenv aims RUST_SRC_PATH at this toolchain unconditionally, and
    # `toolchainFile` never sets `toolchain.rust-src`, so without this line the
    # variable falls back to nixpkgs' rustLibSrc and rust-analyzer indexes a
    # standard library from a different release than the rustc beside it.
    toolchain.rust-src = config.languages.rust.toolchainPackage;
  };

  # Python is here for the Rust build, and the tree holds no Python code.
  # `extension-module` is not a default feature, so a plain `cargo build` links
  # libpython and pyo3-ffi's build script fails without an interpreter.
  # https://pyo3.rs/v0.29.2/building-and-distribution.html
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
  ];

  env.RUST_BACKTRACE = "1";
}
