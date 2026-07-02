---
spec_id: BENCH
title: "Torrent Creation Benchmarking"
status: Accepted
owners:
  - "performance maintainers"
source_paths: []
test_paths: []
last_reviewed: "2026-07-01"
---

# Torrent Creation Benchmarking

## Purpose

This specification defines a reproducible end-to-end benchmark for torrent
creation CLIs using `debian-13.5.0-amd64-DVD-1.iso`. It compares equivalent work,
rejects invalid output, retains raw samples, and produces a readable ASCII summary.

The initial harness is a focused realization of `PERF-BENCH-001`. It is not a
microbenchmark and does not replace later directory-tree or v2/hybrid benchmarks.

## Canonical Dataset

- Path: `debian-13.5.0-amd64-DVD-1.iso`, configurable by CLI argument.
- Exact size: `3,989,078,016` bytes.
- SHA-256: `343b6e02a8bdf6429eb3722ee0056b5c7d9ad17d88328e499909da7205e55f50`.
- Torrent name: `debian-13.5.0-amd64-DVD-1.iso`.

The harness may benchmark another file only when its path, size, SHA-256, and
display name are recorded in the result metadata. Published Debian ISO results
must match the canonical fingerprint above.

## Comparable Work Profile

The primary leaderboard uses one semantic profile:

| Setting | Required value |
| --- | --- |
| Metainfo mode | BitTorrent v1, single-file |
| Piece length | 4,194,304 bytes (`2^22`, 4 MiB) |
| Private | `true` |
| Announce URL | `https://tracker.invalid/announce` |
| Name | Exact input basename |
| Source/comment/web seeds | Omitted |
| Progress/interactive UI | Disabled or redirected |
| Concurrency | Tool default/automatic |

Creation date and creator fields should be omitted when the CLI supports that
without changing its hashing algorithm. They are top-level, excluded from semantic
equivalence, and their unavoidable presence does not make a tool non-comparable.

The optional single-worker diagnostic profile uses the same settings and pins one
worker only for tools with a documented equivalent option. It is reported in a
separate table and must not be merged with the default-concurrency leaderboard.

## Run Protocol

### BENCH-DATA-001 — Verify and warm the complete input

- **Status:** Implemented
- **Sources:** `benches/btpc_bench/dataset.py`, `benches/btpc_bench/runner.py`
- **Verification:** `tests/benchmarks/test_harness.py`
- **Depends on:** None

Before timing, the harness **MUST** stream the complete input once, calculate its
SHA-256, calculate ordered SHA-1 digests for every 4 MiB v1 piece, record file
metadata, and fail if a required fingerprint differs. This untimed preflight is
also the explicit cache warm-up for the primary warm-cache benchmark.

### BENCH-PROFILE-001 — Require equivalent torrent semantics

- **Status:** Implemented
- **Sources:** `benches/btpc_bench/adapters.py`,
  `benches/btpc_bench/validation.py`
- **Verification:** `tests/benchmarks/test_harness.py`
- **Depends on:** `BENCH-DATA-001`

Every comparable command **MUST** use the primary profile above. An adapter that
cannot force v1, 4 MiB pieces, private mode, exact name, and the specified tracker
**MUST** be reported as `UNSUPPORTED`, not included in rankings.

### BENCH-ENV-001 — Isolate configuration and capture the environment

- **Status:** Implemented
- **Sources:** `benches/btpc_bench/runner.py`, `benches/btpc_bench/models.py`
- **Verification:** `tests/benchmarks/test_harness.py`
- **Depends on:** `BENCH-PROFILE-001`

The harness **MUST** run tools with a temporary empty home/config directory and
stable locale, timezone, color, and Python hash settings so user presets cannot
alter behavior. It **MUST** record OS/kernel, architecture, CPU, logical cores,
memory, filesystem, Python/Rust versions when relevant, tool paths/versions,
command templates, input fingerprint, harness version, timestamp, run seed, and
power/cache warnings available without privileged commands.

### BENCH-RUN-001 — Use blocked randomized repeated measurements

- **Status:** Implemented
- **Sources:** `benches/btpc_bench/runner.py`
- **Verification:** `tests/benchmarks/test_harness.py`
- **Depends on:** `BENCH-ENV-001`

The standard preset **MUST** perform two untimed warm-up invocations per comparable
tool followed by ten measured rounds. Each round **MUST** run every tool exactly
once in a seeded randomized order. The quick preset **MAY** use one warm-up and
three measured rounds. Output files **MUST** be removed before timing, process
stdout/stderr **MUST** go to per-run logs, and validation **MUST** occur after the
timer stops. The harness **MUST NOT** automatically discard outliers.

### BENCH-CACHE-001 — Label cache state honestly

- **Status:** Implemented
- **Sources:** `benches/btpc_bench/runner.py`, `benches/btpc_bench/cli.py`
- **Verification:** `tests/benchmarks/test_harness.py`
- **Depends on:** `BENCH-DATA-001`, `BENCH-RUN-001`

The primary result **MUST** be labeled `warm-cache`; the harness **MUST NOT** claim
cold-cache behavior. An optional cold-cache experiment **MAY** run only through an
explicit caller-supplied preparation command and **MUST** be emitted as a separate
profile with its command and privilege assumptions recorded.

### BENCH-METRIC-001 — Capture robust timing and resource metrics

- **Status:** Implemented
- **Sources:** `benches/btpc_bench/runner.py`, `benches/btpc_bench/models.py`
- **Verification:** `tests/benchmarks/test_harness.py`
- **Depends on:** `BENCH-RUN-001`

For every measured invocation, the harness **MUST** record exit status, monotonic
wall time, throughput in MiB/s, output size, and validation status. It **SHOULD**
sample peak resident memory for the complete process tree and record aggregate
user/system CPU time where the platform permits reliable collection. Summaries
**MUST** include sample count, median, mean, standard deviation, minimum, maximum,
median absolute deviation, coefficient of variation, median throughput, maximum
peak RSS, and speed relative to the fastest valid median.

### BENCH-VALID-001 — Validate every measured torrent outside timing

- **Status:** Implemented
- **Sources:** `benches/btpc_bench/validation.py`,
  `benches/btpc_bench/runner.py`
- **Verification:** `tests/benchmarks/test_harness.py`
- **Depends on:** `BENCH-DATA-001`, `BENCH-PROFILE-001`, `META-V1-001`

After every successful process exit, the harness **MUST** parse the generated
torrent and verify v1 single-file mode, exact name, length, piece length, private
flag, tracker, expected piece count, and every concatenated SHA-1 piece digest from
preflight. It **MUST** compute the raw v1 info hash and require all comparable tools
to produce the same info hash. Failed or invalid runs **MUST** remain in raw output,
mark the tool invalid, and exclude it from ranked performance rows.

### BENCH-TOOLS-001 — Use explicit versioned tool adapters

- **Status:** Implemented
- **Sources:** `benches/btpc_bench/adapters.py`, `benches/btpc_bench/runner.py`
- **Verification:** `tests/benchmarks/test_harness.py`
- **Depends on:** `BENCH-PROFILE-001`

The initial adapter registry **MUST** cover BTPC, `mkbrr`, `mktorrent`, `torf-cli`,
and `torrenttools`. Each adapter **MUST** declare executable discovery, version
command, command template, piece-length unit/exponent semantics, output behavior,
private/tracker flags, optional no-date/no-creator flags, and supported profiles.
The harness **MUST** smoke-validate an adapter before measured runs and report
missing binaries, unsupported versions, or command failures without aborting other
tools unless the caller uses `--require-tools`.

### BENCH-OUTPUT-001 — Preserve raw data and print an ASCII leaderboard

- **Status:** Implemented
- **Sources:** `benches/btpc_bench/report.py`, `benches/btpc_bench/models.py`,
  `benches/btpc_bench/cli.py`
- **Verification:** `tests/benchmarks/test_harness.py`
- **Depends on:** `BENCH-METRIC-001`, `BENCH-VALID-001`

Each benchmark **MUST** create a timestamped result directory containing a raw JSON
document, CSV samples, per-run logs, retained final torrents, environment metadata,
and `summary.txt`. Standard output **MUST** print the same fixed-width ASCII table,
sorted by valid median wall time, with columns for tool, version, status, runs,
median, mean ± standard deviation, min–max, median MiB/s, peak RSS, relative speed,
and info-hash prefix. Unavailable, unsupported, failed, and invalid tools **MUST**
appear below ranked rows with an actionable status/reason.

Example shape:

```text
+------------+----------+-------+---------+-------------+-----------+---------+----------+--------+
| Tool       | Version  | Runs  | Median  | Mean +/- SD | Min..Max  | MiB/s   | Peak RSS | Rel.   |
+------------+----------+-------+---------+-------------+-----------+---------+----------+--------+
| mkbrr      | 1.2.3    | 10/10 | 1.842 s | 1.851+-.031 | 1.81..1.9 | 1131.3  | 42.1 MiB | 1.00x  |
+------------+----------+-------+---------+-------------+-----------+---------+----------+--------+
```

### BENCH-REPRO-001 — Make reruns and comparisons explicit

- **Status:** Implemented
- **Sources:** `benches/torrent_creation.py`, `benches/btpc_bench/cli.py`,
  `benches/btpc_bench/models.py`
- **Verification:** `tests/benchmarks/test_harness.py`
- **Depends on:** `BENCH-OUTPUT-001`

The helper CLI **MUST** accept input path, output root, tool selection, preset,
warm-up count, measured rounds, seed, tracker, piece exponent, profile, and
`--require-tools`. It **MUST** support rendering a saved JSON result without rerunning
tools and comparing two saved result files without declaring statistical
significance from insufficient samples.

## Statistics and Interpretation

Median wall time is the primary ranking metric because it is robust to occasional
background-system noise. Mean, standard deviation, median absolute deviation, and
range expose stability. All samples remain available; suspected outliers are
flagged but never deleted automatically. Publication-quality claims require the
standard preset, coefficient-of-variation review, and at least one repeated session.

Results describe this machine, input, cache state, tool versions, and profile only.
They must not be generalized to directory traversal, small files, v2/hybrid mode,
other storage, or cold-cache performance.

## Design Rationale

Hyperfine, Criterion, and `pytest-benchmark` are excellent within their intended
domains, but this workload needs per-run torrent validation, process-tree resource
sampling, configuration isolation, and adapter-specific outputs outside the timed
region. A small Python orchestrator follows standard repeated-measurement practice
while retaining control over correctness. The script should use the standard
library for timing/statistics and `psutil` for best-effort process-tree metrics;
the ASCII renderer should remain dependency-light and deterministic.
