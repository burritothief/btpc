# btpc-core

`btpc-core` is the safe Rust protocol engine for BTPC. It reads, validates,
creates, edits, serializes, and verifies BitTorrent v1, v2, and hybrid metainfo
while preserving exact source bytes for existing info hashes.

```rust
use btpc_core::{Metainfo, TorrentMode};

let bytes =
    b"d4:infod6:lengthi0e4:name5:empty12:piece lengthi16e6:pieces0:ee".to_vec();
let torrent = Metainfo::from_vec(bytes)?;
assert_eq!(torrent.mode(), TorrentMode::V1);
# Ok::<(), btpc_core::Error>(())
```

Creation and verification stream payload data with bounded memory. Protocol
byte strings remain bytes, canonical dictionaries use unsigned raw-byte key
ordering, and library concurrency is configured per operation rather than
through an ambient global pool.

- [Current API documentation](https://burritothief.github.io/btpc/rust/btpc_core/)
- [docs.rs API documentation after the first publish](https://docs.rs/btpc-core)
- [Project documentation](https://burritothief.github.io/btpc/rust/)
- [Repository](https://github.com/burritothief/btpc)

Licensed under the [MIT License](LICENSE).
