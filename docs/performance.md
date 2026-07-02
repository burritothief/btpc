# Performance

BTPC measures creation, parsing, verification, and Python-boundary behavior with
validated outputs before accepting timings. Performance claims record tool versions,
hardware, dataset, cache state, elapsed distribution, throughput, and peak RSS.

The checked-in benchmark harness supports reproducible synthetic fixtures and a
canonical Debian ISO preflight. Large cross-tool results are report-oriented rather
than flaky per-commit gates. See the repository's [benchmark methodology][bench]
for commands and accepted baselines.

[bench]: https://github.com/burritothief/btpc/blob/main/benches/README.md
