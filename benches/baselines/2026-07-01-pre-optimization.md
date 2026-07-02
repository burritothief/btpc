# BTPC pre-optimization baseline — 2026-07-01

This is a local diagnostic baseline before todos 30–32. It is not a publication
benchmark or a competitor performance claim. The 8 MiB payload is intentionally
small enough for a repository smoke baseline; use the standard end-to-end harness
profile for release claims.

## Environment

- Working-tree identifier: no Git metadata was present in this checkout.
- Rust: `rustc 1.85.0 (4d91de4e4 2025-02-17)`, LLVM 19.1.7.
- Cargo: `cargo 1.85.0 (d73d2caf9 2024-12-31)`.
- BTPC: `btpc 0.1.0` release profile.
- Criterion: `0.7.0` (newer Criterion releases require Rust 1.86, above MSRV).
- OS: macOS 26.5 build 25F71; Darwin 25.5.0; arm64; APFS.
- CPU: Apple M3 Max, 14 physical/logical cores.
- Memory: 38,654,705,664 bytes.
- Python/resource sampler: Python 3.14.3, psutil 7.2.2.
- Cache label: warm; no operating-system cache flush was attempted.

## Dataset

- Path: ignored `benchmark-data/todo-29-baseline/payload.bin`.
- Generator: `python -m benches.btpc_bench generate`.
- Seed: `2901`.
- Size: 8,388,608 bytes.
- SHA-256: `f9eeffb73f61d028ef6f8ac456d820cd2089d1da1235682750a9723e0b9474db`.
- Piece length: 262,144 bytes; 32 pieces.
- Tracker: `https://tracker.invalid/announce`; private flag enabled.
- Raw manifests and measurements: ignored
  `benchmark-results/todo-29-baseline/`.

## Criterion smoke

Command:

```bash
cargo bench -p btpc-core --bench core -- \
  --warm-up-time 0.05 --measurement-time 0.1 --sample-size 10 --noplot
```

The deliberately short durations verify that every benchmark executes and provide
rough pre-optimization estimates only.

| Benchmark | Fixture | Estimate | Throughput |
|---|---:|---:|---:|
| `bencode_parse` | 84.9 KiB metainfo | 112.74 µs | 717.51 MiB/s |
| `bencode_encode` | same owned tree | 118.52 µs | 682.53 MiB/s |
| `manifest_sort` | 2,048 entries | 235.70 µs | 8.6892 Melem/s |
| `v1_piece_hashing` | 8 MiB, 256 KiB pieces | 3.8823 ms | 2.0123 GiB/s |
| `v2_merkle_hashing` | 8 MiB, 256 KiB pieces | 4.2380 ms | 1.8434 GiB/s |
| `v2_file_tree_creation` | 256 × 4 KiB files | 35.226 ms | n/a |

## End-to-end creation

Five warm-cache release runs per mode were measured with the psutil process-tree
sampler. The sampler polls at 10 ms, so one very fast process can exit before a
resource sample; maximum RSS and medians below use the observed nonzero samples.
All 15 torrents passed `btpc validate` and `btpc verify` against the payload.

| Mode | Median wall | Median user CPU | Median system CPU | Max peak RSS | Output | Info hash(es) |
|---|---:|---:|---:|---:|---:|---|
| v1 | 42.632 ms | 5.133 ms | 2.291 ms | 7,340,032 B | 834 B | v1 `11c2eff82caf896c344e287f632792953396709e` |
| v2 | 42.492 ms | 5.063 ms | 2.726 ms | 7,028,736 B | 1,363 B | v2 `7edc337e61ee3030886950122385c88903a0369df7e12a3973c3b22647193404` |
| hybrid | 42.435 ms | 5.746 ms | 2.377 ms | 7,307,264 B | 2,032 B | v1 `192046c342017130f811281d4656f1550e61389a`; v2 `91bec1297f6e9588b21dd0459bccfb376f953e25eb97a3086358f09e51f6cdaf` |

The roughly 35 ms v2 many-file creation microbenchmark and sequential v1/v2 hash
rates are the initial profiling targets. Optimization decisions still require a
real profiler capture as documented in `benches/profiling.md`.
