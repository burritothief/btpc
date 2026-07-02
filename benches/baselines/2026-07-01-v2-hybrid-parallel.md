# Bounded v2 and hybrid file concurrency — 2026-07-01

Profiling showed v2 hashing dominated 128 MiB single-file creation (~68 ms), while
hybrid many-file creation spent ~547 ms in hashing because each short real file is
followed by alignment padding. The smallest measured change was bounded file-level
concurrency for many-file inputs. Single-file paths remain sequential because
parallel setup did not improve them.

## Design

- V2 assigns manifest entries to at most the selected worker count. Each worker
  opens and hashes one file with the retained transparent sequential Merkle oracle.
- Hybrid hashes each alignment-independent real-file-plus-padding domain on the
  same bounded worker model, then restores manifest order and padding offsets.
- Result collectors preserve deterministic ordering and ordered live progress.
- An originating I/O error is preferred over secondary cancellation errors.
- Active file descriptors are bounded by the selected workers; tests instrument
  the real v2 worker path and observe no excess.
- `HashThreads::Exact(1)` remains the v2/hybrid oracle. Automatic selection uses
  two workers on multi-core systems and avoids worker setup for one-file manifests.

## Correctness coverage

Tests compare sequential and optimized outputs for fixed v2/hybrid fixtures,
32 property-generated trees, many alignment-padding files, forced out-of-order
file completion, missing-file worker errors, preflight and callback-driven
mid-flight cancellation, ordered progress, and real active-worker descriptor
bounds. Every measured torrent was byte-identical between paths and passed both
`btpc validate` and `btpc verify`.

## Criterion many-file comparison

Command:

```bash
cargo bench -p btpc-core --bench core -- file_tree_creation \
  --warm-up-time 0.1 --measurement-time 0.3 --sample-size 10 --noplot
```

| Mode | Threads | Estimate | Relative |
|---|---:|---:|---:|
| v2 | 1 | 36.523 ms | 1.00× |
| v2 | 2 | 25.217 ms | 1.45× |
| hybrid | 1 | 88.355 ms | 1.00× |
| hybrid | 2 | 59.035 ms | 1.50× |

## End-to-end comparison

Environment and deterministic 128 MiB datasets match the todo 30 report. Ten
randomized blocked rounds compared one versus two workers for each mode/dataset.
Raw ignored artifacts are under `benchmark-results/todo-31/final/`.

| Dataset | Mode | Threads | Median | Mean | SD | MAD | CV | MiB/s | Max RSS |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| one file | v2 | 1 | 125.762 ms | 126.523 ms | 4.557 ms | 1.620 ms | 3.60% | 1,017.80 | 7,045,120 B |
| one file | v2 | 2 | 127.585 ms | 168.782 ms | 123.911 ms | 2.313 ms | 73.41% | 1,003.25 | 7,045,120 B |
| one file | hybrid | 1 | 181.042 ms | 188.044 ms | 19.704 ms | 1.851 ms | 10.48% | 707.02 | 11,485,184 B |
| one file | hybrid | 2 | 181.061 ms | 184.090 ms | 8.934 ms | 2.240 ms | 4.85% | 706.94 | 11,501,568 B |
| 256-file tree | v2 | 1 | 128.016 ms | 178.717 ms | 145.718 ms | 2.991 ms | 81.54% | 999.88 | 8,192,000 B |
| 256-file tree | v2 | 2 | 101.086 ms | 113.206 ms | 28.558 ms | 1.243 ms | 25.23% | 1,266.25 | 8,241,152 B |
| 256-file tree | hybrid | 1 | 606.614 ms | 613.338 ms | 22.079 ms | 9.866 ms | 3.60% | 211.01 | 16,744,448 B |
| 256-file tree | hybrid | 2 | 325.687 ms | 326.320 ms | 7.112 ms | 3.509 ms | 2.18% | 393.02 | 17,530,880 B |

Many-file median speedups are **1.27×** for v2 and **1.86×** for hybrid. Peak RSS
increases by only 49 KiB for v2 and 786 KiB for hybrid. Single-file v2 differs by
−1.45% and hybrid by −0.01%, both below the release gate's unexplained 5%
regression threshold and avoided in automatic operation because one-file manifests
use the sequential path. The two-worker v2 single-file mean/CV contain a scheduler
outlier; median/MAD and identical RSS confirm no algorithmic memory regression.
