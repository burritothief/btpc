# Creating torrents

Select v1 for broad legacy compatibility, v2 for per-file Merkle verification, or
hybrid when both representations are required.

```console
btpc create payload --mode hybrid --piece-length 16384 -o payload.torrent
```

Creation traverses paths deterministically and streams payload data with bounded
memory. For reproducible output, set a fixed creation date, explicit thread count,
and explicit piece length where the protocol permits it. Tracker tiers preserve
their order; dictionary keys are always encoded in canonical raw-byte order.

Use `btpc create --json` for automation or the typed Python `create`/
`create_bytes` functions when embedding BTPC.
