# Python quick start

```python
from btpc import CreateOptions, TorrentMode, create

result = create(
    "payload",
    "payload-hybrid.torrent",
    options=CreateOptions(
        mode=TorrentMode.HYBRID,
        piece_length=16_384,
        creation_date=0,
        threads=1,
    ),
)
torrent = result.metainfo
assert torrent.verify("payload").is_valid
print(torrent.magnet())
torrent.edit(comment="reviewed").write("payload-reviewed.torrent")
```

Textual inputs use Python `str`; parsed protocol bytes and torrent paths remain
lossless `bytes`. See the [Python API guide](../python/index.md) for editing,
callbacks, cancellation, and errors.
