# purba

A Python test framework written in Rust.

*purba* is Papiamentu for **to test, to try**.

## Status

**This is a standing scaffold and holds no product code.** The repository
carries tooling, working rules, dependencies, CI and a docs harness; the
framework itself is a separate effort that begins where the scaffold ends.

purba is the successor to [`oxitest`](https://github.com/kalonji-tools/oxitest),
which is frozen and kept as reference material. Nothing is ported: tooling is
copied wholesale, product code never is, and product *design* only where it is
still demonstrably the best answer.

## Prerequisites

purba needs a **C toolchain on the host** and does not supply one. Rust links
through it, and several dependencies compile C or assembly while they build.

| platform | what to install |
|---|---|
| Linux | your distribution's build tools, such as `build-essential` |
| macOS | the Xcode command line tools, `xcode-select --install` |
| Windows | the Visual Studio Build Tools, with the C++ workload |
| NixOS | `pkgs.gcc`, which supplies both `cc` and `ld` |

Nothing announces a missing compiler in advance. The build fails at the first
dependency that needs one, and it fails loudly.

Everything else comes from [mise](https://mise.jdx.dev), which installs the
Rust toolchain, Python, maturin and the rest from `mise.toml` and pins what
they resolved to in `mise.lock`.

```console
$ mise install
```

## Building

```console
$ maturin build
```

## License

purba is licensed under the [MIT License](LICENSE).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in purba shall be licensed as above, without any additional terms
or conditions.
