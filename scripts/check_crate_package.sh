#!/usr/bin/env bash
set -euo pipefail

toolchain="${1:-stable}"
cargo=(cargo "+${toolchain}")

"${cargo[@]}" package -p btpc-core --locked --allow-dirty

crate=$(find target/package -maxdepth 1 -type f -name 'btpc-core-*.crate' -print | sort | tail -n 1)
if [[ -z "$crate" ]]; then
  echo "btpc-core package archive was not created" >&2
  exit 1
fi

temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
tar -xzf "$crate" -C "$temporary"
package_root=$(find "$temporary" -mindepth 1 -maxdepth 1 -type d -name 'btpc-core-*' -print -quit)

for required in Cargo.toml Cargo.lock LICENSE README.md examples/inspect.rs src/lib.rs; do
  if [[ ! -f "$package_root/$required" ]]; then
    echo "packaged crate is missing $required" >&2
    exit 1
  fi
done

"${cargo[@]}" test \
  --manifest-path "$package_root/Cargo.toml" \
  --locked \
  --offline \
  --all-features \
  --lib \
  --examples
"${cargo[@]}" test \
  --manifest-path "$package_root/Cargo.toml" \
  --locked \
  --offline \
  --all-features \
  --doc

consumer="$temporary/consumer"
mkdir -p "$consumer/src"
cat > "$consumer/Cargo.toml" <<EOF
[package]
name = "btpc-packaged-consumer"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
btpc-core = { path = "${package_root}" }
EOF
cat > "$consumer/src/main.rs" <<'EOF'
use btpc_core::{Metainfo, TorrentMode};

fn main() -> btpc_core::Result<()> {
    let bytes = b"d4:infod6:lengthi0e4:name5:empty12:piece lengthi16e6:pieces0:ee".to_vec();
    let torrent = Metainfo::from_vec(bytes)?;
    assert_eq!(torrent.mode(), TorrentMode::V1);
    Ok(())
}
EOF
"${cargo[@]}" check --manifest-path "$consumer/Cargo.toml" --offline
