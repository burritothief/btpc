#!/bin/sh
set -eu

root=${1:-tests/fixtures/interoperability/competitors}
payload=${2:-.tmp/interop-fixtures/payload}
mkbrr=${MKBRR:-mkbrr}
torrenttools=${TORRENTTOOLS:-torrenttools}

mkdir -p "$root" "$payload/nested"
printf 'alpha\n' > "$payload/alpha.txt"
printf 'beta\n' > "$payload/nested/beta.bin"

mktorrent -a https://tracker.invalid/announce -d -l 15 \
  -o "$root/mktorrent-1.1-v1.torrent" "$payload"
"$mkbrr" create "$payload" --tracker https://tracker.invalid/announce \
  --private=false --piece-length 16 --no-date --no-creator --skip-prefix \
  --output "$root/mkbrr-1.23.0-v1.torrent" --quiet
uvx --from torf-cli==5.2.1 torf "$payload" \
  --tracker https://tracker.invalid/announce --date '1970-01-01 00:00:00' \
  --creator 'torf 5.2.1' --max-piece-size 1 \
  -o "$root/torf-cli-5.2.1-v1.torrent"
for mode in 1 2 hybrid; do
  name=$mode
  [ "$mode" = 1 ] && name=v1
  [ "$mode" = 2 ] && name=v2
  "$torrenttools" create --protocol "$mode" \
    --announce https://tracker.invalid/announce --piece-size 16K \
    --no-creation-date --no-created-by --no-cross-seed --simple-progress \
    --output "$root/torrenttools-0.6.2-$name.torrent" "$payload"
done
