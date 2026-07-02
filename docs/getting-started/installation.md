# Installation

BTPC is pre-1.0 and is not published to package registries yet. Install it from a
source checkout with the locked toolchains.

## CLI

```console
cargo build --release -p btpc-cli
./target/release/btpc --version
```

The binary is `target/release/btpc` (`btpc.exe` on Windows).

## Python

```console
uv sync --all-groups --locked
uv run maturin develop --release
uv run python -c "import btpc; print(btpc.__version__)"
```

Python supports CPython 3.11 through 3.14. Wheels currently require the CPython GIL
and do not support subinterpreters.

## Rust

Add `btpc-core` as a path dependency while working from the checkout. Registry
installation instructions will be added when the crate is published.
