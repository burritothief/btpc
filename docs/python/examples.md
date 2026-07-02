# Python examples

The [quick start](../getting-started/python.md) covers creation and verification.
Use `Metainfo.from_bytes` for contiguous buffers and `Metainfo.read` for direct path
I/O. Parsed raw values remain bytes, while optional text views return `None` when
UTF-8 decoding is not lossless.

```python
from btpc import UNCHANGED, Metainfo

torrent = Metainfo.read("payload.torrent")
reviewed = torrent.edit(comment="reviewed", source=UNCHANGED)
reviewed.to_bytes()
```

Top-level edits preserve info hashes; info-dictionary edits change them.
