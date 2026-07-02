#!/bin/sh
set -eu

for target in parse canonical_roundtrip metainfo metainfo_roundtrip magnet; do
    destination="fuzz/corpus/$target/torrent-fixtures"
    rm -rf "$destination"
    mkdir -p "$destination"
    if [ -d tests/fixtures ]; then
        find tests/fixtures -type f -name '*.torrent' -exec cp {} "$destination"/ \;
    fi
    printf 'Synced %s fixtures to %s\n' \
        "$(find "$destination" -type f | wc -l | tr -d ' ')" "$target"
done
