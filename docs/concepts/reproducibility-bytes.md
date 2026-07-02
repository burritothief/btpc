# Reproducibility and raw bytes

Torrent identity is defined by bytes. BTPC preserves the exact source `info` slice
for hashing, treats protocol byte strings as bytes, and decodes text only at explicit
API or presentation boundaries.

Canonical output sorts dictionary keys by unsigned raw-byte order. Reproducible
creation also requires stable traversal, piece length, metadata fields, creation
date, and tool configuration. A fixed `creation_date=0` and explicit thread count
are useful for tests and release fixtures.
