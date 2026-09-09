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

## Building

The toolchain is named in `rust-toolchain.toml` and is **not** available from
nixpkgs stable — rustc there is 1.95.0, and purba's pinned parser crates
require 1.96. Use the development shell.

```console
$ maturin build
```

## License

purba is licensed under the [MIT License](LICENSE).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in purba shall be licensed as above, without any additional terms
or conditions.
