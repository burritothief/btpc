# Traversal and large-metainfo inspection — 2026-07-01

A 10,000-file analogue was used because creating 100,000 files in a local smoke
checkout would add substantial filesystem churn without changing the identified
per-file costs. The tree contains 100 directories and 10,000 32-byte files. V1,
v2, and hybrid use 16 KiB pieces and two hash workers.

## Profile

Before optimization, median creation phase timings from three release runs were:

| Mode | Scan | Hash | Serialize | Torrent size |
|---|---:|---:|---:|---:|
| v1 | 60 ms | 400 ms | 2 ms | 480,465 B |
| v2 | 57 ms | 237 ms | 6 ms | 841,192 B |
| hybrid | 62 ms | 700 ms | 16 ms | 2,094,370 B |

Traversal inspection found two repeated per-file costs in the default path:

1. The walker called `symlink_metadata`, then `collect_file` called `metadata`
   twice, despite already holding an initial regular-file snapshot.
2. Every file allocated a lossy component vector and joined path string even when
   both include and exclude glob sets were empty.

The focused change passes the walker's metadata snapshot into `collect_file`,
retains one fresh metadata call for mutation detection, and skips match-path
construction entirely when no filters are configured. Filtered behavior is covered
by parity and existing policy tests.

## Results

After optimization, five release runs produced:

| Mode | Scan before | Scan after | Change | Hash after | Serialize after |
|---|---:|---:|---:|---:|---:|
| v1 | 60 ms | 47 ms | −21.7% | 378 ms | 2 ms |
| v2 | 57 ms | 46 ms | −19.3% | 222 ms | 4 ms |
| hybrid | 62 ms | 46 ms | −25.8% | 519 ms | 15 ms |

The separate Criterion `manifest_scan` benchmark scans 2,048 one-byte files in
74.475 ms (27.499 K files/s) on the baseline machine. It now provides a stable
kernel for future traversal changes.

## Inspection

Five `/usr/bin/time -l` runs inspected the 2,094,370-byte hybrid metainfo. Median
wall time remained 30 ms; maximum RSS fell from 70,254,592 to 69,320,704 bytes
(−1.3%). Parsing/validation therefore was not changed: the current latency was not
a demonstrated bottleneck relative to creation, and restructuring the borrowed
parse tree or owned inspection snapshot would add complexity without measured
benefit. Existing bencode parse/encode Criterion kernels remain the baseline.

Raw ignored artifacts are under `benchmark-results/todo-32/` and
`benchmark-results/todo-32-after/`. All generated torrents were validated by the
normal creation and metainfo conformance suites; no dependency or parallel walker
was added.
