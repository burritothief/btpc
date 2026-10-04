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

After the first PyPI release, install the library in your Python environment:

```console
python -m pip install btpc
python -c "import btpc; print(btpc.__version__)"
```

Until publication, build a wheel from the checkout or download a validated release
candidate artifact. Install the wheel file with `python -m pip install
/path/to/btpc.whl`. The wheel contains the Python library and Rust extension. It
does not install the native `btpc` CLI.

The release workflow builds wheels for each CPython minor version on these platforms:

| Platform | Architectures | Requirement |
| --- | --- | --- |
| Linux | x86-64, AArch64 | glibc 2.28 or later |
| macOS | Intel, Apple Silicon | An OS version compatible with the wheel's platform tag |
| Windows | x86-64 | Standard CPython for x86-64 |

PyPy, musl Linux, Windows ARM64, and free-threaded wheels are outside this tested
wheel matrix. Use `--only-binary=:all:` when you want installation to fail if a
compatible wheel is absent. Without this option, pip can try to build the source
distribution. A source build requires a Rust toolchain at or above MSRV 1.85 and
the platform's compiler/linker tools. For a reproducible development build, use
the pinned toolchain in `rust-toolchain.toml`.

Long native operations release the GIL. Progress callbacks reacquire it. A
free-threaded CPython runtime must enable the GIL to import this extension. Use
threads in one main interpreter or separate processes; subinterpreters are not supported.

## Rust

Add `btpc-core` as a path dependency while working from the checkout. Registry
installation instructions will be added when the crate is published.
