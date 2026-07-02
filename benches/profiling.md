# Profiling BTPC

Profile optimized binaries against a deterministic dataset and retain the command,
tool versions, machine description, dataset manifest, cache label, and generated
torrent validation result with every capture. Never publish a profile from an
invalid output.

## Criterion microbenchmarks

Compile every benchmark without executing it:

```bash
cargo bench -p btpc-core --bench core --no-run
```

Run the complete baseline suite:

```bash
cargo bench -p btpc-core --bench core
```

Use a filter to isolate one kernel. Reduced timings are smoke checks only and are
not performance evidence:

```bash
cargo bench -p btpc-core --bench core -- bencode_parse \
  --warm-up-time 0.1 --measurement-time 0.2 --sample-size 10
```

Criterion fixture creation occurs before timed iteration. The manifest-sort
benchmark uses `iter_batched` so only the clone needed to provide owned input is
setup; all returned values are passed through `black_box`.

## macOS Instruments

Build once, locate the generated benchmark executable from `cargo bench --no-run`,
and profile it directly so Cargo startup is excluded:

```bash
cargo bench -p btpc-core --bench core --no-run
xcrun xctrace record --template 'Time Profiler' --output btpc.trace --launch -- \
  target/release/deps/core-<hash> --bench v1_piece_hashing
```

For end-to-end CLI creation, profile the release binary and preserve its output:

```bash
cargo build --release -p btpc-cli
xcrun xctrace record --template 'Time Profiler' --output create.trace --launch -- \
  target/release/btpc create benchmark-data/profile/payload.bin \
  --mode v1 --piece-length 4194304 --private \
  --tracker https://tracker.invalid/announce \
  --output benchmark-results/profile-v1.torrent --quiet
```

## Linux perf

Run the benchmark executable directly after compiling it:

```bash
cargo bench -p btpc-core --bench core --no-run
perf record -g -- target/release/deps/core-<hash> --bench v2_merkle_hashing
perf report
```

For flamegraphs, install `cargo-flamegraph` outside the repository and retain its
reported version. Do not add profiling tools as project dependencies:

```bash
cargo flamegraph --bench core --root -- v2_file_tree_creation
```

## Memory and CPU

The end-to-end harness under `benches/btpc_bench` samples aggregate process-tree
RSS and user/system CPU with `psutil`. On macOS, `/usr/bin/time -l` is also useful
for a single native process; on Linux use `/usr/bin/time -v`. Record which method
was used because their process-tree semantics differ.

For v1 concurrency comparisons, use `--threads 1` as the sequential correctness
oracle, `--threads 2` or another positive count for an explicit bounded pipeline,
and omit the flag (or use `--threads 0`) for the measured automatic heuristic.
