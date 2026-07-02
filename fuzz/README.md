# BTPC fuzzing

Fuzzing uses `cargo-fuzz` and nightly Rust, but is intentionally separate from
the normal stable unit-test gate.

```console
cargo +stable install cargo-fuzz --locked
rustup toolchain install nightly --profile minimal
cargo +nightly fuzz run parse fuzz/corpus/parse -- -max_total_time=60
cargo +nightly fuzz run canonical_roundtrip fuzz/corpus/canonical_roundtrip -- -max_total_time=60
cargo +nightly fuzz run metainfo fuzz/corpus/metainfo -- -max_total_time=60
cargo +nightly fuzz run metainfo_roundtrip fuzz/corpus/metainfo_roundtrip -- -max_total_time=60
cargo +nightly fuzz run magnet fuzz/corpus/magnet -- -max_total_time=60
```

The parse corpus contains scalars, containers, malformed/non-canonical values,
deep nesting, duplicate and non-UTF-8 keys. Typed targets cover validated
metainfo conversion and accessors, canonical serialization with hash-preserving
reparse, and default/minimal magnet generation. Scheduled CI runs every target
for five minutes. Copy all committed `.torrent` fixtures into every target with
`scripts/sync-fuzz-corpus.sh`.
