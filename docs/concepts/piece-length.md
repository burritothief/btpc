# Piece length

Piece length balances metadata size, hashing granularity, and client behavior. v1
accepts power-of-two piece sizes. v2 and hybrid creation also require BEP 52 rules
and file-tree consistency.

Leaving piece length automatic uses BTPC's deterministic policy for the payload
size. Set an explicit value when reproducing a known torrent or comparing tools.
