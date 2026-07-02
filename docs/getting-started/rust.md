# Rust quick start

```rust
use btpc_core::create::{CreateMode, CreateOptions, Creator, NoProgress, PieceLength};

let options = CreateOptions::builder()
    .mode(CreateMode::Hybrid)
    .piece_length(PieceLength::Exact(16_384))
    .creation_date(0)
    .build()?;
let result = Creator::new("payload").options(options).create(&NoProgress)?;
println!("{} bytes", result.bytes().len());
# Ok::<(), btpc_core::Error>(())
```

Protocol logic, traversal, hashing, canonical serialization, and verification live
in `btpc-core`. Continue with the [Rust API guide](../rust/index.md).
