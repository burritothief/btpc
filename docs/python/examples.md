# Python examples

The [quick start](../getting-started/python.md) covers creation and verification.
Use `Metainfo.from_bytes` for contiguous buffers and `Metainfo.read` for direct path
I/O. Parsed raw values remain bytes, while optional text views return `None` when
UTF-8 decoding is not lossless.

```python
from btpc import UNCHANGED, Metainfo

torrent = Metainfo.read("payload.torrent")
reviewed = torrent.edit(comment="reviewed", source=UNCHANGED)
reviewed.write("payload-reviewed.torrent")
```

Top-level edits preserve info hashes; info-dictionary edits change them.
`write()` preserves the edited object's encoding and publishes atomically.
Pass `overwrite=True` to replace an existing destination. Pass `canonical=True`
to normalize bencoding, which can change info hashes for noncanonical input.

Reuse creation results directly:

```python
from btpc import create_bytes

result = create_bytes("payload")
print(result.metainfo.magnet())
assert result.metainfo.verify("payload").is_valid
```

Copy a UTF-8 comment to another torrent without replacement decoding:

```python
from btpc import Metainfo

source = Metainfo.read("source.torrent")
destination = Metainfo.read("destination.torrent")
comment = source.comment_text
if source.comment is not None and comment is None:
    raise ValueError("source comment is not UTF-8")
destination.edit(comment=comment).write("destination-reviewed.torrent")
```

Inspect an immutable extension value with ordinary collection operations:

```python
from btpc import BencodeDictionary, BencodeList

values = BencodeList((1, b"raw"))
extension = BencodeDictionary(((b"values", values),))
assert values[1:] == BencodeList((b"raw",))
assert extension[b"values"] == values
assert dict(extension) == {b"values": values}
```
