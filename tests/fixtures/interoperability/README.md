# Interoperability and regression fixtures

Every row in `manifest.tsv` declares the fixture path, generator and pinned
version, torrent mode, expected disposition, expected info hashes, source
payload checksum, and the reason the fixture exists.

Disposition meanings:

- `accept`: valid competitor output accepted without warnings.
- `accept-warning`: accepted compatibility input that must report a warning.
- `preserve`: irregular but parseable input whose exact source bytes and info
  hash must remain available; canonical output must still parse.
- `reject`: invalid input that every public binding must reject.

## Shared payload

The competitor fixtures were generated from this byte-for-byte tree:

```text
payload/alpha.txt       "alpha\n"
payload/nested/beta.bin "beta\n"
```

The manifest records a deterministic aggregate SHA-256 over the sorted
`sha256(relative path and contents)` lines: `0bddf77ab01dcf8f6ddf0482bc84091fe3921504385e632421a742520b9b1391`.
All fixtures use `https://tracker.invalid/announce`; no network tracker is
contacted.

## Generators

- `mktorrent 1.1` from the upstream `v1.1` release (Homebrew formula revision
  does not change the reported upstream version), v1 only.
- `mkbrr 1.23.0` from upstream release `v1.23.0`, v1 only.
- `torf-cli 5.2.1` from PyPI package `torf-cli==5.2.1`, v1 only.
- `torrenttools 0.6.2` from upstream release `v0.6.2`, covering v1, v2, and
  hybrid modes. Its v2 fixture includes an unnecessary empty top-level
  `piece layers` dictionary and is accepted with a compatibility warning.

Generation disabled timestamps and optional creator fields where supported and
used fixed piece lengths. The exact commands are retained in
`scripts/generate-interop-fixtures.sh`; the script requires the pinned tools to
already be installed or supplied through its documented environment variables.

## Irregular fixtures

- `valid-empty-v1.torrent`: minimal canonical reference accepted and preserved.
- `preserved-unknown-field.torrent`: unknown top-level `comment` bytes are
  retained in `original_bytes`; the `info` hash is unchanged.
- `invalid-piece-length.torrent`: a historical permissive-parser case with
  piece length 15. It is preserved rather than silently rewritten; creation APIs
  remain stricter.
- `truncated-dictionary.torrent`: malformed bencode rejected.
- `prior-fuzz-noncanonical-integer.torrent`: promoted parser fuzz seed rejected
  as invalid metainfo.
- `prior-fuzz-non-utf8-dictionary.torrent`: promoted arbitrary-byte fuzz seed
  rejected as invalid metainfo without attempting text decoding.
- `fuzz-regression-noncanonical-info.torrent`: accepted for inspection with its
  exact original v1 hash; canonical serialization normalizes a non-minimal
  integer and therefore intentionally produces a different canonical info hash.
- `fuzz-regression-unsorted-file-tree.torrent`: accepted for inspection with its
  exact original v2 hash; canonical serialization sorts raw file-tree keys while
  preserving the same semantic file set.
