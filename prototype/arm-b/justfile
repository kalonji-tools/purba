# ARM B — just alone. mise supplies the tools; just never resolves one.

_default:
    @just --list

# Build the wheel
build:
    maturin build

# Rewrite formatting
fmt:
    cargo fmt

# Refuse unformatted code
fmt-check:
    cargo fmt --check

# Refuse a lint
clippy:
    cargo clippy --all-targets -- -D warnings

# Refuse a broken intra-doc link (#28)
doc:
    cargo doc --no-deps

# Rust tests
test-rust filter="":
    cargo test {{filter}}

# Verify the examples in rustdoc (#28)
test-doc:
    cargo test --doc

# Every gate, in one command
check: fmt-check clippy doc test-rust test-doc

# Remove build output
clean:
    cargo clean
