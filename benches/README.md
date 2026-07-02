# Competitor benchmark harness

The Python harness implements the accepted `v1-single-file` profile from
`specs/benchmarking.md`. It fingerprints deterministic input, isolates tool
configuration, randomizes complete per-round tool blocks, samples process-tree
CPU and RSS with `psutil`, validates every successful torrent through BTPC and
an independent bencode/checksum oracle, and preserves raw artifacts.

Generate a deterministic payload:

```bash
uv run python -m benches.btpc_bench generate benchmark-data/tiny \
  --seed 20260701 --size-bytes 1048576
```

Verify and warm the canonical ISO without invoking any benchmark tool:

```bash
make benchmark-iso ISO=/path/to/debian-13.5.0-amd64-DVD-1.iso
```

This writes versioned preflight JSON with path, name, size, mtime, SHA-256, and
ordered 4 MiB SHA-1 piece digests. `benches/torrent_creation.py` is the stable
executable wrapper; `python -m benches.btpc_bench` exposes the same commands.

Run the quick preset against every registered adapter:

```bash
uv run python -m benches.btpc_bench run benchmark-data/tiny/payload.bin \
  --preset quick --tools all
```

The standard preset uses two warmups and ten randomized measured rounds. The
quick preset uses one warmup and three rounds. Override either count with
`--warmups` or `--rounds`. `--cache-state cold` is a label only; the harness
requires an explicit `--cache-prepare-command ...`; the harness never invokes
privileged cache dropping implicitly. Process trees are sampled every 10 ms by
default (`--sample-interval-ms`) and each invocation has a 600-second default
timeout (`--timeout`).

Render or compare saved results without rerunning tools:

```bash
uv run python -m benches.btpc_bench render benchmark-results/RUN/result.json
uv run python -m benches.btpc_bench compare BASELINE.json CANDIDATE.json
```

Missing or unsupported competitors remain in JSON, CSV, and summaries and do
not abort other tools unless `--require-tools` is supplied. Generated datasets,
logs, torrents, and result directories are intentionally ignored by Git.

## Competitor setup

The harness never installs tools. Review and pin versions outside the timed run;
these examples match the locally validated release-candidate matrix:

```bash
# macOS
brew install mktorrent
uv tool install 'torf-cli==5.2.1'
GOBIN="$HOME/.local/bin" go install github.com/autobrr/mkbrr@v1.23.0
# Install the signed torrenttools 0.6.2 release package from its GitHub release.

# Debian/Ubuntu (mktorrent package version varies by distribution)
sudo apt-get install mktorrent
uv tool install 'torf-cli==5.2.1'
GOBIN="$HOME/.local/bin" go install github.com/autobrr/mkbrr@v1.23.0
# Install the torrenttools 0.6.2 AppImage from its GitHub release.
```

Capture actual `--version` output in every result. The harness may mark a present
tool `UNSUPPORTED` or `SMOKE_FAILED` when its version cannot force the exact v1
profile. `torrenttools 0.6.2`, for example, adds an `info` cross-seed field and is
therefore excluded rather than silently ranked.

## Python boundary profile

Benchmark public parse-buffer variants, repeated properties, magnet, edits, and
exact equality with validated deterministic JSON and ASCII output:

```bash
uv run python -m benches.btpc_bench.python_boundary payload.torrent \
  --payload payload.bin \
  --repetitions 20 --json benchmark-results/python-boundary/result.json
```

Use at least three warmups and twenty repetitions for local comparisons. Compare
medians on the same machine, Python build, native BTPC build, fixture, and cache
state. Stable synthetic workflows use a default 1.25x regression budget; noisy
large external datasets remain report-only.
