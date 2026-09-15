{ pkgs, ... }:

# ARM B: devenv is the SYSTEM LAYER ONLY.
# It supplies what mise cannot: a C toolchain and a libpython to link against.
# It deliberately does NOT enable languages.rust, because mise owns every tool
# version in this arm. If devenv supplied rustc the arm would not be testing
# what it claims to test.
{
  languages.python = {
    enable = true;
    package = pkgs.python312;
  };

  packages = [ pkgs.stdenv.cc ];

  env.RUST_BACKTRACE = "1";
}
