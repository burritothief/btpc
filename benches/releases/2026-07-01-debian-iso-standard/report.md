# Debian ISO v1 creation benchmark — 2026-07-01

## Scope

This report covers one large-file, warm-cache, BitTorrent v1 creation profile on
one macOS arm64 machine. It does not generalize to cold cache, directory trees,
v2/hybrid torrents, other storage, or other machines. No universal fastest-tool
claim is made.

The canonical input was `debian-13.5.0-amd64-DVD-1.iso`, exactly 3,989,078,016
bytes with SHA-256
`343b6e02a8bdf6429eb3722ee0056b5c7d9ad17d88328e499909da7205e55f50`.
The profile used v1 single-file metainfo, 4 MiB pieces, private mode, tracker
`https://tracker.invalid/announce`, default tool concurrency, two untimed warmups,
ten seeded blocked rounds, and full post-timing independent validation.

## Environment and tools

- Apple M3 Max, 14 physical/logical cores, 38,654,705,664 bytes RAM.
- macOS 26.5 / Darwin 25.5.0, arm64, APFS.
- BTPC native 0.1.0 release binary and BTPC Python 0.1.0 release extension on
  CPython 3.14.3.
- mkbrr v1.23.0, mktorrent 1.1, torf-cli 5.2.1, torrenttools v0.6.2.

Exact paths, command arrays, and host metadata are retained in the committed
`primary/` and `replication/` metadata files.

## Primary session

Command:

```console
uv run maturin develop --release
uv run python benches/torrent_creation.py run \
  debian-13.5.0-amd64-DVD-1.iso \
  --output-root benchmark-results/todo52-standard-release \
  --preset standard --seed 20260703 --tools all
```

The session completed in 46.29 seconds wall time. Every ranked tool produced ten
valid samples and identical raw v1 info hash
`6fd397c6de29f77d0f0c1928e6c457240112204c`.

| Tool | Median | Mean ± SD | CV | Throughput | Peak RSS | Relative |
|---|---:|---:|---:|---:|---:|---:|
| mkbrr v1.23.0 | 0.247 s | 0.251 ± 0.013 s | 5.20% | 15,387.6 MiB/s | 74.5 MiB | 1.00× |
| mktorrent 1.1 | 0.519 s | 0.523 ± 0.019 s | 3.60% | 7,331.6 MiB/s | 26.1 MiB | 2.10× |
| torf-cli 5.2.1 | 0.642 s | 0.648 ± 0.027 s | 4.22% | 5,924.5 MiB/s | 95.0 MiB | 2.60× |
| BTPC native 0.1.0 | 0.816 s | 0.821 ± 0.021 s | 2.60% | 4,661.7 MiB/s | 23.1 MiB | 3.30× |
| BTPC Python 0.1.0 | 0.861 s | 0.861 ± 0.006 s | 0.71% | 4,419.4 MiB/s | 39.9 MiB | 3.48× |

`torrenttools v0.6.2` was discovered and smoke-run but was correctly unranked:
it inserts an extra cross-seed field in the `info` dictionary, changing the raw
info hash and violating the comparable profile.

## Replication

Because mkbrr's primary-session CV was 5.20%, a second complete session was run
with seed `20260704`; no samples were removed. It completed in 46.05 seconds.
Median changes versus the primary were BTPC native +0.96%, BTPC Python -0.19%,
mktorrent -0.44%, torf-cli +0.64%, and mkbrr +6.88%. The other four tools were
highly reproducible; mkbrr remained the fastest but showed the greatest session
variation and a 7.24% within-session CV in the replication.

## Raw evidence

Ignored raw directories retained locally:

- `benchmark-results/todo52-standard-release/20260701T233001.755832Z/`
- `benchmark-results/todo52-standard-release-rerun/20260701T233054.341474Z/`

Primary SHA-256:

| File | SHA-256 |
|---|---|
| `results.json` | `da7be2bc5d6cb67d16b8e97c580daf337ebed5d369aeb6cc3e1bd0cb025540d7` |
| `samples.csv` | `c348b87150fdcebcf1d2cee0b1311f49009599a155a34d186697ebafbd1d414c` |
| `environment.json` | `b0392df05c0d22e807e6f5d1f129b4ac0ac47e717fe7f89ffb755d84f75c3c53` |
| `commands.json` | `4db2638c72d5e50d2227f7fb3fda783f8cbc36bb558e77d69d79fd8cbaf1731c` |
| `summary.txt` | `03d606713ad24116313c349d42d192a4918dd39962bf527ed3db3b402e02095a` |

Replication SHA-256:

| File | SHA-256 |
|---|---|
| `results.json` | `ac85514977b2a03b76177548575f965ceb53f480ecb50ab35e2af2792961e04f` |
| `samples.csv` | `85e83887303a2a7e82c8a1b744f047d0dfdf2ef3c48c6b349b46c72c64930b27` |
| `environment.json` | `b0392df05c0d22e807e6f5d1f129b4ac0ac47e717fe7f89ffb755d84f75c3c53` |
| `commands.json` | `9b6a7f2932b0ae965b75b4e96a60189493c2dc7ad79f32c4efc444f36ac5acfc` |
| `summary.txt` | `90f6ec062854e5c275e623c75575975a52f10cdac294de8fb7502460752d3375` |

Two earlier standard sessions used a debug editable Python extension and are
retained under `benchmark-results/todo52-standard*/`; they are excluded from the
published table because that build mode performed different work from the release
native binary. This was corrected before the primary session rather than hidden.
