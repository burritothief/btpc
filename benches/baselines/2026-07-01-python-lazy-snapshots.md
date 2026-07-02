# Python lazy snapshot memory baseline

Date: 2026-07-01 PDT
Machine: Apple Silicon macOS development host, CPython 3.14.3, Rust 1.94.1
Command: `uv run python scripts/benchmark_python_snapshot_memory.py ...`
Metric: process peak RSS from `resource.getrusage`; three isolated processes per mode.

| Fixture | Mode | Peak RSS samples |
| --- | --- | --- |
| 20,000 zero-length files, 1,060,066-byte metainfo | lazy | 57,311,232; 57,835,520; 57,376,768 bytes |
| 20,000 zero-length files, 1,060,066-byte metainfo | materialized | 69,369,856; 69,517,312; 69,402,624 bytes |
| 10 files plus 16 MiB unknown byte string, 16,777,827-byte metainfo | lazy | 110,657,536; 110,657,536; 110,657,536 bytes |
| 10 files plus 16 MiB unknown byte string, 16,777,827-byte metainfo | materialized | 161,251,328; 161,218,560; 161,234,944 bytes |

“Lazy” parses and retains the facade without accessing source bytes, canonical
bytes, files, trackers, web seeds, unknown fields, or validation. “Materialized”
accesses every one of those properties. The many-file fixture saves about 12 MiB
peak RSS. The large-metainfo fixture saves about 50 MiB, consistent with adopting
the native input buffer and deferring Python source/canonical byte copies. The
benchmark is diagnostic, not a correctness substitute; API tests directly assert
cache state and repeated-property identity.
