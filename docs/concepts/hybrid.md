# Hybrid torrents

A hybrid torrent carries matching v1 and v2 representations for the same payload.
BTPC creates and verifies both hash domains. Alignment padding may appear in the v1
view so file boundaries satisfy the v2 layout.

Hybrid magnets include both `btih` and `btmh` exact-topic parameters in deterministic
order.
